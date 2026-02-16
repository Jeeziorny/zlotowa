# Task 21: Title Cleanup Rules

**Track:** Full-stack — New feature (backend + frontend)
**Priority:** HIGH
**Blocked by:** Task 20 (needs `update_expense` for applying changes)
**Blocks:** nothing

## Problem

Bank transaction titles are full of noise — card numbers, payment processor names, reference codes, station IDs. Examples:

- `OP. MC 557519******2036 PŁATNOŚĆ KARTĄ 71.24 PLN ORLEN STACJA NR 983 Wroclaw` — the useful part is just "ORLEN STACJA NR 983 Wroclaw"
- `Zakup BLIK PayPro S.A. Pastelowa 860-198 Poznan ref:93115592795` — the useful part is "PayPro S.A." or maybe just the merchant name

Users need a way to define find/replace rules that strip or transform these noisy fragments. The rules should be:

1. **Definable** — user says "if title contains X, remove it" or "replace X with Y"
2. **Previewable** — before applying, user can see which expenses would be affected and what the result looks like
3. **Selectively applicable** — user can choose to apply to all matching expenses or pick specific ones

## Current State

### Database (`crates/core/src/db.rs`)

- No title transformation tables or methods
- `update_expense()` will exist after Task 20 — needed to persist title changes
- `get_all_expenses()` (line 142) returns all expenses — can be used to find matches

### Models (`crates/core/src/models.rs`)

- `Expense` struct (line 6-14) — `title: String` is the field to transform
- No `TitleCleanupRule` type exists

### Tauri IPC (`src-tauri/src/lib.rs`)

- No title cleanup commands

### Frontend

- No title cleanup UI anywhere

## Scope

### 1. Add `TitleCleanupRule` struct (`crates/core/src/models.rs`)

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TitleCleanupRule {
    pub id: Option<i64>,
    pub pattern: String,         // regex pattern to find
    pub replacement: String,     // replacement string (empty = delete the match)
    pub is_regex: bool,          // true = pattern is regex, false = literal string match
}
```

### 2. Add `title_cleanup_rules` table (`crates/core/src/db.rs`, `migrate()`)

Append to the existing `execute_batch`:

```sql
CREATE TABLE IF NOT EXISTS title_cleanup_rules (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    pattern     TEXT NOT NULL,
    replacement TEXT NOT NULL DEFAULT '',
    is_regex    INTEGER NOT NULL DEFAULT 0
);
```

### 3. Add DB methods (`crates/core/src/db.rs`)

```rust
pub fn insert_title_cleanup_rule(&self, rule: &TitleCleanupRule) -> Result<i64, DbError>
pub fn get_all_title_cleanup_rules(&self) -> Result<Vec<TitleCleanupRule>, DbError>
pub fn delete_title_cleanup_rule(&self, id: i64) -> Result<(), DbError>
pub fn update_title_cleanup_rule(&self, rule: &TitleCleanupRule) -> Result<(), DbError>
```

Also add a method that applies a rule to specific expenses:

```rust
/// Apply a title cleanup rule to specific expenses. Returns count of updated rows.
pub fn apply_title_cleanup(&self, rule: &TitleCleanupRule, expense_ids: &[i64]) -> Result<usize, DbError>
```

Implementation:
- Build a `regex::Regex` from `rule.pattern` (or `regex::escape()` it if `is_regex == false`)
- Start a transaction
- For each expense id: fetch current title, apply `regex.replace_all(title, &rule.replacement)`, trim whitespace, collapse multiple spaces, update if changed
- Commit, return count of actually-modified rows

### 4. Add a preview method (`crates/core/src/db.rs`)

```rust
/// Preview what a cleanup rule would do to all expenses. Returns (expense_id, original_title, cleaned_title) for affected rows.
pub fn preview_title_cleanup(&self, rule: &TitleCleanupRule) -> Result<Vec<(i64, String, String)>, DbError>
```

- Fetches all expenses, applies the regex in-memory (no DB writes), returns only rows where the title actually changes

### 5. Add Tauri IPC commands (`src-tauri/src/lib.rs`)

```rust
#[tauri::command]
fn get_title_cleanup_rules(state: State<AppState>) -> Result<Vec<TitleCleanupRule>, String>

#[tauri::command]
fn save_title_cleanup_rule(state: State<AppState>, rule: TitleCleanupRule) -> Result<i64, String>
// If rule.id is Some → update, if None → insert. Returns rule id.

#[tauri::command]
fn delete_title_cleanup_rule(state: State<AppState>, id: i64) -> Result<(), String>

#[tauri::command]
fn preview_title_cleanup(state: State<AppState>, rule: TitleCleanupRule) -> Result<Vec<TitleCleanupPreview>, String>
// TitleCleanupPreview = { expense_id: i64, original: String, cleaned: String }

