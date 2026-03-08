# Task 97: DB Indices & Schema Tightening

## Goal

Add missing indices on columns used in WHERE/ORDER BY/GROUP BY, tighten nullable columns, and consider refactoring the in-memory rule match counting.

## Deliverables

### 1. Add indices — `db.rs` `migrate()`

Add to the migration block:
- `CREATE INDEX IF NOT EXISTS idx_classification_rules_category ON classification_rules(category)`
- `CREATE INDEX IF NOT EXISTS idx_upload_batches_uploaded_at ON upload_batches(uploaded_at)`
- `CREATE INDEX IF NOT EXISTS idx_budget_categories_category ON budget_categories(category)`

### 2. Tighten `upload_batches.filename` — `db.rs` schema

The column is nullable but app logic always provides a filename. Check if changing to `NOT NULL` would break existing data (rows with NULL filename). If no NULL rows exist in practice, add NOT NULL via migration. Otherwise, add a DEFAULT '' and backfill.

### 3. Consider `query_rules()` match counting — `db.rs:766-771`

Currently loads ALL expense titles into memory, then regex-tests every rule. This is O(rules x titles). Options:
- (a) Accept it — if title count stays under a few thousand, it's fine for a desktop app.
- (b) Move to a lazy/on-demand count (only compute when the Rules page is viewed).
- (c) Use SQLite's `REGEXP` function with a custom implementation.

Document the chosen approach in a code comment.

### 4. Tests

- Verify indices exist after migration (query `sqlite_master`).
- Test that NULL filename is rejected after NOT NULL constraint.

## Files to modify
- `crates/core/src/db.rs`

## Notes
- Existing indices: `idx_expenses_batch_id`, `idx_budget_categories_budget_id`, `idx_classification_rules_pattern` (UNIQUE). Don't duplicate.
- The `expenses` table already has indices on `date`, `category`, `batch_id`.
