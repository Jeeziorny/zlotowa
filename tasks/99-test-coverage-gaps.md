# Task 99: Test Coverage Gaps

## Goal

Add tests for untested public API surface and missing edge cases identified in the audit.

## Deliverables

### 1. `Database::open(path)` — `db.rs`

Test with:
- Valid path (creates DB file)
- Non-existent parent directory (should error, not panic)
- Read-only directory (permission error handling)

### 2. End-to-end integration test

CSV string → `CsvParser::parse()` → `RegexClassifier::classify_pipeline()` → `Database::insert_expenses_bulk()` → `Database::query_expenses()` and verify the roundtrip. This should live in `crates/core/src/` as an integration test or in the existing `tests` module.

### 3. `BudgetStatus::from_ratio()` boundaries — `models.rs`

Test: `from_ratio(0.0)`, `from_ratio(0.8)`, `from_ratio(0.80001)`, `from_ratio(1.0)`, `from_ratio(1.5)`, `from_ratio(-0.1)`.

### 4. `get_category_averages(months)` edge cases — `db.rs`

Test: `months = 0`, `months = -1`, `months = 1` (single month), `months = 120` (10 years, likely no data).

### 5. `delete_category()` edge cases — `db.rs`

Test: delete category to itself (self-referential), delete non-existent category, delete last category.

## Files to modify
- `crates/core/src/db.rs` (test module)
- `crates/core/src/models.rs` (test module)
- Possibly a new integration test file

## Notes
- Use `Database::open_memory()` for unit tests, `Database::open(path)` only for the filesystem tests (use `tempfile` crate).
- Don't add the `tempfile` dependency if a `std::env::temp_dir()` + cleanup approach works.