#[tauri::command]
fn apply_title_cleanup(state: State<AppState>, rule_id: i64, expense_ids: Vec<i64>) -> Result<usize, String>
// Looks up the rule by id, applies to selected expense_ids
```

Add `TitleCleanupPreview` struct in `src-tauri/src/lib.rs`:

```rust
#[derive(Serialize, Deserialize)]
pub struct TitleCleanupPreview {
    pub expense_id: i64,
    pub original: String,
    pub cleaned: String,
}
```

Register all in `invoke_handler![]`.

### 6. Add "Title Cleanup" to sidebar and routing

- `Sidebar.svelte`: add `{ id: "cleanup", label: "Title Cleanup", icon: "✂" }` to items array
- `App.svelte`: add import + route for `TitleCleanup.svelte`

### 7. Create Title Cleanup page (`src/lib/TitleCleanup.svelte`)

**Layout — two sections:**

**Section A: Rules list**
- Table of existing rules: Pattern, Replacement (or "Remove"), Type (Regex/Literal), Actions
- "Add Rule" button opens an inline form:
  - Pattern input (text)
  - Replacement input (text, empty = remove the match)
  - Toggle: Literal string / Regex
  - Save / Cancel
- Edit button per rule (inline editing, same as add form)
- Delete button per rule with confirmation

**Section B: Preview & Apply (shown when a rule is selected)**
- Click "Preview" on a rule → calls `preview_title_cleanup` → shows a table:
  - Checkbox per row (for selective apply)
  - Original title (with matched portion highlighted, e.g. red strikethrough or background)
  - Arrow →
  - Cleaned title (with changes highlighted, e.g. green background)
- "Select All" / "Deselect All" buttons
- "Apply to N selected" button → calls `apply_title_cleanup` with selected expense IDs → shows success count → refreshes preview (should now be empty or reduced)

**UX flow:**
1. User adds a rule like pattern=`OP\. MC \d{6}\*{6}\d{4} PŁATNOŚĆ KARTĄ [\d.]+ PLN ` replacement=`` (empty)
2. Clicks Preview — sees 47 expenses that would be affected, with before/after
3. Reviews the preview, unchecks 2 that look wrong
4. Clicks "Apply to 45 selected" — titles are cleaned in the DB
5. Rule stays saved for future imports

## Files to Change

| File | Change |
|---|---|
| `crates/core/src/models.rs` | Add `TitleCleanupRule` struct |
| `crates/core/src/db.rs` | Migration (new table), add CRUD methods + `preview_title_cleanup` + `apply_title_cleanup` |
| `src-tauri/src/lib.rs` | Add `TitleCleanupPreview` struct, 5 new IPC commands, register in `invoke_handler!` |
| `src/lib/Sidebar.svelte` | Add "Title Cleanup" nav item |
| `src/App.svelte` | Add import and route for `TitleCleanup.svelte` |
| `src/lib/TitleCleanup.svelte` | **New file** — rule management + preview/apply UI |

## Test Scenarios

### Backend (Rust unit tests)

1. Insert a literal rule, preview against expenses with matching titles — returns correct before/after pairs
2. Insert a regex rule with capture groups, preview shows correct replacements
3. `apply_title_cleanup` updates only the specified expense IDs, not others that also match
4. `apply_title_cleanup` returns 0 when no titles actually change (pattern doesn't match selected IDs)
5. Literal rule with special regex chars (`.`, `*`, `(`) is auto-escaped and matches literally
6. Cleanup collapses multiple spaces and trims whitespace after replacement
7. CRUD: insert, get_all, update, delete rules all work correctly
8. Empty replacement string removes the matched portion entirely

### Frontend (manual UI tests)

1. Navigate to Title Cleanup — empty state shows "No cleanup rules" message
2. Add a literal rule with pattern "PŁATNOŚĆ KARTĄ" replacement "" — rule appears in list
3. Click Preview — matching expenses shown with highlighted diff
4. Select all, click Apply — success message, preview now empty
5. Go to Expenses page — affected titles are cleaned
6. Add a regex rule — toggle shows "Regex" badge, preview works with regex matching
7. Edit an existing rule — changes persist after save
8. Delete a rule — confirmation, rule removed from list

## Acceptance Criteria

- Users can create literal or regex find/replace rules for expense titles
- Rules can be previewed against all expenses showing before/after diffs
- Users can selectively apply rules to chosen expenses (not forced to apply to all)
- Applied changes are permanent (titles updated in DB)
- Rules persist for future use (reusable on new imports)
- Multiple spaces collapsed and whitespace trimmed after replacement
- Regex errors are caught and shown to the user (invalid patterns don't crash)
- UI follows dark theme conventions
- Svelte 5 syntax used throughout
