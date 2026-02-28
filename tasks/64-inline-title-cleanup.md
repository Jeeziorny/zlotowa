# Task 64: Inline Title Cleanup (Replace Rules Engine with Bulk Upload Step)

**Track:** Full-stack — Simplification (frontend + backend)
**Blocked by:** nothing
**Blocks:** nothing
**Supersedes:** Tasks 21, 28, 52, 54 (all title cleanup related)

## Problem

The title cleanup system is overengineered. It has a persistent rules engine with CRUD, a separate sub-view buried inside ExpenseList, a preview/apply workflow, rule ordering that matters but isn't controllable, auto-application during bulk import that contradicts its own help text ("rules are applied manually"), and backup/restore integration — all for what is fundamentally a find-and-replace operation on bank transaction titles.

**UX issues identified:**

1. **Discoverability** — hidden behind "Clean Titles" button in ExpenseList toolbar, no indication it exists elsewhere
2. **Workflow friction** — after bulk import, user must navigate to ExpenseList > Clean Titles > Preview each rule > Apply each rule. With 10 rules, that's 20 clicks per import
3. **No cumulative preview** — each rule previewed in isolation, no way to see combined pipeline effect
4. **Rule ordering invisible** — rules applied sequentially in `suggest_title_cleanups()` but UI shows no order and offers no reorder
5. **Contradictory behavior** — help text says "manual only" but `bulk_save_expenses` auto-applies via `suggest_title_cleanups()`
6. **No undo** — `display_title` overwritten with no revert path exposed in UI

**Solution:** Replace the entire system with a single find-and-replace step in the bulk upload flow. Users clean titles inline, right where they see the messy data. Recent find-replace pairs are persisted in the existing `config` table for quick re-use on future imports.

## Current State

### Title Cleanup Rules — DB Schema (`crates/core/src/db.rs`, lines 91-96)

```sql
CREATE TABLE IF NOT EXISTS title_cleanup_rules (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    pattern TEXT NOT NULL,
    replacement TEXT NOT NULL DEFAULT '',
    is_regex INTEGER NOT NULL DEFAULT 0
);
-- Plus unique index at line 203-206
```

### Title Cleanup DB Functions (`crates/core/src/db.rs`)

- `insert_title_cleanup_rule()` (line 706)
- `get_all_title_cleanup_rules()` (line 714)
- `update_title_cleanup_rule()` (line 729)
- `delete_title_cleanup_rule()` (line 746)
- `get_title_cleanup_rule()` (line 760)
- `build_cleanup_regex()` (line 782) — private helper
- `normalize_whitespace()` (line 793) — private helper, **keep this**
- `preview_title_cleanup()` (line 799)
- `apply_title_cleanup()` (line 824)
- `suggest_title_cleanups()` (line 878) — called during bulk save

### Title Cleanup IPC Commands (`src-tauri/src/lib.rs`)

- `get_title_cleanup_rules` (line 563)
- `save_title_cleanup_rule` (line 571)
- `delete_title_cleanup_rule` (line 589)
- `preview_title_cleanup` (line 597)
- `apply_title_cleanup` (line 607)
- `TitleCleanupPreview` struct (line 72)

### Title Cleanup Frontend

- `src/lib/TitleCleanup.svelte` (434 lines) — entire component deleted
- `src/lib/ExpenseList.svelte` (lines 168-169) — sub-view routing for cleanup
- `src/lib/ExpenseList.svelte` (lines 198-204) — "Clean Titles" toolbar button

### Bulk Save Integration (`src-tauri/src/lib.rs`, lines 447-451)

```rust
let titles: Vec<String> = expenses.iter().map(|e| e.title.clone()).collect();
let suggestions = db.suggest_title_cleanups(&titles)?;
```

This auto-applies cleanup rules to `display_title` during every bulk save.

### Backup/Restore (`crates/core/src/backup.rs`)

- `BackupTitleCleanupRule` struct (line 53-58)
- `BackupData.title_cleanup_rules` field (line 30)
- `RestoreSummary.cleanup_rules_upserted` field (line 78)
- Backup creation reads cleanup rules (line 90, lines 134-141)
- Restore upserts cleanup rules (`db.rs` lines 1287-1294)
- Test data includes cleanup rules (backup.rs lines 252-256)

### Models (`crates/core/src/models.rs`)

- `TitleCleanupRule` struct (lines 105-111)

### Bulk Upload Flow (`src/lib/BulkUpload.svelte`)

Current steps: `input` → `column-mapping` → `review` → `done`

Progress bar defined at lines 129-148. Step routing at lines 156-176.

### Config Table (`crates/core/src/db.rs`)

Already exists with `get_config(key)` (line 928) and `set_config(key, value)` (line 941). Stores key-value pairs as strings. Used for LLM settings and widget state. Will store recent find-replace pairs as JSON.

## Scope

### 1. Add "Cleanup" step to bulk upload flow (`src/lib/BulkUpload.svelte`)

Insert a new step between `column-mapping` and `review`:

