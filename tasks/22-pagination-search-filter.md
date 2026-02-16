# Task 22: Pagination, Search & Filtering for Expense List

**Track:** Full-stack — ExpenseList enhancement (backend + frontend)
**Priority:** HIGH
**Blocked by:** Task 20 (ExpenseList UI will be restructured there; build on that)
**Blocks:** nothing

## Problem

`ExpenseList.svelte` loads all expenses at once via `get_expenses` → `get_all_expenses()`. There is no pagination, no search, and no filtering. With 1000+ expenses, the table will be slow to render and impossible to navigate.

Users have no way to find a specific expense by title, filter by category, narrow by date range, or filter by amount range.

## Current State

### Database (`crates/core/src/db.rs`)

- `get_all_expenses()` (line 142-170): `SELECT id, title, amount, date, category, classification_source FROM expenses ORDER BY date DESC`
- No parameters for limit, offset, search, or filtering
- Indexes exist: `idx_expenses_date`, `idx_expenses_dup(title, amount, date)`, `idx_expenses_category`, `idx_expenses_source`

### Tauri IPC (`src-tauri/src/lib.rs`)

- `get_expenses` (line 86-89): takes no parameters, calls `get_all_expenses()`

### Frontend (`src/lib/ExpenseList.svelte`)

- `expenses = await invoke("get_expenses")` on mount (line 20) — loads all
- Static `{#each expenses as expense}` loop (line 167) — no pagination controls
- No search input, no filter dropdowns, no date range picker

## Scope

### 1. Add `ExpenseQuery` struct (`crates/core/src/models.rs`)

```rust
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ExpenseQuery {
    pub search: Option<String>,           // LIKE %search% on title
    pub category: Option<String>,         // exact match (or "uncategorized" for NULL)
    pub date_from: Option<NaiveDate>,     // >= date_from
    pub date_to: Option<NaiveDate>,       // <= date_to
    pub amount_min: Option<f64>,          // >= amount_min
    pub amount_max: Option<f64>,          // <= amount_max
    pub limit: Option<i64>,              // page size (default 50)
    pub offset: Option<i64>,             // pagination offset
}
```

### 2. Add `ExpenseQueryResult` struct (`crates/core/src/models.rs`)

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpenseQueryResult {
    pub expenses: Vec<Expense>,
    pub total_count: i64,    // total matching rows (before limit/offset)
}
```

### 3. Add `query_expenses` method (`crates/core/src/db.rs`)

```rust
pub fn query_expenses(&self, query: &ExpenseQuery) -> Result<ExpenseQueryResult, DbError>
```

Implementation:
- Build a dynamic SQL query with WHERE clauses based on which `ExpenseQuery` fields are `Some`
- `search`: `WHERE title LIKE '%' || ?N || '%'` (case-insensitive via SQLite COLLATE NOCASE or LOWER)
- `category`: `WHERE category = ?N` or `WHERE category IS NULL` for uncategorized
- `date_from`/`date_to`: `WHERE date >= ?N` / `WHERE date <= ?N`
- `amount_min`/`amount_max`: `WHERE amount >= ?N` / `WHERE amount <= ?N`
- Run a `SELECT COUNT(*)` with the same WHERE for `total_count`
- Append `ORDER BY date DESC LIMIT ?N OFFSET ?N`
- Default limit: 50, default offset: 0

Keep `get_all_expenses()` as-is — it's used by dashboard widgets, export, and other callers that need all data.

### 4. Add Tauri IPC command (`src-tauri/src/lib.rs`)

```rust
#[tauri::command]
fn query_expenses(state: State<AppState>, query: ExpenseQuery) -> Result<ExpenseQueryResult, String>
```

Register in `invoke_handler![]`. Keep `get_expenses` for backward compatibility (dashboard, export).

### 5. Update `ExpenseList.svelte` frontend

**Search bar:**
- Text input at top of page with magnifying glass icon
- Debounced (300ms) — triggers re-query on each keystroke pause
- Searches by title (LIKE match)

**Filter bar (below search):**
- Category dropdown: "All categories" + list from `get_categories`, + "Uncategorized"
- Date range: two date inputs (From / To)
- Amount range: two number inputs (Min / Max)
- "Clear filters" button when any filter is active

**Pagination controls (below table):**
- Shows "Showing X-Y of Z expenses"
- Previous / Next buttons
- Page size selector: 25 / 50 / 100
- Keyboard: left/right arrows for page navigation

**Data flow:**
- Replace `get_expenses` call with `query_expenses`
- All filter/search/pagination state → reactive → triggers `query_expenses` on change
- `$effect()` watches query params and re-fetches
- Loading state while query is in-flight

## Files to Change

| File | Change |
|---|---|
| `crates/core/src/models.rs` | Add `ExpenseQuery` and `ExpenseQueryResult` structs |
| `crates/core/src/db.rs` | Add `query_expenses` method with dynamic SQL |
| `src-tauri/src/lib.rs` | Add `query_expenses` IPC command, register in handler, add imports |
| `src/lib/ExpenseList.svelte` | Add search bar, filter controls, pagination, switch to `query_expenses` |

## Test Scenarios

### Backend (Rust unit tests)

1. `query_expenses` with empty query returns all expenses (default behavior)
2. `query_expenses` with `search: "coffee"` returns only expenses with "coffee" in title (case-insensitive)
3. `query_expenses` with `category: "Food"` returns only Food expenses
4. `query_expenses` with `category` set to uncategorized returns expenses with NULL category
5. `query_expenses` with `date_from` and `date_to` returns expenses in range
6. `query_expenses` with `amount_min: 10.0, amount_max: 50.0` returns expenses in range
7. `query_expenses` with `limit: 2, offset: 0` returns first 2, `total_count` reflects all matching
8. `query_expenses` with `limit: 2, offset: 2` returns next 2 (page 2)
9. Combined filters: search + category + date range — returns intersection
10. `total_count` is correct regardless of limit/offset

### Frontend (manual UI tests)

1. Page loads with first 50 expenses and pagination showing "1-50 of N"
2. Type in search bar — results filter after debounce, pagination resets to page 1
3. Select a category from dropdown — table filters, count updates
4. Set date range — only expenses in range appear
5. Set amount range — only expenses in range appear
6. Click Next — page 2 loads, Previous becomes enabled
7. Change page size to 100 — table shows 100 rows, pagination adjusts
8. Click "Clear filters" — all filters reset, full list returns
9. Combine search + category + date range — shows intersection
10. With filters active, pagination still works correctly (total_count matches)

## Acceptance Criteria

- Expense list paginates at 50 rows by default with page size selector (25/50/100)
- Title search is debounced (300ms), case-insensitive, partial match
- Filtering works by category, date range, and amount range
- Filters can be combined (intersection logic)
- Pagination reflects filtered results (correct total count, page boundaries)
- "Clear filters" resets all filters and search
- Existing `get_expenses` / `get_all_expenses` unchanged (backward compatible)
- Loading indicator while query runs
- UI follows dark theme conventions
- Svelte 5 syntax used throughout
