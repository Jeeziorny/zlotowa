# 58 — Backup & Restore

## Problem

After months of use, a user's database contains curated data that can't be easily recreated: expenses with manually assigned categories, classification rules built up over time, title cleanup rules, and budget configurations. The existing export (CSV) is a data extraction tool — it doesn't capture rules/budgets, and the bulk-upload path can't reimport its output faithfully (categories get reclassified, display_title is lost). There is no way to create a full snapshot of the app state and restore it later.

## Goal

Add **backup** (export full app state to a single JSON file) and **restore** (import that file back) as new operations, available in both CLI and GUI. These are separate from the existing CSV export and bulk-upload — they use a fixed format the app controls, skip all parsing/classification heuristics, and handle the complete data set.

## Backup Format

Single `.json` file:

```json
{
  "version": 1,
  "app": "4ccountant",
  "exported_at": "2026-02-27T14:30:00Z",
  "expenses": [
    {
      "title": "UBER TRIP",
      "display_title": "Uber Trip",
      "amount": 12.99,
      "date": "2025-01-16",
      "category": "Transport",
      "classification_source": "llm"
    }
  ],
  "classification_rules": [
    { "pattern": "(?i)uber", "category": "Transport" }
  ],
  "title_cleanup_rules": [
    { "pattern": "UBER TRIP", "replacement": "Uber Trip", "is_regex": false }
  ],
  "budgets": [
    {
      "ref": 1,
      "start_date": "2026-01-01",
      "end_date": "2026-01-31",
      "categories": [
        { "category": "Transport", "amount": 200.0 },
        { "category": "Groceries", "amount": 500.0 }
      ]
    }
  ]
}
```

### Design decisions

- **No internal IDs.** Database `id` and `batch_id` are omitted — they're auto-generated on insert and meaningless across databases.
- **Budgets are nested.** `budget_categories` are inlined under their parent budget to preserve the FK relationship without exposing DB IDs. The `ref` field is a local reference within the file (not a DB ID), used only if future format versions need cross-references.
- **`classification_source` as string.** Matches the existing DB storage format (`"database"`, `"llm"`, `"manual"`).
- **`upload_batches` excluded.** Batch metadata is operational (for undo); in a restore scenario it has no meaning — the expenses themselves are what matter.
- **`config` excluded.** LLM keys are machine-specific and sensitive. `active_widgets` is a minor UI preference not worth the complexity of selective key filtering.
- **`version` field.** Allows evolving the format later without breaking older backups.

## Restore Behavior

### Conflict strategy

The restore target may not be empty — the user might be merging data from another machine or restoring a partial backup alongside existing data.

| Table | Conflict detection | Strategy |
|---|---|---|
| `expenses` | `(title, amount, date)` tuple | **Skip** duplicates (same as bulk-upload) |
| `classification_rules` | `pattern` (UNIQUE) | **Upsert** — `INSERT OR REPLACE`, backup wins |
| `title_cleanup_rules` | `(pattern, replacement, is_regex)` (UNIQUE index) | **Upsert** — `INSERT OR REPLACE`, backup wins |
| `budgets` | Overlapping `start_date`/`end_date` | **Skip** if any overlap exists with an existing budget |

### Restore should:
1. Validate the JSON structure and `version` field before touching the DB
2. Run inside a single transaction — all or nothing
3. Return a summary: counts of inserted/skipped per table

### Restore should NOT:
- Run the classification pipeline (categories are already assigned)
- Generate classification rules from restored expenses (rules are restored directly)
- Apply title cleanup rules (display_title is already set)
- Delete any existing data

## Implementation

### 1. Core — `crates/core/src/backup.rs` (new module)

New module with two public functions. Add `mod backup;` and `pub use backup::*;` to `crates/core/src/lib.rs`.

```rust
// Backup data structures (serde-serializable)
pub struct BackupData {
    pub version: u32,
    pub app: String,
    pub exported_at: String,
    pub expenses: Vec<BackupExpense>,
    pub classification_rules: Vec<BackupClassificationRule>,
    pub title_cleanup_rules: Vec<BackupTitleCleanupRule>,
    pub budgets: Vec<BackupBudget>,
}

pub struct BackupExpense {
    pub title: String,
    pub display_title: Option<String>,
    pub amount: f64,
    pub date: String,             // "YYYY-MM-DD"
    pub category: Option<String>,
    pub classification_source: Option<String>,
}

pub struct BackupClassificationRule {
    pub pattern: String,
    pub category: String,
}

pub struct BackupTitleCleanupRule {
    pub pattern: String,
    pub replacement: String,
    pub is_regex: bool,
}

pub struct BackupBudget {
    pub ref_id: u32,              // local reference, not DB id
    pub start_date: String,       // "YYYY-MM-DD"
    pub end_date: String,
    pub categories: Vec<BackupBudgetCategory>,
}

pub struct BackupBudgetCategory {
    pub category: String,
    pub amount: f64,
}

pub struct RestoreSummary {
    pub expenses_inserted: usize,
    pub expenses_skipped: usize,
    pub rules_inserted: usize,
    pub rules_updated: usize,
    pub cleanup_rules_inserted: usize,
    pub cleanup_rules_updated: usize,
    pub budgets_inserted: usize,
    pub budgets_skipped: usize,
}
```

