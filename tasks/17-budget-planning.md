# Task 17: Monthly Budget Planning with Calendar Import

**Track:** A — New Feature (full stack: core + IPC + frontend)
**Blocked by:** nothing
**Blocks:** nothing

## Problem

4ccountant tracks and classifies past expenses but has no forward-looking budgeting. The user currently plans monthly budgets outside the app by:
1. Reviewing past spending per category in spreadsheets
2. Checking their Google Calendar for upcoming cost-generating events (doctor appointments, dinners, car service)
3. Setting category spending limits and comparing actual spending against the plan

This workflow belongs inside the app — the historical expense data is already here, and the plan-vs-actual comparison is a natural extension of the existing category breakdown.

**What "debt" means here:** overspending the budget. When a category goes over its limit, that's the user's concept of debt. No separate debt/loan tracker needed.

## Current State

### Data model (`crates/core/src/models.rs`)
- `Expense` struct (lines 6–14): `id, title, amount, date, category, classification_source`
- Categories are strings stored in `classification_rules.category` — retrieved as distinct values via `db.get_all_categories()` (`crates/core/src/db.rs:280–286`)
- No budget-related structs exist

### Database (`crates/core/src/db.rs`)
- Three tables: `expenses`, `classification_rules`, `config` (lines 57–84)
- Migration via `migrate()` method using `execute_batch` with `CREATE TABLE IF NOT EXISTS` — idempotent, safe to extend (line 57)
- `get_all_expenses()` returns all expenses ordered by date DESC (lines 142–170) — no month-filtering method exists
- `get_all_categories()` returns `SELECT DISTINCT category FROM classification_rules` (lines 280–286)
- No foreign key pragma currently set

### IPC layer (`src-tauri/src/lib.rs`)
- `AppState { db: Mutex<Database> }` (lines 10–12)
- Pattern: `#[tauri::command]` fns acquire `state.db.lock()`, call DB methods, return `Result<T, String>` (e.g. `get_expenses` at line 86)
- `parse_date()` helper at line 66 — reusable for budget dates
- Commands registered in `invoke_handler!` macro (lines 478–498)
- No budget commands exist

### Frontend routing (`src/App.svelte`)
- String-based routing with `currentPage` state (line 10)
- 6 pages: dashboard, add, bulk, expenses, categories, settings (lines 17–29)

### Sidebar (`src/lib/Sidebar.svelte`)
- Navigation items array (lines 4–9): dashboard, add, bulk, expenses, categories
- Settings rendered separately at bottom (lines 33–44)

### Dashboard widgets (`src/lib/widgets/registry.js`)
- Widget objects: `{ id, name, description, size, component }` (lines 17–53)
- All widgets receive `expenses` prop — passed from `Dashboard.svelte` line 142
- `Dashboard.svelte` loads all expenses via `get_expenses` (line 23) and active widget IDs from config (line 25)

### File upload pattern (`src/lib/BulkUpload.svelte`)
- Step-based flow with `$state("input")` (line 6)
- File input + textarea for CSV paste (same pattern needed for .ics import)
- FileReader to read file contents as text, then pass to IPC command

### Calendar data format (from user's `kamij98@gmail.com.ical.zip`)
- Standard iCal/VCALENDAR with VEVENT blocks
- 1244 events across 2 .ics files (personal + family calendar)
- Three DTSTART formats to handle:
  - `DTSTART:20250227T160000Z` — UTC datetime
  - `DTSTART;VALUE=DATE:20241229` — all-day event
  - `DTSTART;TZID=Europe/Warsaw:20251118T180000` — timezone-qualified datetime
- Relevant fields per event: SUMMARY, DESCRIPTION, LOCATION, DTSTART, DTEND
- Some events have RRULE (recurring) — skip for v1, parse only single instances

## Scope

### 1. Add budget structs to `crates/core/src/models.rs`

New structs with `#[derive(Debug, Clone, Serialize, Deserialize)]`:

```rust
pub struct Budget { pub id: Option<i64>, pub year: i32, pub month: u32 }
pub struct BudgetCategory { pub id: Option<i64>, pub budget_id: i64, pub category: String, pub amount: f64 }
pub struct PlannedExpense { pub id: Option<i64>, pub budget_id: i64, pub title: String, pub amount: f64, pub date: NaiveDate, pub category: Option<String> }
pub struct CalendarEvent { pub id: Option<i64>, pub budget_id: i64, pub summary: String, pub description: Option<String>, pub location: Option<String>, pub start_date: NaiveDate, pub end_date: Option<NaiveDate>, pub all_day: bool }
pub struct BudgetCategoryStatus { pub category: String, pub budgeted: f64, pub spent: f64, pub status: String }
pub struct CategoryAverage { pub category: String, pub average: f64, pub months_with_data: u32 }
```

