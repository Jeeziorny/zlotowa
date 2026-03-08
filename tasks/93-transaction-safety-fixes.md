# Task 93: Transaction Safety Fixes

## Goal

Standardize transaction handle usage and fix a minor logic bug in `delete_budget`, ensuring all operations within transaction blocks use the `tx` handle consistently.

## Deliverables

### 1. Standardize `create_budget_with_categories()` — `db.rs`

Changed `self.conn.execute()` → `tx.execute()` and `self.conn.last_insert_rowid()` → `tx.last_insert_rowid()` inside the transaction block. Functionally equivalent with `unchecked_transaction` (same connection), but consistent with the rest of the codebase.

### 2. Refactor `save_budget_categories_inner()` — `db.rs`

Accepts `&Transaction` parameter instead of using `self.conn` directly. Callers (`save_budget_categories`, `create_budget_with_categories`) now pass `&tx`. Removed misleading doc comment ("caller must ensure this runs inside a transaction") — the type system enforces it now.

### 3. Fix `restore_backup_data()` — `db.rs`

Changed `self.conn.last_insert_rowid()` → `tx.last_insert_rowid()` for consistency.

### 4. Fix `delete_budget()` post-commit check — `db.rs`

Moved the `if rows == 0` error check before `tx.commit()` so the transaction rolls back when the budget is not found, instead of committing an empty delete first.

### 5. Tests

- `create_budget_with_duplicate_categories_rolls_back` — duplicate categories trigger UNIQUE constraint violation; verifies budget row is also rolled back.
- `delete_budget_not_found_does_not_commit` — deleting non-existent budget ID returns error without affecting existing budgets.

## Files modified
- `crates/core/src/db.rs`
