# 34 — Tauri IPC Layer Tests

## Problem

All 30+ `#[tauri::command]` functions in `src-tauri/src/lib.rs:89-761` have zero test coverage. This is the entire frontend-backend boundary. Key untested workflows:

- `parse_and_classify` — full parse→classify→deduplicate flow, LLM fallback failure
- `bulk_save_expenses` — partial batch failure, transaction rollback
- `update_expense` / `delete_expenses` — nonexistent IDs, invalid data
- `get_budget_summary` — missing budget data, category mismatch
- `import_calendar_events` — malformed .ics, out-of-month filtering

## Scope

- Add `#[cfg(test)]` module to `src-tauri/src/lib.rs` (or separate test file)
- Test happy paths for the most critical commands: `add_expense`, `get_expenses`, `query_expenses`, `parse_and_classify`, `bulk_save_expenses`, `get_budget_summary`
- Test error paths: invalid inputs, empty data, nonexistent IDs
- Use in-memory DB (`Database::open_memory()`) for test isolation