### 2. Add 4 new tables to `migrate()` in `crates/core/src/db.rs`

Add `PRAGMA foreign_keys = ON;` at the start of the migration batch (line 59), then append:

```sql
CREATE TABLE IF NOT EXISTS budgets (
    id    INTEGER PRIMARY KEY AUTOINCREMENT,
    year  INTEGER NOT NULL,
    month INTEGER NOT NULL,
    UNIQUE(year, month)
);
CREATE TABLE IF NOT EXISTS budget_categories (
    id        INTEGER PRIMARY KEY AUTOINCREMENT,
    budget_id INTEGER NOT NULL REFERENCES budgets(id) ON DELETE CASCADE,
    category  TEXT NOT NULL,
    amount    REAL NOT NULL,
    UNIQUE(budget_id, category)
);
CREATE TABLE IF NOT EXISTS planned_expenses (
    id        INTEGER PRIMARY KEY AUTOINCREMENT,
    budget_id INTEGER NOT NULL REFERENCES budgets(id) ON DELETE CASCADE,
    title     TEXT NOT NULL,
    amount    REAL NOT NULL,
    date      TEXT NOT NULL,
    category  TEXT
);
CREATE TABLE IF NOT EXISTS calendar_events (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    budget_id   INTEGER NOT NULL REFERENCES budgets(id) ON DELETE CASCADE,
    summary     TEXT NOT NULL,
    description TEXT,
    location    TEXT,
    start_date  TEXT NOT NULL,
    end_date    TEXT,
    all_day     INTEGER NOT NULL DEFAULT 0
);
```

### 3. Add ~10 new DB methods to `Database` impl in `crates/core/src/db.rs`

Add a `// ── Budgets ──` section after the Config section (after line 398):

- `get_or_create_budget(year: i32, month: u32) -> Result<i64, DbError>` — INSERT OR IGNORE + SELECT
- `get_budget(year: i32, month: u32) -> Result<Option<Budget>, DbError>`
- `save_budget_categories(budget_id: i64, categories: &[BudgetCategory]) -> Result<(), DbError>` — DELETE existing + INSERT all (transaction)
- `get_budget_categories(budget_id: i64) -> Result<Vec<BudgetCategory>, DbError>`
- `insert_planned_expense(expense: &PlannedExpense) -> Result<i64, DbError>`
- `delete_planned_expense(id: i64) -> Result<(), DbError>`
- `get_planned_expenses(budget_id: i64) -> Result<Vec<PlannedExpense>, DbError>`
- `save_calendar_events(budget_id: i64, events: &[CalendarEvent]) -> Result<usize, DbError>` — DELETE old + INSERT new (re-import replaces)
- `get_calendar_events(budget_id: i64) -> Result<Vec<CalendarEvent>, DbError>`
- `get_expenses_for_month(year: i32, month: u32) -> Result<Vec<Expense>, DbError>` — `WHERE strftime('%Y', date) = ? AND strftime('%m', date) = ?`
- `get_category_averages(months: u32) -> Result<Vec<CategoryAverage>, DbError>` — AVG of monthly totals per category over last N months

### 4. Create iCal parser at `crates/core/src/ical.rs`

Add `ical = "0.11"` to `crates/core/Cargo.toml` (after line 14). Register `pub mod ical;` in `crates/core/src/lib.rs` (after line 6).

```rust
pub struct ParsedCalendarEvent {
    pub summary: String,
    pub description: Option<String>,
    pub location: Option<String>,
    pub start_date: NaiveDate,
    pub end_date: Option<NaiveDate>,
    pub all_day: bool,
}

pub fn parse_ics(input: &str) -> Result<Vec<ParsedCalendarEvent>, IcalError>
pub fn filter_events_by_month(events: &[ParsedCalendarEvent], year: i32, month: u32) -> Vec<ParsedCalendarEvent>
```

Date parsing must handle all three DTSTART formats. Skip events without SUMMARY or DTSTART. Ignore RRULE for v1.

### 5. Add 6 IPC commands to `src-tauri/src/lib.rs`

Add new types and commands after the widget config section (after line 461):

