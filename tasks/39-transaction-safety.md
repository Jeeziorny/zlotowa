# 39 — Transaction Safety for Multi-Write Operations

## Problem

Several Tauri IPC commands perform multiple DB writes without wrapping them in a transaction. If the app crashes or any write fails mid-operation, the database is left inconsistent.

### Locations

1. **`src-tauri/src/lib.rs:102-117`** — `add_expense` calls `db.insert_expense()` then `save_rule_if_categorized()` (which calls `db.insert_rule()`). Not atomic. The `let _ =` on rule insertion silently discards errors.

2. **`src-tauri/src/lib.rs:127-142`** — `update_expense` has the same pattern: `db.update_expense()` + `save_rule_if_categorized()` not atomic.

3. **`src-tauri/src/lib.rs:169-185`** — `save_llm_config` writes `llm_provider` and `llm_api_key` as two separate `db.set_config()` calls. A crash between them leaves provider with no key or vice versa.

4. **`src-tauri/src/lib.rs:200-204`** — `clear_llm_config` same two-write-no-transaction pattern.

5. **`src-tauri/src/lib.rs:423-429`** — `bulk_save_expenses` inserts expenses atomically via `insert_expenses_bulk` (good), but then calls `db.insert_rules_bulk(&rules)` as a separate transaction. `let _ =` silently drops rule errors.

6. **`src-tauri/src/lib.rs:678-697`** — `create_budget` calls `db.create_budget()` then `db.save_budget_categories()` as separate operations. Budget without categories if second call fails.

### Also

**`crates/core/src/db.rs:152-181`** — Budget migration (year/month → date-range) runs multiple DDL statements without a transaction. A crash between `DROP TABLE budgets` and `ALTER TABLE budgets_v2 RENAME` leaves the database with no `budgets` table. Also leaves `PRAGMA foreign_keys = OFF` if it crashes before reaching line 180.

## Scope

- Wrap each multi-write IPC command in a single DB transaction
- Add a `Database::transaction()` or `Database::with_transaction()` helper that takes a closure
- Wrap the budget migration in a transaction (note: DDL in SQLite auto-commits, so use a savepoint or restructure)
- Stop silently discarding rule insertion errors — propagate or log them
