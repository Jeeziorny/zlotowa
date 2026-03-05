# Task Board

## TODO

| # | Task | Summary |
|---|------|---------|
| 50 | Minor Polish | Shared constants (debounce, page sizes), version sourcing, date staleness fix, tighten `pub` visibility. |

## DONE

| # | Task | Summary |
|---|------|---------|
| 23 | UI Polish & Animations | Page fade transitions, widget FLIP/scale/stagger entrance, skeleton loading (Dashboard + Categories), toast notifications (AddExpense, LLM, Backup), count-up animation on TotalStats, chart grow-in (MonthlyTrend + SpendingByCategory), button press feedback, card-hover utility, `prefers-reduced-motion` support (CSS global kill + JS reactive boolean). 4 new files, ~10 modified. |

| # | Task | Summary |
|---|------|---------|
| 81 | Safer Backup Restore | Two-step restore: `preview_backup` IPC parses file and returns counts (expenses, rules, categories, budgets, created date). Preview card + red warning block + "backup first" button + confirmation checkbox gate. Restore button disabled until confirmed. 3 new tests. |
| 78 | Keyboard Shortcuts & Quick-Add | Global ⌘/Ctrl+N/U/K/1-4/Esc shortcuts. Sidebar `+ Add` button (1-click to add expense). `?` hint button opens cheat sheet overlay. ExpenseList subView bindable from App.svelte. Guards: skip in inputs (except Esc), skip if modal open, respect bulk upload dirty state. |
| 80b | Input Validation: Character Limits | Title maxlength 200 with graduated counter (appears at 150+, amber at 175+, red at 190+). Pattern maxlength 500. Category maxlength 100 via Autocomplete `maxlength` prop. |
| 77 | Accessibility Round 4 | Colorblind-safe budget bar labels (OK/80%+/Over), sign prefix on remaining (±), confidence badge icons (✓/~/!), aria-labels on all icon buttons, `.sr-only` live region for sort announcements in Categories. |
| 79 | Remember Column Mappings | Saved to `column_mappings` config key on step 2→3. Exact header match (priority 1), filename pattern match (priority 2). Restore banner with Reset link. 10-entry FIFO cap. Zero backend changes. |
| 73 | Widget Drag-and-Drop | HTML5 DnD in edit mode: drag handle ⠿, amber insertion line indicator (before/after), drop reorders + persists. Arrow fallback kept. Grid `items-start` removed for uniform row gaps. |
| 72 | Table Interaction Consistency | Unified action column (hover-reveal icons) + left border accent across all 3 tables. No hint (border accent only). |
| 71 | Autocomplete Component | Shared `Autocomplete.svelte` replacing fragile blur-timeout pattern in 4 components. Keyboard nav, proper focus. |
| 70 | Chart Visualizations | Unovis charts: SpendingByCategory → donut, MonthlyTrend → bar chart with axes/tooltips. Zero backend changes. |
| 69 | Rebrand to złotówa | Full rebrand: DB path migration (auto-migrates from old 4ccountant dir), sidebar coin logo+wordmark, app icons (.icns/.ico/PNGs), config renames (tauri.conf, package.json, Cargo.toml), Rust source strings, frontend backup filename, all docs. |
| 80a | Input Validation: Amounts | Bulk upload `Math.abs()` at save, AddExpense & ExpenseTable block `<= 0` with inline error, `min="0.01"` on number inputs. BudgetCreator already handled. |
| 75 | Filter Chip Bar | Dismissible chips between SearchFilterBar and ExpenseTable. Per-filter × removes one filter; "Clear all" appears when 2+ active. `activeFilterCount` derived. |
| 76 | Generic Confirm Modal | Shared `ConfirmModal.svelte` replacing 4 duplicate modals (DeleteConfirmModal, BatchDeleteModal, RuleDeleteModal, ConfirmLeaveModal). Reusable `focusTrap` action applied to all modals including Categories merge and Dashboard config dialog. ~150 lines of duplication removed. |
| 74 | Empty State Component | Shared `EmptyState.svelte` (page/widget variants) with 5 inline SVG icons and optional CTA button. Applied across 13 locations: ExpenseList (2), Dashboard, Categories, Rules (2), BudgetPlanning, UploadHistory, and 5 widgets. |
| 82 | Visual Identity & Color Palette | Emerald→amber/gold accent swap across 28 Svelte files. CSS custom properties for palette. Sidebar SVG icons (receipt, tag, pie, bolt, gear) with left-border active state. Primary buttons use dark text (`text-gray-950`) on amber. Semantic green preserved for budget health, success messages, confidence badges. |
| 67 | Classification Rules Tab | Rules page (opt-in via Settings toggle) with table view of regex rules, inline editing, filtering, match counts, add/delete. 3 new model types, 3 new DB methods (7 tests), 4 new IPC commands, 4 new Svelte components. PaginationBar generalized with `label` prop. |
| 66 | Bulk Upload Navigation Guard | Confirmation modal when navigating away from in-progress bulk upload. BulkUpload exposes dirty state via `ondirtychange` callback. ExpenseList guards "Back to Expenses" button, App.svelte guards sidebar/dashboard nav. Shared `ConfirmLeaveModal.svelte` component. |
| 65 | Multi-Instance Widgets & Keyword Tracker | Widget system evolved from flat string-array to instance-object array with per-instance config. Rust IPC widened to `serde_json::Value`. New KeywordTracker widget (keyword filter + monthly bar chart). Config dialog for add/edit. Old format auto-migrates. Multi-instance widgets always shown in picker with "(+ add another)". |
| 64 | Inline Title Cleanup | Replaced title cleanup rules engine with inline find-and-replace step in bulk upload flow (5-step wizard). Persists recent pairs in config table. Dropped `title_cleanup_rules` table, removed 5 IPC commands, deleted `TitleCleanup.svelte`. Added generic `get_config`/`save_config` IPC commands. Backup v2 format (ignores v1 cleanup rules gracefully). |
| 63 | Docs Sync Round 3 | Removed phantom Exporter trait/references, updated CLI commands (4: llm-conf, bulk-insert, backup, restore), IPC count 39→40, added backup domain, noted BudgetStatus self-fetch. |
| 62 | Accessibility Round 3 | `group-focus-within:opacity-100` on ExpenseTable actions, `aria-label` on icon buttons/checkboxes/inputs, decorative SVG `aria-hidden`, removed unnecessary svelte-ignore. |
| 61 | UX Feedback States | Double-submit prevention, auto-clear messages, timer cleanup, loading spinners, stale date fix, fire-and-forget catches. |
| 60 | Security Hardening | `.expect()` → `map_err` in restore, silent migration `let _ =` → logged warnings, LLM data leakage notice in Settings. |
| 59 | Bulk Upload Error Handling | try/catch + user-visible error banner for all 3 BulkUpload wizard steps (preview, classify, save). |
| 58 | Backup & Restore | Full app state backup to JSON (expenses, rules, title cleanup rules, budgets). Restore with dedup/upsert. CLI commands + Settings UI. |
| 49 | Component Splitting | Split BulkUpload (4 steps), ExpenseList (SearchFilterBar, ExpenseTable, PaginationBar), Settings (LLM + uploads). |
| 57 | Structured Logging | `log` + `tauri-plugin-log` for Rust backend. Stdout + LogDir targets, 5 MB rotation. Logging across IPC commands, DB, LLM, parsers, classifiers, exporters, iCal. |
| 56 | Calendar Suggestions | ICS upload during budget creation (drag-and-drop), in-memory parsing, event display in category budgets step. DB table dropped, no persistent storage. |
| 55 | Budget Planning Overhaul | DatePicker in budget creator, category amount validation, remove planned expenses (full cleanup), calendar tab disabled, budget navigation (prev/next arrows). |

