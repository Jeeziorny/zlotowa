# Task 18: Batch Tracking & Undo for Bulk Uploads

**Track:** B — Data Safety (backend + frontend)
**Blocked by:** nothing
**Blocks:** nothing

## Problem

There is no way to revert a bulk upload. If a user uploads the wrong CSV or maps columns incorrectly, the only recovery is manually deleting the SQLite database. There is no expense-deletion functionality at all — no DB method, no IPC command, no UI control.

Example scenario: user uploads `bank-2024.csv` (200 expenses), realises they mapped the amount column wrong, and every expense has the wrong value. They're stuck.

## Current State

### Database layer (`crates/core/src/db.rs`)

**Migration** (lines 57-87): Creates `expenses`, `classification_rules`, `config` tables. No batch or upload tracking.

**`insert_expenses_bulk`** (lines 113-140): Takes `&[Expense]`, wraps all inserts in a single `unchecked_transaction()`. INSERT names 5 columns explicitly (`title, amount, date, category, classification_source`). Returns count.

```rust
pub fn insert_expenses_bulk(&self, expenses: &[Expense]) -> Result<usize, DbError>
```

**`insert_expense`** (lines 91-110): Single-expense insert, same 5 named columns. Used by `add_expense` IPC for manual entry.

**No delete methods** exist for expenses. Only `delete_category` (reassigns expenses to a replacement category).

### Tauri IPC (`src-tauri/src/lib.rs`)

**`bulk_save_expenses`** (lines 345-393): Validates expenses, builds `Vec<Expense>`, calls `db.insert_expenses_bulk(&to_insert)` (line 386), then best-effort `insert_rules_bulk`. No filename or batch metadata is captured.

```rust
fn bulk_save_expenses(
    state: State<AppState>,
    expenses: Vec<BulkSaveExpense>,
) -> Result<usize, String>
```

**Handler registration** (lines 478-498): All commands listed in `tauri::generate_handler![]`.

**Import** (line 4): `use accountant_core::models::{CategoryStats, ClassificationRule, ClassificationSource, Expense, ParsedExpense};`

### Frontend (`src/lib/BulkUpload.svelte`)

**`saveApproved()`** (lines 253-270): Calls `invoke("bulk_save_expenses", { expenses: toSave })`. No filename passed. The `file` state variable (line 9) holds the `File` object from drag-drop but is never sent to the backend.

### CLI (`crates/cli/src/main.rs`)

**`cmd_bulk_insert`** (lines 215-489): Has the file `path: PathBuf` available. Calls `db.insert_expenses_bulk(&to_insert)` at line 479 without any batch tracking.

### Models (`crates/core/src/models.rs`)

`Expense` struct has no `batch_id` field. No `UploadBatch` type exists.

### Existing tests (`crates/core/src/db.rs`)

- `bulk_insert_expenses` (line 534): calls `db.insert_expenses_bulk(&expenses)` — will need the new second arg.
- `bulk_insert_rolls_back_on_nan` (line 547): calls `db.insert_expenses_bulk(&expenses)` — same.

## Scope

### 1. Add `UploadBatch` struct (`crates/core/src/models.rs`)