**`create_backup(db: &Database) -> Result<BackupData, BackupError>`**

Reads all tables, assembles `BackupData`. For budgets: query `budgets` table, then for each budget query its `budget_categories` and nest them.

DB methods needed (most already exist):
- `db.get_all_expenses()` — exists
- `db.get_all_rules()` — exists
- `db.get_all_title_cleanup_rules()` — exists
- `db.list_budgets()` — exists, returns `Vec<Budget>`
- `db.get_budget_categories(budget_id)` — exists

**`restore_backup(db: &Database, data: &BackupData) -> Result<RestoreSummary, BackupError>`**

Validates version, inserts data in a single transaction. Uses `db.check_duplicates_batch()` for expense dedup. Uses `INSERT OR REPLACE` for rules. Checks budget date overlap before inserting each budget.

New DB method needed:
- `db.check_budget_overlap(start_date, end_date) -> Result<bool, DbError>` — the `check_budget_overlap` IPC command already does this inline, but it should be extracted to a DB method.

**`BackupError` enum:**
- `Io(std::io::Error)`
- `Json(serde_json::Error)`
- `UnsupportedVersion(u32)`
- `Db(DbError)`
- `InvalidData(String)`

### 2. CLI — `crates/cli/src/main.rs`

Two new subcommands:

```rust
/// Backup all data to a JSON file
Backup {
    /// Output file path (default: 4ccountant-backup-YYYY-MM-DD.json)
    path: Option<PathBuf>,
},

/// Restore data from a backup file
Restore {
    /// Path to backup JSON file
    path: PathBuf,
},
```

**`cmd_backup`**: Calls `create_backup()`, serializes to pretty JSON, writes to file. Default filename: `4ccountant-backup-{date}.json`.

**`cmd_restore`**: Reads file, deserializes, calls `restore_backup()`, prints summary table (inserted/skipped counts per table). Prompts for confirmation before proceeding with `dialoguer::Confirm`.

### 3. Tauri — `src-tauri/src/lib.rs`

Two new IPC commands:

**`backup_database(state, path: String) -> Result<(), String>`**
- Lock DB, call `create_backup()`, release lock
- Serialize to JSON, write to `path`

**`restore_database(state, path: String) -> Result<RestoreSummary, String>`**
- Read file, deserialize, validate
- Lock DB, call `restore_backup()`, release lock
- Return summary to frontend

Register both in `invoke_handler![]`. Add `RestoreSummary` to serde derives.

### 4. Frontend — `src/lib/settings/BackupRestore.svelte`

New component added to `Settings.svelte` alongside `LlmSettings` and `UploadHistory`.

**Backup section:**
- "Create Backup" button
- Calls `save()` from `@tauri-apps/plugin-dialog` with filter `[{ name: "JSON", extensions: ["json"] }]` and default filename `4ccountant-backup-{date}.json`
- Calls `invoke("backup_database", { path })`
- Shows success message with file path

**Restore section:**
- "Restore from Backup" button
- Calls `open()` from `@tauri-apps/plugin-dialog` with JSON filter
- Calls `invoke("restore_database", { path })`
- Shows summary: "Restored: 142 expenses (8 skipped), 23 rules, 3 budgets"

Follow existing patterns from `ExportPanel.svelte` for dialog usage and `LlmSettings.svelte` for settings card styling.

### 5. Tauri capabilities

Add `dialog:allow-open` to `src-tauri/capabilities/default.json` if not already present (currently has `dialog:allow-save` for export).

## File changes

| File | Change |
|---|---|
| `crates/core/src/backup.rs` | **New** — `BackupData` structs, `create_backup()`, `restore_backup()`, `BackupError` |
| `crates/core/src/lib.rs` | Add `mod backup; pub use backup::*;` |
| `crates/core/Cargo.toml` | Add `serde_json` dependency (if not already present) |
| `crates/core/src/db.rs` | Extract `check_budget_overlap()` as a DB method |
| `crates/cli/src/main.rs` | Add `Backup` and `Restore` subcommands + handlers |
| `src-tauri/src/lib.rs` | Add `backup_database` and `restore_database` IPC commands |
| `src/lib/settings/BackupRestore.svelte` | **New** — backup/restore UI in settings |
| `src/lib/Settings.svelte` | Import and render `BackupRestore` |
| `src-tauri/capabilities/default.json` | Add `dialog:allow-open` if missing |

## Tests

In `crates/core/src/backup.rs`:
- `test_backup_roundtrip` — backup → restore into empty DB → backup again → compare
- `test_restore_skips_duplicate_expenses` — insert expenses, restore same backup, verify no duplicates
- `test_restore_upserts_rules` — existing rule with different category, restore overwrites
- `test_restore_skips_overlapping_budgets` — existing budget in same date range, restore skips
- `test_restore_is_atomic` — corrupt one expense in the middle, verify nothing was inserted
- `test_restore_rejects_unsupported_version` — version 999 returns error
- `test_backup_empty_database` — produces valid JSON with empty arrays

## Out of scope

- Scheduled/automatic backups
- Backup encryption or compression
- Diffing two backup files
- Partial restore (e.g., only rules)
- `config` table backup (LLM keys, widget state)
- `upload_batches` backup