| Command | Params | Returns | Logic |
|---------|--------|---------|-------|
| `get_budget_summary` | `year: i32, month: u32` | `BudgetSummaryOutput` | Get/create budget → load categories, planned, calendar, actual expenses → compute per-category status (under/approaching/over) → return assembled summary |
| `save_budget_categories` | `year: i32, month: u32, categories: Vec<{category, amount}>` | `()` | Get/create budget → save categories |
| `add_planned_expense` | `year: i32, month: u32, expense: {title, amount, date, category?}` | `i64` | Get/create budget → insert planned expense |
| `delete_planned_expense` | `id: i64` | `()` | Delete by id |
| `import_calendar_events` | `year: i32, month: u32, ics_content: String` | `usize` | Parse .ics → filter to month → save to DB |
| `get_category_averages` | — | `Vec<{category, average, months_with_data}>` | Last 3 months |

Status thresholds: `spent/budgeted < 0.8` → "under", `0.8–1.0` → "approaching", `> 1.0` → "over".

Register all 6 in the `invoke_handler!` macro (line 478). Add `use accountant_core::ical;` import at top.

### 6. Create Budget page at `src/lib/BudgetPlanning.svelte`

Page shell with month selector and 3 tabs:

```
[ < ]  March 2026  [ > ]
[ Budget | Calendar | Overview ]
```

Loads data via `get_budget_summary` on mount and on month change. Sub-components:

### 7. Create `src/lib/budget/MonthSelector.svelte`

```javascript
let { year, month, onchange } = $props();
// Renders: [ < ] March 2026 [ > ]
// Prev/next buttons call onchange({year, month})
```

### 8. Create `src/lib/budget/BudgetEditor.svelte`

```javascript
let { year, month, categories, averages, budgetCategories, plannedExpenses, onrefresh } = $props();
```

- Table: Category | Avg (last 3mo, from `averages`) | Budget amount (input field)
- Pre-populate input with existing `budgetCategories` values
- Below: planned expenses list with delete button per row
- Add form: title, amount, date picker, category dropdown
- Save button calls `save_budget_categories` then `onrefresh()`

### 9. Create `src/lib/budget/CalendarEvents.svelte`

```javascript
let { year, month, events, onrefresh } = $props();
```

- File drop zone (reuse pattern from BulkUpload.svelte step 1 — file input + drag area)
- Accept `.ics` files, read with `FileReader.readAsText()`, call `import_calendar_events`
- Display event list: date | summary | location
- Count indicator: "N events for March 2026"

### 10. Create `src/lib/budget/BudgetOverview.svelte`

```javascript
let { categories, plannedExpenses, totalBudgeted, totalSpent, totalPlanned } = $props();
```

- Per-category progress bars (similar visual to `SpendingByCategory.svelte` lines 29–33)
- Bar color: `bg-emerald-500` (under), `bg-amber-500` (approaching), `bg-red-500` (over)
- Over-budget: bar capped at 100%, red text "OVER by X.XX"
- Planned expenses section below
- Total summary card at bottom: budgeted | spent | remaining or "OVER by X"

### 11. Add routing in `src/App.svelte`

- Import `BudgetPlanning` (after line 8)
- Add `{:else if currentPage === "budget"}` block with `<BudgetPlanning />` (after line 25)

### 12. Add sidebar nav item in `src/lib/Sidebar.svelte`

Add `{ id: "budget", label: "Budget", icon: "◎" }` after the categories entry (line 9).

### 13. Create dashboard widget `src/lib/widgets/BudgetStatus.svelte`

Compact card for current month's budget:
- Fetches own data via `get_budget_summary` (independent of `expenses` prop)
- Shows: Budgeted / Spent / Remaining
- Full-width progress bar with color coding
- "N categories over budget" warning
- "No budget set" fallback with hint to visit Budget page

### 14. Register widget in `src/lib/widgets/registry.js`

Add import and entry: `{ id: "budget-status", name: "Budget Status", description: "...", size: "half", component: BudgetStatus }` (after line 52).

## Files to Change

