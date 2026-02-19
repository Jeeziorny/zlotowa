# 48 — Integration & Roundtrip Tests

## Problem

Tests cover individual stages but never the full pipeline end-to-end. The seams between stages are the most likely place for bugs.

### Missing roundtrip tests

1. **Parse → classify → save → query** — No test goes through: parse CSV → classify via regex → save to DB → query back → verify data integrity. The closest (`parse_and_classify_with_existing_rule`) stops before saving.

2. **Export → reimport** — No test verifies that CSV exported by `CsvExporter` can be re-parsed by `CsvParser` and produce the same data. Round-trip fidelity is untested.

3. **Title cleanup → classify** — Title cleanup rules modify expense titles in the DB. If a cleanup changes a title to now match a classification rule, this workflow is untested.

### Missing edge case coverage in existing tests

4. **`db.rs:26`** — `Database::open_default()` never tested (only `open_memory()` used). The production code path with `dirs::data_dir()` and `create_dir_all` is untested.

5. **`db.rs:61`** — Budget date-range migration (lines 152-181) never tested. `open_memory()` always creates fresh tables, so the `has_old_schema` branch is unreachable in tests.

6. **`models.rs:73`** — `ClassificationRule::from_pattern()` never tested for regex metacharacters in titles (parentheses, dots, brackets).

7. **`parsers/csv_parser.rs`** — Empty input, Unicode currency symbols, blank lines interspersed, `\r\n` line endings — all untested.

8. **`ical.rs:31`** — `parse_ics()` with malformed iCal data (truncated mid-event, BOM, invalid dates) — only well-formed input tested. Unicode in SUMMARY also untested.

9. **`exporters.rs`** — Unicode in titles and `\r\n` (CRLF) in fields untested. `NaN`/`Infinity` amounts would produce invalid CSV.

10. **`db.rs:216`** — `insert_expenses_bulk()` with empty slice + `batch_filename = Some("test.csv")` creates a batch with `expense_count = 0`. Untested edge case.

## Scope

- Add parse → classify → save → query roundtrip test
- Add export → reimport roundtrip test
- Add title cleanup → reclassify test
- Add budget migration test (create old-schema DB, run migrate, verify data)
- Add targeted edge case tests for items 6-10
