# Task 20: Expense Deletion & Editing

**Track:** Full-stack — CRUD completion (backend + frontend)
**Priority:** HIGH
**Blocked by:** nothing
**Blocks:** Task 21 (title cleanup rules reuse `update_expense`)

## Problem

After an expense is saved there is no way to edit or delete it. The only "undo" is deleting the entire SQLite database. If a user types the wrong amount, misspells a title, or assigns the wrong category, the expense is permanently frozen.

The expense list (`ExpenseList.svelte`) is read-only — a static table with no action buttons. The database layer has no `update_expense` or `delete_expense` methods. The Tauri IPC layer has no corresponding commands.

## Current State

### Database (`crates/core/src/db.rs`)

- `insert_expense()` (line 91) — single insert, returns `i64` ID
- `insert_expenses_bulk()` (line 113) — batch insert
- `get_all_expenses()` (line 142) — `SELECT ... ORDER BY date DESC`, returns all rows
- **No** `update_expense()`, `delete_expense()`, or `delete_expenses_batch()` methods

Schema (line 60-67):
```sql
CREATE TABLE expenses (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    title           TEXT NOT NULL,
    amount          REAL NOT NULL,
    date            TEXT NOT NULL,
    category        TEXT,
    classification_source TEXT
);
```

### Tauri IPC (`src-tauri/src/lib.rs`)

- `get_expenses` (line 86) — returns `Vec<Expense>`
- `add_expense` (line 92) — inserts one expense
- **No** `update_expense`, `delete_expense`, or `delete_expenses` commands

### Frontend (`src/lib/ExpenseList.svelte`)

- Read-only table: Date, Title, Amount, Category, Source columns (lines 156-192)
- No edit/delete buttons, no inline editing, no row selection
- Exports CSV but cannot modify individual rows

### Models (`crates/core/src/models.rs`)

- `Expense` struct (line 6-14): `{ id: Option<i64>, title, amount, date, category, classification_source }`
- No changes needed to the struct — `id` is already `Option<i64>` and can be used for UPDATE/DELETE WHERE

## Scope

### 1. Add `update_expense` method (`crates/core/src/db.rs`)

```rust
pub fn update_expense(&self, expense: &Expense) -> Result<(), DbError> {
    let id = expense.id.ok_or(DbError::InvalidData("Cannot update expense without id".into()))?;
    if !expense.amount.is_finite() {
        return Err(DbError::InvalidData(format!("Amount is not a valid number: {}", expense.amount)));
    }
    let rows = self.conn.execute(
        "UPDATE expenses SET title = ?1, amount = ?2, date = ?3, category = ?4, classification_source = ?5 WHERE id = ?6",
        params![expense.title, expense.amount, expense.date.to_string(), expense.category,
                expense.classification_source.as_ref().map(|s| s.as_db_str()), id],
    )?;
    if rows == 0 {
        return Err(DbError::InvalidData(format!("Expense with id {} not found", id)));
    }
    Ok(())
}
```

### 2. Add `delete_expense` and `delete_expenses` methods (`crates/core/src/db.rs`)

```rust
pub fn delete_expense(&self, id: i64) -> Result<(), DbError> {
    let rows = self.conn.execute("DELETE FROM expenses WHERE id = ?1", params![id])?;
    if rows == 0 {
        return Err(DbError::InvalidData(format!("Expense with id {} not found", id)));
    }
    Ok(())
}

pub fn delete_expenses(&self, ids: &[i64]) -> Result<usize, DbError> {
    if ids.is_empty() { return Ok(0); }
    let tx = self.conn.unchecked_transaction()?;
    let mut count = 0;
    for id in ids {
        count += tx.execute("DELETE FROM expenses WHERE id = ?1", params![id])?;
    }
    tx.commit()?;
    Ok(count)
}
```

### 3. Add Tauri IPC commands (`src-tauri/src/lib.rs`)

```rust
#[tauri::command]
fn update_expense(state: State<AppState>, expense: ExpenseInput, id: i64) -> Result<(), String>

#[tauri::command]
fn delete_expense(state: State<AppState>, id: i64) -> Result<(), String>

#[tauri::command]
fn delete_expenses(state: State<AppState>, ids: Vec<i64>) -> Result<usize, String>
```

- `update_expense`: parse date, build `Expense` with `id: Some(id)`, call `db.update_expense()`. If a new category is provided, also update/insert a classification rule via `save_rule_if_categorized`.
- `delete_expense`: call `db.delete_expense(id)`
- `delete_expenses`: call `db.delete_expenses(&ids)`, return count
- Register all three in `invoke_handler![]` (line 478)

Note: the `update_expense` IPC command name collides with the DB method. Either rename the IPC command to e.g. `update_expense_cmd` and use `#[tauri::command(rename_all = "camelCase")]`, or just keep the same name since Rust scoping handles it.

### 4. Add edit and delete UI to `ExpenseList.svelte`

**Delete (single):**
- Add a trash icon button per row (rightmost column)
- Click shows inline confirmation: "Delete? [Yes] [No]"
- On confirm: `invoke("delete_expense", { id: expense.id })`, remove from local `expenses` array

**Delete (batch):**
- Add checkbox per row + "select all" checkbox in header
- When 1+ rows selected, show "Delete N selected" button above table
- Confirm dialog, then `invoke("delete_expenses", { ids: [...] })`, remove from local array

**Inline edit:**
- Click a row to enter edit mode (or a pencil icon button)
- Title, amount, category become editable inputs; date becomes a date input
- "Save" / "Cancel" buttons appear in the row
- On save: `invoke("update_expense", { id, expense: { title, amount, date, category } })`
- On cancel: restore original values
- Only one row editable at a time

## Files to Change

| File | Change |
|---|---|
| `crates/core/src/db.rs` | Add `update_expense`, `delete_expense`, `delete_expenses` methods |
| `src-tauri/src/lib.rs` | Add 3 new IPC commands, register in `invoke_handler!` |
| `src/lib/ExpenseList.svelte` | Add edit mode, delete buttons, batch selection, confirmation UX |

## Test Scenarios

### Backend (Rust unit tests in `crates/core/src/db.rs`)

1. `update_expense` changes title, amount, date, category for existing expense
2. `update_expense` returns error for non-existent id
3. `update_expense` returns error for expense without id (`None`)
4. `update_expense` rejects NaN/infinity amounts
5. `delete_expense` removes single expense by id
6. `delete_expense` returns error for non-existent id
7. `delete_expenses` removes multiple expenses atomically
8. `delete_expenses` with empty ids returns 0
9. After delete, `get_all_expenses` no longer includes deleted rows

### Frontend (manual UI tests)

1. Click edit on an expense — fields become editable, Save/Cancel appear
2. Edit title + amount, click Save — row updates, edit mode closes
3. Click Cancel during edit — original values restored
4. Click delete on a single expense — confirmation appears, confirm deletes it
5. Select multiple expenses via checkboxes — "Delete N selected" button appears
6. Confirm batch delete — all selected rows disappear
7. After any edit/delete, navigate away and back — changes persist

## Acceptance Criteria

- Expenses can be edited inline (title, amount, date, category)
- Expenses can be deleted individually with confirmation
- Multiple expenses can be selected and deleted in batch
- All database mutations (update, delete) validate input and return errors for invalid data
- UI follows dark theme conventions (gray-950/900/800, emerald accents)
- Svelte 5 syntax used throughout (`$state`, `$derived`, `$props`, `onclick`)
- `cargo test` passes (existing + new tests)