| File | Change |
|---|---|
| `crates/core/Cargo.toml` | Add `ical = "0.11"` dependency |
| `crates/core/src/lib.rs` | Add `pub mod ical;` |
| `crates/core/src/models.rs` | Add 6 new structs (Budget, BudgetCategory, PlannedExpense, CalendarEvent, BudgetCategoryStatus, CategoryAverage) |
| `crates/core/src/db.rs` | Add 4 tables to `migrate()`, add ~10 new methods in `// ── Budgets ──` section |
| `crates/core/src/ical.rs` | **NEW** — iCal parser with `parse_ics()` and `filter_events_by_month()` |
| `src-tauri/src/lib.rs` | Add 6 new IPC commands + output types, register in `invoke_handler!` |
| `src/App.svelte` | Add budget route + import |
| `src/lib/Sidebar.svelte` | Add Budget nav item |
| `src/lib/BudgetPlanning.svelte` | **NEW** — page shell with month selector + tabs |
| `src/lib/budget/MonthSelector.svelte` | **NEW** — month prev/next navigation |
| `src/lib/budget/BudgetEditor.svelte` | **NEW** — category budget inputs + planned expense CRUD |
| `src/lib/budget/CalendarEvents.svelte` | **NEW** — .ics upload + event list |
| `src/lib/budget/BudgetOverview.svelte` | **NEW** — progress bars + totals |
| `src/lib/widgets/BudgetStatus.svelte` | **NEW** — dashboard widget |
| `src/lib/widgets/registry.js` | Register BudgetStatus widget |

## Test Scenarios

### Backend (Rust unit tests)

1. **Budget creation:** `get_or_create_budget(2026, 3)` returns an id; calling again returns the same id
2. **Category budgets:** Save 3 category budgets, retrieve them, verify amounts; save again with different values, verify replacement
3. **Planned expenses:** Insert 2 planned expenses, retrieve by budget_id, verify fields; delete one, verify only 1 remains
4. **Calendar events:** Save 5 events for a budget, retrieve them; re-save with 3 events, verify old ones are replaced (count = 3)
5. **Month filtering:** Insert expenses across Jan, Feb, Mar 2026; `get_expenses_for_month(2026, 2)` returns only February expenses
6. **Category averages:** Insert expenses: Jan Food=100, Feb Food=200, Mar Food=150; `get_category_averages(3)` returns Food avg=150
7. **iCal parsing — UTC datetime:** Parse `DTSTART:20250227T160000Z` → date 2025-02-27, all_day=false
8. **iCal parsing — all-day:** Parse `DTSTART;VALUE=DATE:20241229` → date 2024-12-29, all_day=true
9. **iCal parsing — timezone:** Parse `DTSTART;TZID=Europe/Warsaw:20251118T180000` → date 2025-11-18, all_day=false
10. **iCal parsing — missing SUMMARY:** Event without SUMMARY is skipped, no error
11. **iCal month filter:** 10 events across 3 months, `filter_events_by_month(_, 2025, 3)` returns only March events

### Frontend (manual UI tests)

1. Click Budget in sidebar → Budget page opens with current month displayed
2. Navigate months with < > arrows → data reloads for selected month
3. Budget tab: categories from existing rules appear in the table, "Avg" column shows historical values
4. Enter budget amounts for 3 categories, click Save → refresh page, amounts persist
5. Add a planned expense (title: "Dentist", amount: 300, date: Mar 12, category: Health) → appears in list
6. Delete a planned expense → removed from list
7. Calendar tab: drag-drop a .ics file → events for the selected month appear in list
8. Calendar tab: import a second .ics → events replace previous import, not duplicate
9. Overview tab: categories with budget show progress bars with correct colors (green under 80%, amber 80-100%, red over)
10. Overview tab: overspent category shows bar at 100% + "OVER by X" in red
11. Overview tab: total summary shows correct budgeted/spent/remaining
12. Dashboard: add BudgetStatus widget → shows current month's budget status
13. Dashboard: BudgetStatus widget with no budget set → shows "No budget set" message

## Acceptance Criteria

- `cargo test -p accountant-core` passes — all new DB methods and iCal parser tests green
- `cargo build` succeeds — Tauri compiles with all 6 new IPC commands
- Budget page is accessible from sidebar and renders without console errors
- Month selector navigates correctly and reloads data
- Category budgets persist across page navigations and app restarts
- Planned expenses can be added and deleted
- .ics file import parses all three DTSTART formats and filters to selected month
- Budget Overview shows per-category progress bars with correct color thresholds (under/approaching/over)
- Overspend is clearly visible with red indicators
- BudgetStatus dashboard widget displays current month summary
- All UI follows existing dark theme (gray-950/900/800, emerald-400/500 accents)
- All Svelte components use Svelte 5 syntax ($state, $derived, $props, onclick)
