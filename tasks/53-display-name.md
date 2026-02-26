# 53 — `display_name` Data Model

## Problem

Title cleanup currently overwrites the original `title` field permanently. Users lose the raw bank transaction title, which is needed for classification rules to keep working. Originally requested in task 38.

## Changes

### Database

- **Migration:** Add `display_name TEXT` column to `expenses` table, default to `title` value for existing rows
- **Query updates:** All reads return `display_name`, all writes preserve both fields
- **Duplicate detection:** Still uses `(title, amount, date)` — raw title, not display name

### Model

- Add `display_name: Option<String>` to `Expense` struct
- When `display_name` is `None`, display falls back to `title`

### Title Cleanup

- `apply_title_cleanup()` updates `display_name` instead of `title`
- Raw `title` remains immutable after initial insert

### Classification

- Regex rules still match against `title` (raw bank format), not `display_name`
- Auto-generated rules from manual categorization use raw `title`

### Frontend

- Show `display_name` (falling back to `title`) everywhere expenses are displayed
- ExpenseList, Dashboard widgets, BulkUpload review, export

## Scope

- `crates/core/src/db.rs` — migration, query updates, cleanup logic
- `crates/core/src/models.rs` — add `display_name` field
- `src-tauri/src/lib.rs` — IPC commands pass through new field
- All frontend components that display expense titles
