# 43 — Dead Code Cleanup

## Problem

Several structs, enum variants, and functions exist in the codebase but are never used in production.

### Dead types

1. **`crates/core/src/models.rs:186-191`** — `ClassifiedExpense` struct — replaced by `ClassifiedExpenseRow` in the Tauri layer
2. **`crates/core/src/classifiers.rs:7-12`** — `ClassifyError` enum — never constructed or returned
3. **`crates/core/src/exporters.rs:5-9`** — `ExportError::Failed(String)` variant — never constructed
4. **`crates/core/src/parsers/mod.rs:9-10`** — `ParseError::UnrecognizedFormat` — never constructed
5. **`crates/core/src/parsers/mod.rs:13-14`** — `ParseError::TooFewExpenses` — never constructed

### Dead functions

6. **`crates/core/src/ical.rs:106-116`** — `filter_events_by_month()` — never called from production code (replaced by `filter_events_by_date_range()`)
7. **`crates/core/src/db.rs:398-405`** — `Database::is_duplicate()` — never called (batch variant `check_duplicates_batch` used instead)
8. **`crates/core/src/db.rs:992-1022`** — `Database::get_all_budgets()` — never called outside tests

### Dead props

9. **`src/lib/widgets/BudgetStatus.svelte:5`** — `expenses` prop destructured but never used

### Dead code in LLM

10. **`crates/core/src/llm.rs:131-143`** — Legacy flat-string response parsing (`["Groceries", "Transport"]` format). If no LLM currently produces this format, it's dead code.

## Scope

- Remove items 1-9 (clear dead code)
- For item 10 (legacy parsing): verify no provider uses this format, then remove
- For items 7-8 (dead DB methods): remove from production, keep tests only if they exercise useful paths
