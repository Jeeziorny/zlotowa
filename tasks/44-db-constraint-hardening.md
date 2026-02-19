# 44 — DB Constraint Hardening

## Problem

Several data integrity guarantees are enforced only at the application level (TOCTOU race conditions). The database schema does not enforce them.

### Missing UNIQUE constraints

1. **`crates/core/src/db.rs:74`** — `idx_expenses_dup` on `(title, amount, date)` is a regular index, NOT UNIQUE. Two rapid `insert_expense` calls with the same data will both succeed. The duplicate check is SELECT-then-INSERT with no constraint.

2. **`crates/core/src/db.rs:160-163`** — `budgets_v2` has NO uniqueness constraint. Budget overlap is checked at the application level (`check_budget_overlap`), but concurrent `create_budget` calls can create overlapping budgets.

3. **`crates/core/src/db.rs:89-94`** — `title_cleanup_rules` has no UNIQUE constraint on `(pattern, replacement, is_regex)`. Duplicate rules can be inserted.

### Missing ON DELETE behavior

4. **`crates/core/src/db.rs:146`** — `batch_id INTEGER REFERENCES upload_batches(id)` on expenses has no `ON DELETE CASCADE` or `ON DELETE SET NULL`. The `delete_batch` method manually deletes expenses first, but a direct DELETE on `upload_batches` leaves orphaned `batch_id` values. Contrast with budget FKs which all have `ON DELETE CASCADE`.

### Missing FK indices

5. `planned_expenses`, `calendar_events`, `budget_categories` all filter by `budget_id` with no index. SQLite does full table scans. Also relevant for ON DELETE CASCADE performance.

### Floating-point money

6. **`crates/core/src/db.rs:67`** — `amount REAL NOT NULL` uses `f64` for monetary values. `db.rs:465` compares `f64` with `==` for duplicates, which fails on precision drift. Budget summaries accumulate rounding errors. (NOTE: migrating to INTEGER cents is a large change — may be deferred.)

### Schema validation

7. **`crates/core/src/db.rs:68`** — `date TEXT NOT NULL` has no CHECK constraint for format. Corrupt date values would cause parse failures at read time, not write time.

## Scope

- Add UNIQUE constraint on `expenses(title, amount, date)` — use `INSERT OR IGNORE` or `ON CONFLICT` where appropriate
- Add `ON DELETE SET NULL` or `ON DELETE CASCADE` to `batch_id` FK on expenses
- Add indices on `budget_id` for child tables
- Add UNIQUE constraint on `title_cleanup_rules(pattern, replacement, is_regex)`
- Consider CHECK constraint on date format
- Log decision on float-vs-integer money (likely deferred to a separate task)
