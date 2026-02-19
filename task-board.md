# Task Board

## TODO

| # | Task | Summary |
|---|------|---------|
| 19 | Chip Input Category | Replace plain text category input in BulkUpload review with a tag/chip component. Autocomplete from existing categories, add/remove with Enter/Backspace/x. |
| 23 | UI Polish & Animations | Page transitions, widget entrance animations, loading skeletons, toast notifications, micro-interactions (hover scale/shadow). Respect `prefers-reduced-motion`. |
| 25 | Bulk Upload UX Overhaul | LLM progress overlay, review table layout redesign (fields below row), chip input, column mapping simplification (1 row + placeholders), dismissible LLM warning, rename tab. |
| 28 | Title Cleanup Explanation | Add persistent help text explaining what Title Cleanup does and how it works. Clarify bulk upload integration. |
| 29 | Dashboard Widget Clicks | Make Total Expenses/Transactions click → Expenses tab, Spending by Category click → Categories tab. |
| 30 | Budget Planning Redesign | Date-range budgets (no overlap), Overview as default tab, multi-step "Create +" flow, category defaults from averages, calendar event amounts. |
| 33 | Accessibility Fixes | Escape-to-close modals, keyboard-accessible drop zone, keyboard-sortable table headers. |
| 34 | Tauri IPC Tests | Test coverage for all `#[tauri::command]` functions — happy paths + error paths with in-memory DB. |
| 37 | CLAUDE.md Sync | Add 7 missing IPC commands to docs. |

## DONE

| # | Task | Summary |
|---|------|---------|
| 01 | LLM Providers | `OpenAiProvider`, `AnthropicProvider`, `OllamaProvider` with `validate()` + `classify_batch()`. |
| 02 | LLM Pipeline Wiring | LLM fallback integrated into classification pipeline in `parse_and_classify`. |
| 03 | LLM Frontend | Settings UI for LLM config, validation feedback, source badges in bulk upload review. |
| 04 | CSV Exporter | `CsvExporter` with configurable columns, proper CSV escaping. |
| 05 | Export UI | Export modal in ExpenseList with column checkboxes, native save dialog. |
| 06 | CLI | 5 commands: `llm-conf`, `insert`, `bulk-insert`, `export`, `dashboard`. |
| 07 | Category Autocomplete | `suggest_category` IPC command, autocomplete in AddExpense. |
| 08 | Docs Alignment | mdBook docs updated to reflect implemented features. |
| 09 | LLM Classification Hardening | Confidence tiers, amounts in prompt, keyed ID-based responses, temperature tuning. |
| 10 | Click-to-Assign Column Mapping | Popover on header click to assign Title/Amount/Date, color-coded columns, auto date-format detection. |
| 11 | UX Polish (DatePicker + warnings) | Custom DatePicker component, LLM warning in bulk upload. |
| 12 | Editable Rule Pattern | "Match keyword" column in review, auto-apply category to similar expenses. |
| 13 | Category Management | Categories page with stats, rename, delete (with reassignment), merge. |
| 14 | Fix CSV Export Download | Native file dialog via `@tauri-apps/plugin-dialog`. |
| 15 | Code Quality Cleanup | Batch duplicate check, `unwrap_or_else`, `.get()` access, enum usage, error propagation, DB indices, `from_pattern()`. |
| 17 | Budget Planning + Calendar Import | Monthly budgets per category, planned expenses, iCal import, overview with progress bars, BudgetStatus dashboard widget. |
| 20 | Expense CRUD | Inline editing (title, amount, date, category) and deletion (single + batch) in ExpenseList. |
| 18 | Batch Undo | `upload_batches` table + `batch_id` on expenses, batch tracking in bulk upload (GUI + CLI), upload history with undo in Settings. |
| 21 | Title Cleanup Rules | Find/replace rules (literal or regex), preview affected expenses, selective apply, whitespace normalization. |
| 22 | Pagination, Search & Filter | `query_expenses` API with title search (LIKE), category/date/amount filters, pagination (limit/offset). Search bar + filter controls + page size selector in ExpenseList. |
| 24 | Tests & Docs Sync | 11 new tests for category management, batch duplicate check, upload batches. Docs updated: roadmap, navigation, dashboard widget table, introduction feature list. |
| 26 | Expense List Cleanup | Removed Source column, Category column wider, delete confirmation modals (single + batch). |
| 24b | DEMO_TBD | Client demo notes file that spawned tasks 25–30. Not a task itself. |
| 35 | DB & Core Edge Case Tests | 29 edge case tests across db.rs, classifiers.rs, llm.rs, csv_parser.rs. |
| 32 | Release Mutex Before LLM | Restructured `parse_and_classify()` into 5 phases — DB lock released before LLM HTTP calls. |
| 36 | ClassificationSource Roundtrip | Already fixed — `from_str_opt()` handles both cases, roundtrip test exists. |
| 31 | DB Query Performance | Batch N+1 queries (title cleanup, delete), replace `strftime()` with range queries, chunk duplicate check, add budget index. 7 new tests. |

## N/A

| # | Task | Summary |
|---|------|---------|
| 16 | TBD | Notes file that spawned tasks 17–23. Not a task itself. |
| 27 | Navigation Fix | Dev-only HMR issue, not a production bug. |
