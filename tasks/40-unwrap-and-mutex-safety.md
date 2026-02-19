# 40 — Unwrap & Mutex Safety

## Problem

### Panic risk

**`src-tauri/src/lib.rs:579`** — `build_budget_summary()` calls `budget.id.unwrap()`. If a `Budget` without an ID reaches this function, the app crashes. Should use `.ok_or_else(|| "Budget has no id".to_string())?`.

### Mutex held during disk I/O

**`src-tauri/src/lib.rs:365-378`** — `export_expenses()` holds the DB mutex lock while calling `std::fs::write()`. All other Tauri commands are blocked during file export. Should release lock after fetching data, then write to disk outside the lock.

### Mutex held during expense construction

**`src-tauri/src/lib.rs:389-431`** — `bulk_save_expenses()` holds the lock for the entire function, including building the expense/rule vecs (pure computation). Construction should happen before acquiring the lock.

## Scope

- Replace `budget.id.unwrap()` with `?` error propagation
- In `export_expenses()`: clone/collect expense data, release lock, then write file
- In `bulk_save_expenses()`: build expense/rule vecs before acquiring the lock