- Update step sequence: `input` → `column-mapping` → `cleanup` → `review` → `done`
- Update progress bar labels: "1. Input", "2. Columns", "3. Cleanup", "4. Review", "5. Done"
- After `handleMapping()` resolves with classified rows, go to `cleanup` instead of `review`
- The cleanup step receives the `classifiedRows` array and modifies titles in place
- A `handleCleanupDone()` callback advances to `review`
- Pass `display_title` values through to the review step (each row gets a `display_title` field set during cleanup, or `null` if unchanged)

### 2. Create `TitleCleanupStep.svelte` (`src/lib/bulk-upload/TitleCleanupStep.svelte`)

New component for the cleanup step. Layout:

**Top: Find-and-replace bar**
- Find input (text, monospace)
- Replace input (text, monospace, placeholder "Leave empty to remove")
- Regex checkbox (opt-in, no explanations — if you check it, you know what you're doing)
- "Apply" button
- Match counter after each apply: "Replaced in N titles"

**Middle: Recent cleanups list**
- Loaded from config on mount via `invoke("get_config", { key: "recent_title_cleanups" })`
- Each entry shows find → replace (or "remove") as a clickable row
- Clicking populates the find/replace fields — user still hits Apply manually
- "Re-apply all" button that runs all recent pairs in sequence on current titles
- Collapsed by default if empty, shown as a small section below the find bar

**Bottom: Expense table**
- Columns: date, title (editable display), amount
- Titles update in place after each Apply
- Rows where title differs from original get a subtle highlight (e.g. emerald tint)
- Tooltip or small icon on modified rows showing original title

**Footer: Navigation**
- "Back" button → returns to column-mapping
- "Next" / "Skip" button → proceeds to review
- "Reset" button → reverts all titles to originals

**State management:**
- Keep a copy of original titles (from `classifiedRows`) for reset and diff display
- Track all applied find-replace operations as an ordered list
- On "Next", write the applied operations to the row data as `display_title` values

### 3. Persist recent find-replace pairs (`src-tauri/src/lib.rs` + `crates/core/src/db.rs`)

No new IPC commands needed — reuse existing `get_config` / `save_config` commands (already registered and working).

Config key: `"recent_title_cleanups"`
Value: JSON array of `{ find, replace, is_regex }` objects

```json
[
  { "find": "PLATNOSC KARTA", "replace": "", "is_regex": false },
  { "find": "CARD \\*\\d{4}", "replace": "", "is_regex": true }
]
```

- Saved when user clicks "Next" (completes cleanup step), not on each Apply
- Cap at 20 entries, deduplicated by (find, replace, is_regex), newest first
- Saving logic lives in the frontend — read JSON from config, prepend new entries, dedup, truncate, write back

### 4. Remove `suggest_title_cleanups()` from bulk save path (`src-tauri/src/lib.rs`)

In `bulk_save_expenses` (lines 447-451), remove the call to `db.suggest_title_cleanups()`. Instead, the `BulkSaveExpense` struct should accept an optional `display_title` field that the frontend sets during the cleanup step.

```rust
// BulkSaveExpense gains:
pub display_title: Option<String>,
```

The `handleSave()` in `BulkUpload.svelte` passes `display_title` from each row (set during cleanup step, `null` if untouched).

### 5. Delete `TitleCleanup.svelte` and all references

- Delete `src/lib/TitleCleanup.svelte`
- `src/lib/ExpenseList.svelte`: remove "Clean Titles" button (lines 198-204), remove `subView === "cleanup"` branch (lines 168-169), remove the import

### 6. Delete title cleanup IPC commands (`src-tauri/src/lib.rs`)

Remove these 5 commands and their registrations in `invoke_handler!`:
- `get_title_cleanup_rules`
- `save_title_cleanup_rule`
- `delete_title_cleanup_rule`
- `preview_title_cleanup`
- `apply_title_cleanup`

Remove the `TitleCleanupPreview` struct.

### 7. Delete title cleanup DB functions (`crates/core/src/db.rs`)

Remove:
- `insert_title_cleanup_rule()`
- `get_all_title_cleanup_rules()`
- `update_title_cleanup_rule()`
- `delete_title_cleanup_rule()`
- `get_title_cleanup_rule()`
- `build_cleanup_regex()` — move `normalize_whitespace()` out of this section, keep it as a pub utility
- `preview_title_cleanup()`
- `apply_title_cleanup()`
- `suggest_title_cleanups()`

Remove all related unit tests (lines ~1698-1740, ~2892-2915).

### 8. Drop `title_cleanup_rules` table (`crates/core/src/db.rs`)

Add `DROP TABLE IF EXISTS title_cleanup_rules` to the `migrate()` function. Remove the `CREATE TABLE` and `CREATE UNIQUE INDEX` statements.

### 9. Delete `TitleCleanupRule` model (`crates/core/src/models.rs`)

Remove the `TitleCleanupRule` struct (lines 105-111).

### 10. Clean up backup/restore (`crates/core/src/backup.rs`)

- Remove `BackupTitleCleanupRule` struct (lines 53-58)
- Remove `title_cleanup_rules` field from `BackupData` (line 30)
- Remove `cleanup_rules_upserted` from `RestoreSummary` (line 78)
- Remove cleanup rules from `create_backup()` (line 90, lines 134-141)
- Remove cleanup rules from restore logic in `db.rs` (lines 1287-1294)
- Update backup log messages (lines 145-151, 195-203)
- Update test `sample_backup()` — remove cleanup rule data (lines 252-256)
- Update test assertions that check `cleanup_rules_upserted` (line 284) and `title_cleanup_rules.len()` (line 293)
- Bump `CURRENT_VERSION` to 2, handle version 1 backups by ignoring their `title_cleanup_rules` field (use `#[serde(default)]` on `BackupData`)

### 11. Clean up integration tests (`crates/core/tests/integration.rs`)

Remove tests related to title cleanup rules (e.g. `title_cleanup_enables_reclassification`). Remove any test helpers that create cleanup rules.

### 12. Update CLAUDE.md

- Remove `title_cleanup_rules` from the Tables list
- Remove title cleanup IPC commands from the command list (reduces count by 5)
- Update bulk upload flow description to include the cleanup step

## Files to Change

| File | Change |
|---|---|
| `src/lib/BulkUpload.svelte` | Add `cleanup` step, update progress bar (5 steps), wire new component |
| `src/lib/bulk-upload/TitleCleanupStep.svelte` | **New file** — find-and-replace UI with recent pairs, expense table |
| `src/lib/TitleCleanup.svelte` | **Delete** |
| `src/lib/ExpenseList.svelte` | Remove "Clean Titles" button, cleanup sub-view routing, import |
| `src-tauri/src/lib.rs` | Remove 5 IPC commands + `TitleCleanupPreview`, remove `suggest_title_cleanups()` call from `bulk_save_expenses`, add `display_title` to `BulkSaveExpense`, update `invoke_handler!` |
| `crates/core/src/db.rs` | Remove 9 functions, drop table in `migrate()`, remove restore logic, keep `normalize_whitespace()` |
| `crates/core/src/models.rs` | Remove `TitleCleanupRule` struct |
| `crates/core/src/backup.rs` | Remove `BackupTitleCleanupRule`, clean up `BackupData`, `RestoreSummary`, backup/restore logic, update tests |
| `crates/core/tests/integration.rs` | Remove title cleanup tests |
| `CLAUDE.md` | Update tables, IPC commands, bulk upload flow |

## Test Scenarios

### Backend (Rust unit tests)

1. `normalize_whitespace(" hello   world ")` returns `"hello world"` — verify it's still accessible after refactor
2. `bulk_save_expenses` with `display_title: Some("Clean Title")` persists the display title to the expense
3. `bulk_save_expenses` with `display_title: None` leaves `display_title` as `None` in the DB
4. Backup v1 (with `title_cleanup_rules` field) restores successfully — field is ignored, no error
5. Backup v2 (without `title_cleanup_rules`) roundtrips correctly
6. `migrate()` drops `title_cleanup_rules` table if it exists from a previous version

### Frontend (manual UI tests)

1. Bulk upload a CSV → after column mapping, lands on "3. Cleanup" step (not directly on Review)
2. Type "PLATNOSC KARTA" in find, leave replace empty, click Apply → all matching titles have that string removed, counter shows "Replaced in N titles"
3. Apply a second find-replace → titles show cumulative effect of both operations
4. Click "Reset" → all titles revert to their original parsed state
5. Click "Skip"/"Next" without applying anything → proceeds to Review with original titles unchanged
6. Complete a full upload with 2 find-replace operations → navigate away and start a new upload → "Recent" section shows the 2 pairs from last time
7. Click a recent pair → find/replace fields populate, click Apply → works as expected
8. Click "Re-apply all" in Recent → all recent pairs run in sequence on current titles
9. Check "Regex" box, type `CARD \*\d{4}`, apply → matches variable card numbers like "CARD *1234", "CARD *5678"
10. ExpenseList page no longer shows "Clean Titles" button
11. Backup/restore works without title cleanup rules — no errors, no missing data

## Acceptance Criteria

- Bulk upload flow has 5 steps: Input → Columns → Cleanup → Review → Done
- Find-and-replace works on expense titles in the cleanup step (literal and regex)
- Multiple find-replace operations accumulate — table shows the running result
- "Reset" reverts all titles to their original parsed state
- "Skip"/"Next" proceeds to review without requiring any cleanup
- Recent find-replace pairs persist across upload sessions via the config table
- Clicking a recent pair populates the fields; "Re-apply all" runs all recent pairs
- Recent pairs capped at 20, deduplicated, newest first
- `display_title` values from cleanup step are saved to the DB on bulk save
- The `title_cleanup_rules` DB table is dropped on migration
- All 5 title cleanup IPC commands are removed
- `TitleCleanup.svelte` is deleted
- "Clean Titles" button is removed from ExpenseList
- Backup/restore handles both v1 (with cleanup rules) and v2 (without) gracefully
- No regression in classification rules (category matching) — completely untouched
- Whitespace is normalized after each replacement
- UI follows dark theme conventions (gray-950/900/800, emerald accents)
- Svelte 5 syntax throughout new component
