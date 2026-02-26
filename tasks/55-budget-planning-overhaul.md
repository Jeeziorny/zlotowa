# 55 — Budget Planning Overhaul

## Problem

Budget Planning has several UX issues: native date pickers (inconsistent with rest of app), missing validation on category amounts, an unused Planned Expenses tile, no way to browse historical/future budgets, and a calendar tab that needs to be disabled until redesigned.

## Changes

### 1. DatePicker in BudgetCreator

**File:** `src/lib/budget/BudgetCreator.svelte`

- Import existing `DatePicker` from `../DatePicker.svelte`
- Replace both `<input type="date">` (lines 129-135, 139-145) with `<DatePicker>` components
- Pattern: `<DatePicker id="budget-start-date" value={startDate} onchange={(d) => startDate = d} />`
- Wrap each in width-constrained container to preserve horizontal flex layout

### 2. Category Amount Validation

**File:** `src/lib/budget/BudgetCreator.svelte`

- Add `categoryError` state variable
- Replace step 2→3 "Next" `onclick` (line 238) with validation function
- Block transition if any category has `amount <= 0`, show error listing invalid categories
- Keep existing `c.amount > 0` filter in `createBudget()` as safety net

### 3. Remove Planned Expenses — Full Cleanup

**Frontend:**
- `src/lib/budget/BudgetOverview.svelte` — Remove props (`plannedExpenses`, `totalPlanned`), state vars (`peTitle`, `peAmount`, `peDate`, `peCategory`, `peError`, `deletePlannedError`), functions (`addPlannedExpense`, `deletePlanned`), template (lines 304-413)
- `src/lib/BudgetPlanning.svelte` — Remove `plannedExpenses`/`totalPlanned` props from BudgetOverview

**Backend IPC (`src-tauri/src/lib.rs`):**
- Remove `PlannedExpenseInput` struct
- Remove `planned_expenses` and `total_planned` from `BudgetSummaryOutput`
- Remove planned_expenses logic from `build_budget_summary()`
- Remove `add_planned_expense` and `delete_planned_expense` command functions
- Remove both from `invoke_handler!` registration
- Remove `PlannedExpense` from imports
- Update tests that reference planned expenses

**Core crate (`crates/core/src/`):**
- `models.rs` — Remove `PlannedExpense` struct (lines 130-138)
- `db.rs` — Remove `insert_planned_expense()`, `delete_planned_expense()`, `get_planned_expenses()`
- `db.rs` — Remove `DELETE FROM planned_expenses` from `delete_budget()` transaction
- `db.rs` — Add `DROP TABLE IF EXISTS planned_expenses` to `migrate()`
- `db.rs` — Remove `CREATE TABLE IF NOT EXISTS planned_expenses` and its index from `migrate()`
- `db.rs` — Remove tests: `planned_expenses_crud`, `planned_expense_delete_nonexistent_fails`, planned_expenses parts of `budget_delete_cascades`

### 4. Calendar Tab Always Disabled

**File:** `src/lib/BudgetPlanning.svelte`

- Change `{ id: "calendar", label: "Calendar", disabled: !activeBudget }` → `disabled: true`
- Calendar Events component stays (unreachable, full cleanup in task 56)

### 5. Delete Budget Confirmation — No Changes

Already implemented with modal (BudgetOverview.svelte lines 416-452). Confirmed working.

### 6. Budget Navigation (Prev/Next Arrows)

**6a. Backend — `get_all_budgets()` method**
- `crates/core/src/db.rs` — Add `get_all_budgets() -> Result<Vec<Budget>>` returning all budgets `ORDER BY start_date`
- Add tests: `budget_get_all_sorted`, `budget_get_all_empty`

**6b. Backend — IPC command**
- `src-tauri/src/lib.rs` — Add `#[tauri::command] fn list_budgets(state) -> Result<Vec<Budget>, String>`
- Register in `invoke_handler!` macro

**6c. Frontend — Rewrite BudgetPlanning.svelte**
- Replace `get_active_budget_summary` with `list_budgets` to load all budgets
- Track `allBudgets` (sorted array) + `currentIndex` for navigation
- Default to active budget (date spans today), fallback to newest
- Add `goPrev()`/`goNext()` loading summary via existing `get_budget_summary(budgetId)`
- Add prev/next nav bar above BudgetOverview: `< Prev  |  1 / N  |  Next >`
- "Create +" tab no longer disabled when budget exists (overlap check prevents conflicts)
- Handle deletion: fall back to newest remaining budget
- Handle creation: reload list, switch to new budget

## Scope

| File | Changes |
|------|---------|
| `crates/core/src/db.rs` | Add `get_all_budgets()`, drop planned_expenses table, remove planned_expenses methods/tests |
| `crates/core/src/models.rs` | Remove `PlannedExpense` struct |
| `src-tauri/src/lib.rs` | Add `list_budgets` IPC, remove planned expense IPC commands/structs, update summary |
| `src/lib/BudgetPlanning.svelte` | Navigation logic, tab changes, remove planned expense props |
| `src/lib/budget/BudgetCreator.svelte` | DatePicker swap, amount validation |
| `src/lib/budget/BudgetOverview.svelte` | Remove Planned Expenses tile/state/functions |

## Verification

1. `cargo test -p accountant-core` — new `get_all_budgets` tests pass, no planned_expenses references
2. `cargo test -p accountant-app` — `list_budgets` IPC test passes, no planned_expenses references
3. `npm run tauri dev` — manual testing:
   - Create budget: DatePicker works, amount validation blocks categories with 0
   - Overview: no Planned Expenses tile, Calendar tab grayed out
   - Delete budget: confirmation modal still works
   - Create 2+ non-overlapping budgets, navigate between them with arrows
   - Delete a budget while viewing it, verify fallback to nearest budget