| # | Task | Summary |
|---|------|---------|
| 52 | Title Cleanup Relocation | Title Cleanup accessible via "Clean Titles" button in Expenses toolbar, rendered as sub-view with back link. |
| 51 | Sidebar Restructuring & Expenses Page Consolidation | Sidebar reduced to 3 nav items + Settings. Logo clicks to Dashboard. Add Expense & Bulk Upload consolidated as sub-views in ExpenseList. Cleanup route removed (prep for task 52). |
| 54 | Auto-apply Title Cleanup During Bulk Upload | Added `suggest_title_cleanups` method to core DB. Bulk save now auto-applies existing cleanup rules to populate `display_title` silently. No UI changes needed. |
| 53 | `display_title` Data Model | Added `display_title` column to expenses. Title cleanup writes to `display_title`, raw `title` stays immutable after insert. Classification/duplicate detection still uses raw title. Export supports both columns. Tooltip shows raw title on hover. |
| 47 | Docs Sync Round 2 | CLAUDE.md: added 8 missing IPC commands, `ical` module, trait methods (`name`/`priority`/`extension`), grouped commands by domain. mdBook: dashboard "active budget", Claude Haiku 4.5, CLI clear fix, export source strings. |
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
| 40 | Unwrap & Mutex Safety | `budget.id.unwrap()` → `?`, mutex released before disk I/O in `export_expenses`, data built before lock in `bulk_save_expenses`. |
| 42 | Type Safety: Magic Strings | Case-insensitive `ClassificationSource::from_str_opt()` and `create_provider()`, `BudgetStatus` enum with `from_ratio()`, `UNCATEGORIZED` constant. |
| 43 | Dead Code Cleanup | Removed `ClassifiedExpense`, `ClassifyError::LlmNotConfigured`, `ExportError::Failed`, 2 `ParseError` variants, `filter_events_by_month`, `is_duplicate`, `get_all_budgets`, dead BudgetStatus prop, legacy LLM string parsing. |
| 44 | DB Constraint Hardening | FK indices on `budget_categories`, `planned_expenses`, `calendar_events`. UNIQUE index on `title_cleanup_rules`. |
| 45 | LLM Provider Dedup | Shared `http_classify()` helper for all 3 providers. Provider structs made private. ~80 lines removed. |
| 41 | Frontend Error Handling | User-visible error feedback for 6 `invoke()` catch blocks. Fixed TitleCleanup `deleteTarget` logic bug. Replaced native `confirm()` with custom modal. |
| 46 | Accessibility Round 2 | `aria-label` on icon-only buttons, `for` on form labels, `aria-modal`+`aria-labelledby` on dialogs, keyboard handler on CalendarEvents drop zone. |
| 48 | Integration Tests | 27 tests: parse→classify→save→query roundtrip, export→reimport, title cleanup→reclassify. Edge cases for `from_pattern` metacharacters, iCal, CSV, exporters, bulk insert. |

## N/A

| # | Task | Summary |
|---|------|---------|
| 16 | TBD | Notes file that spawned tasks 17–23. Not a task itself. |
| 27 | Navigation Fix | Dev-only HMR issue, not a production bug. |