Add after `CategoryStats` (line 89):

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadBatch {
    pub id: i64,
    pub filename: Option<String>,
    pub uploaded_at: String,   // ISO 8601 datetime
    pub expense_count: i64,
}
```

### 2. Add `upload_batches` table and `batch_id` column (`crates/core/src/db.rs`, `migrate()`)

After the existing `execute_batch` (line 85), append:

- `CREATE TABLE IF NOT EXISTS upload_batches (id INTEGER PRIMARY KEY AUTOINCREMENT, filename TEXT, uploaded_at TEXT NOT NULL, expense_count INTEGER NOT NULL)`
- `ALTER TABLE expenses ADD COLUMN batch_id INTEGER REFERENCES upload_batches(id)` — wrap with `let _ =` to ignore "duplicate column name" error on subsequent opens (SQLite has no `ADD COLUMN IF NOT EXISTS`)
- `CREATE INDEX IF NOT EXISTS idx_expenses_batch_id ON expenses(batch_id)`

Note: `insert_expense` (line 98) names columns explicitly, so the new `batch_id` column defaults to `NULL` without any code change there.

### 3. Modify `insert_expenses_bulk` signature (`crates/core/src/db.rs`)

Change to:

```rust
pub fn insert_expenses_bulk(
    &self,
    expenses: &[Expense],
    batch_filename: Option<&str>,
) -> Result<usize, DbError>
```

When `batch_filename` is `Some`:
- Insert into `upload_batches` (filename, `Utc::now().to_rfc3339()`, expense count) inside the same transaction
- Capture `last_insert_rowid()` as `batch_id`
- Include `batch_id` in each expense INSERT (6th column)

When `None`: no batch record, `batch_id = NULL`.

### 4. Add `get_upload_batches` method (`crates/core/src/db.rs`)

```rust
pub fn get_upload_batches(&self) -> Result<Vec<UploadBatch>, DbError>
```

Query: `SELECT id, filename, uploaded_at, expense_count FROM upload_batches ORDER BY uploaded_at DESC`

### 5. Add `delete_batch` method (`crates/core/src/db.rs`)

```rust
pub fn delete_batch(&self, batch_id: i64) -> Result<usize, DbError>
```

In one transaction: `DELETE FROM expenses WHERE batch_id = ?`, then `DELETE FROM upload_batches WHERE id = ?`. Return count of deleted expenses.

Classification rules are intentionally NOT deleted — they represent learned patterns and should persist independently.

### 6. Update existing tests (`crates/core/src/db.rs`)

- `bulk_insert_expenses` (line 541): change to `insert_expenses_bulk(&expenses, None)`
- `bulk_insert_rolls_back_on_nan` (line 553): change to `insert_expenses_bulk(&expenses).is_err()` → `insert_expenses_bulk(&expenses, None).is_err()`

### 7. Add Tauri IPC commands (`src-tauri/src/lib.rs`)

- Add `filename: Option<String>` param to `bulk_save_expenses` (line 348). Pass `filename.as_deref()` to `insert_expenses_bulk` (line 386). Tauri deserializes absent JS params as `None` for `Option<T>`.
- Add `get_upload_batches` command: lock DB, call `db.get_upload_batches()`, return `Vec<UploadBatch>`.
- Add `delete_batch` command: lock DB, call `db.delete_batch(batch_id)`, return deleted count.
- Register both in `invoke_handler![]` (line 478).
- Add `UploadBatch` to the import (line 4).

### 8. Wire filename from frontend (`src/lib/BulkUpload.svelte`)

- Add derived: `let batchFilename = $derived(file ? file.name : "Pasted data");`
- In `saveApproved()` (line 265), add `filename: batchFilename` to the invoke params.

### 9. Add Upload History UI (`src/lib/Settings.svelte`)

Add a new card section below the LLM config (after line 209), inside the existing `max-w-lg space-y-6` wrapper:

- Load batches via `invoke("get_upload_batches")` in the existing `onMount`.
- Render each batch as a row: filename, formatted date, expense count, "Undo" button.
- Two-click delete confirmation inline: first click shows "Delete N expenses? [Confirm] [Cancel]", second click executes.
- On confirm: call `invoke("delete_batch", { batchId })`, remove batch from local `$state` array, show success message.
- Style: match existing gray-900 card pattern with gray-800 borders and emerald accents.

### 10. Wire filename from CLI (`crates/cli/src/main.rs`)

At line 479, extract filename from `path` and pass it:

```rust
let filename = path.file_name().map(|n| n.to_string_lossy().to_string());
match db.insert_expenses_bulk(&to_insert, filename.as_deref()) {
```

## Files to Change

| File | Change |
|---|---|
| `crates/core/src/models.rs` | Add `UploadBatch` struct |
| `crates/core/src/db.rs` | Migration (new table + column), modify `insert_expenses_bulk` signature, add `get_upload_batches` + `delete_batch`, update 2 existing tests |
| `src-tauri/src/lib.rs` | Add `filename` to `bulk_save_expenses`, add 2 new IPC commands, register in handler, add import |
| `src/lib/BulkUpload.svelte` | Derive `batchFilename`, pass to `bulk_save_expenses` invoke |
| `src/lib/Settings.svelte` | Add Upload History section with batch list and undo UI |
| `crates/cli/src/main.rs` | Pass filename to `insert_expenses_bulk` |

## Test Scenarios

### Backend (Rust unit tests)

1. **Bulk insert with batch tracking**: Insert 3 expenses with `batch_filename: Some("test.csv")`. Verify `get_upload_batches()` returns 1 batch with `filename = "test.csv"`, `expense_count = 3`.
2. **Bulk insert without batch**: Insert 2 expenses with `batch_filename: None`. Verify `get_upload_batches()` returns empty, `get_all_expenses()` returns 2.
3. **Delete batch removes expenses and record**: Insert batch of 3 expenses. Call `delete_batch(batch_id)`. Verify returns `3`, `get_all_expenses()` is empty, `get_upload_batches()` is empty.
4. **Delete batch doesn't affect other expenses**: Insert batch A (2 expenses) and batch B (1 expense). Delete batch A. Verify batch B's expense remains, only batch B in `get_upload_batches()`.
5. **Existing tests pass**: `bulk_insert_expenses` and `bulk_insert_rolls_back_on_nan` still pass after signature change (with `None`).

### Frontend (manual UI tests)

1. Upload a CSV file via BulkUpload drag-drop. Go to Settings. Verify "Upload History" shows the filename, upload date, and correct expense count.
2. Paste CSV text (no file) into BulkUpload. Go to Settings. Verify upload shows as "Pasted data".
3. Click "Undo" on a batch. Verify confirmation appears ("Delete N expenses?"). Click "Cancel". Verify nothing changes.
4. Click "Undo" then "Confirm". Verify success message appears, batch disappears from list, and ExpenseList no longer shows those expenses.
5. Upload two different CSVs. Undo only the second one. Verify the first upload's expenses remain intact.
6. With no uploads, verify Settings shows "No bulk uploads yet." message.

## Acceptance Criteria

- `upload_batches` table is created on app start (migration runs idempotently)
- Existing databases gain `batch_id` column on `expenses` without data loss (existing rows get `NULL`)
- Every bulk upload (GUI and CLI) creates a batch record with filename, timestamp, and expense count
- Manually added expenses (`add_expense`) have `batch_id = NULL` and are unaffected by batch operations
- Users can view all past uploads in Settings with filename, date, and count
- Users can undo any specific upload, permanently deleting its expenses
- Undo requires two clicks (confirmation step) before deleting
- Classification rules are preserved when a batch is undone
- `cargo test` passes (all existing + new tests)
- `cargo build --workspace` compiles without warnings
