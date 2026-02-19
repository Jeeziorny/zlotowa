# Task Board

## TODO

| # | Task | Summary |
|---|------|---------|
| 23 | UI Polish & Animations | Page transitions, widget entrance animations, loading skeletons, toast notifications, micro-interactions (hover scale/shadow). Respect `prefers-reduced-motion`. |
| 40 | Unwrap & Mutex Safety | Replace `budget.id.unwrap()` with `?`. Release mutex before disk I/O in `export_expenses`. Build data before acquiring lock in `bulk_save_expenses`. |
| 41 | Frontend Error Handling | User-visible error feedback for 6 `invoke()` catch blocks. Fix TitleCleanup `deleteTarget` logic bug. Replace native `confirm()` with custom modal. |
| 42 | Type Safety: Magic Strings | `LlmProviderType` enum, case-insensitive `ClassificationSource` parsing, `BudgetStatus` enum, `"uncategorized"` constant. |
| 43 | Dead Code Cleanup | Remove unused structs (`ClassifiedExpense`), enum variants (`ClassifyError`, `ExportError::Failed`, 2 `ParseError` variants), functions (`filter_events_by_month`, `is_duplicate`, `get_all_budgets`), dead prop. |
| 44 | DB Constraint Hardening | UNIQUE on `expenses(title,amount,date)`, ON DELETE for `batch_id` FK, FK indices on child tables, UNIQUE on `title_cleanup_rules`. |
| 45 | LLM Provider Dedup | Extract shared HTTP classify helper across 3 providers. Reduce ~120 lines of duplicated code. Tighten provider struct visibility. |
| 46 | Accessibility Round 2 | `aria-label` on icon-only buttons, `for` on form labels, `aria-modal`+`aria-labelledby` on dialogs, keyboard handler on CalendarEvents drop zone. |
| 47 | Docs Sync Round 2 | CLAUDE.md: missing commands, `ical` module, trait methods. mdBook: budget "monthly"→"date-range", dashboard "active budget", LLM model version, export source strings. |
| 48 | Integration Tests | Parse→classify→save→query roundtrip, export→reimport roundtrip, title cleanup→classify. Budget migration test. Edge cases for `from_pattern`, iCal, CSV, exporters. |
| 49 | Component Splitting | Split BulkUpload (4 steps), ExpenseList (extract modals), Settings (LLM + uploads). |
| 50 | Minor Polish | Shared constants (debounce, page sizes), version sourcing, date staleness fix, tighten `pub` visibility. |

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
| 19 | Chip Input Category | Implemented as part of task 25 — chip input with autocomplete in BulkUpload review. |
| 25 | Bulk Upload UX Overhaul | Tab rename, dismissible LLM warning, 1 preview row, LLM progress overlay, card layout with chip category input. Docs updated. |
| 33 | Accessibility Fixes | Escape-to-close modals (Categories, ExpenseList, TitleCleanup), keyboard-accessible drop zone (Enter/Space), keyboard-sortable table headers with `aria-sort`. |
| 30 | Budget Planning Redesign | Date-range budgets (no overlap), Overview as default tab, multi-step "Create +" flow, category defaults from averages, calendar event amounts. |
| 37 | CLAUDE.md Sync | IPC commands synced with code. |
| 28 | Title Cleanup Explanation | Collapsible help section in TitleCleanup page, docs "Bulk Import" section, test confirming rules don't auto-apply on insert. |
| 29 | Dashboard Widget Clicks | Total Expenses/Transactions → Expenses tab, Spending by Category/Categories count → Categories tab. Hover affordance on clickable widgets. |
| 34 | Tauri IPC Tests | 35 tests for all `#[tauri::command]` functions — expense CRUD, query/filter, bulk save/undo, CSV parse/classify, budget lifecycle, calendar import, title cleanup, categories, widget config, LLM config, export. Uses Tauri MockRuntime + in-memory DB. |
| 39 | Transaction Safety | `with_transaction` helper, atomic multi-write IPC commands, `insert_expenses_bulk` includes rules, `create_budget_with_categories`, budget migration in transaction, removed silent `let _ =` error drops. 4 new tests. |

## N/A

| # | Task | Summary |
|---|------|---------|
| 16 | TBD | Notes file that spawned tasks 17–23. Not a task itself. |
| 27 | Navigation Fix | Dev-only HMR issue, not a production bug. |
