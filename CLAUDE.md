# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Commands

```bash
# Development
npm run dev              # Vite dev server (port 1420)
npm run tauri dev        # Full Tauri desktop app (runs Vite automatically)

# Build
npm run build            # Frontend only
cargo build --release    # Rust only

# Tests
cargo test                       # All workspace tests
cargo test -p accountant-core    # Core crate only
cargo test -p accountant-app     # Tauri IPC layer tests
cargo test -p accountant-core -- test_name  # Single test

# Docs (mdBook)
mdbook serve docs                # Local preview at localhost:3000
mdbook build docs                # Build to docs/book/
```

## Architecture

Rust + Tauri v2 + Svelte 5 + Tailwind CSS v4 desktop app for expense tracking and classification.

**Cargo workspace:**
- `crates/core` (`accountant-core`) — business logic: parsers, classifiers, DB, models
- `crates/cli` (`accountant-cli`) — CLI binary (5 commands: llm-conf, insert, bulk-insert, export, dashboard)
- `src-tauri` (`accountant-app`) — Tauri IPC commands, app state

**Data flow:**
CSV → Parser (auto-detect format) → ParsedExpense → Classification Pipeline (regex rules from DB → LLM fallback) → Expense → SQLite

### Key Traits

**Parser** (`crates/core/src/parsers/`): `detect()` returns confidence score, `preview_rows()` for column mapping UI, `parse()` with user-confirmed `ColumnMapping`. Currently only `CsvParser`.

**Classifier** (`crates/core/src/classifiers.rs`): `classify()` returns `Option<ClassificationResult>`. Pipeline runs classifiers by priority (lower = first), winner-takes-all. `RegexClassifier` from DB rules; LLM fallback runs post-pipeline as a batch call.

**LLM** (`crates/core/src/llm.rs`): `LlmProvider` trait with `validate()` and `classify_batch()`. Implementations: `OpenAiProvider`, `AnthropicProvider`, `OllamaProvider`. Uses blocking `reqwest`.

**Exporter** (`crates/core/src/exporters.rs`): `Exporter` trait with `export()` returning bytes. `CsvExporter` with configurable `ExportColumns`.

### Tauri IPC Bridge

All commands in `src-tauri/src/lib.rs`. State is `AppState { db: Mutex<Database> }`.

Key commands: `get_expenses`, `query_expenses`, `add_expense`, `suggest_category`, `preview_csv`, `parse_and_classify`, `bulk_save_expenses`, `get_categories`, `get/save/validate/clear_llm_config`, `export_expenses`, `get/save_active_widgets`, `get/save/delete_title_cleanup_rule(s)`, `preview/apply_title_cleanup`, `get_budget_summary(budget_id)`, `get_active_budget_summary`, `create_budget(start_date, end_date, categories)`, `save_budget_categories(budget_id, categories)`, `add/delete_planned_expense(budget_id, ...)`, `delete_budget(id)`, `import_calendar_events(budget_id, ics_content)`, `update_calendar_event_amount(event_id, amount)`, `check_budget_overlap(start_date, end_date)`, `get_category_averages`, `get_upload_batches`, `delete_batch`.

Frontend calls via `invoke("command_name", { params })` from `@tauri-apps/api/core`.

### Database

SQLite at `~/Library/Application Support/4ccountant/4ccountant.db` (macOS). Schema auto-created via `migrate()`.

Tables: `expenses` (has optional `batch_id` FK), `classification_rules` (regex pattern → category), `title_cleanup_rules` (find/replace rules for title noise), `config` (key-value for LLM settings, widget state), `budgets` (start_date/end_date date-range, no overlap allowed), `budget_categories` (per-category limits), `planned_expenses` (upcoming costs), `calendar_events` (imported iCal events, optional `amount` column), `upload_batches` (filename, timestamp, count for bulk upload undo).

Duplicate detection on `(title, amount, date)` tuple. Bulk inserts use transactions.

Error handling: `DbError` enum via `thiserror`.

### Frontend

SPA routing in `App.svelte` with string-based page state. Pages: Dashboard, AddExpense, BulkUpload, ExpenseList, Categories, TitleCleanup, BudgetPlanning, Settings.

Dashboard widgets registered in `src/lib/widgets/registry.js`, widget visibility/order persisted to DB.

## Conventions

- **Svelte 5 syntax** — `$state()`, `$derived()`, `$props()`, `onclick={}`, callback props (NOT Svelte 4 patterns)
- **Tailwind v4** — imported as `@import "tailwindcss"` in CSS, plugin is `@tailwindcss/postcss`
- **Dark theme** — gray-950 background, gray-900 cards, gray-800 borders, emerald-400/500 accents
- **Rust errors** — `thiserror` for custom error enums, Tauri commands return `Result<T, String>`
- **Auto-rules** — when user manually categorizes an expense, a regex rule `(?i)<escaped_title>` is auto-saved

## Task Board

`task-board.md` tracks all tasks (TODO / DONE / N/A). Individual task specs live in `tasks/##-name.md`. Always consult the board before picking work and update it after completing a task.

## Extending

- **New parser:** Implement `Parser` trait in `crates/core/src/parsers/`, add to `detect_parser()`
- **New classifier:** Implement `Classifier` trait, add to pipeline in caller
- **New widget:** Create `.svelte` in `src/lib/widgets/`, register in `registry.js`
- **New LLM provider:** Implement `LlmProvider` trait in `crates/core/src/llm.rs`, add to `create_provider()` factory
- **New export format:** Implement `Exporter` trait in `crates/core/src/exporters.rs`
- **New IPC command:** Add `#[tauri::command]` fn in `src-tauri/src/lib.rs`, register in `invoke_handler!`
