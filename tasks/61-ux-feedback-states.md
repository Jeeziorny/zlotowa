# 61 — UX Feedback States

## Problem

Several pages lack loading indicators, allow double-submit, or show stale time-based data.

### Findings

1. **Double-submit on Add Expense** — `src/lib/AddExpense.svelte:192-198`: Submit button has no loading/disabled state during async save. Users can click twice and create duplicates.

2. **Sticky success message** — `src/lib/AddExpense.svelte:200-208`: "Expense added successfully!" message never auto-clears. It persists while the user fills out the next expense.

3. **Debounce timer leak** — `src/lib/AddExpense.svelte:34`: `debounceTimer` is a plain `let` with no `onDestroy` cleanup. Timer can fire after component unmount. Compare with `SearchFilterBar.svelte:18-28` which correctly cleans up.

4. **No loading indicators** — `src/lib/Dashboard.svelte:118-122`: Empty area shown while data loads. Same issue in `Categories.svelte` (flashes "No categories yet") and `TitleCleanup.svelte` (flashes "No cleanup rules").

5. **Stale date computations** — `src/lib/DatePicker.svelte:16-17`: `today`/`todayStr` computed once at mount. If app stays open past midnight, "Today" button highlights yesterday. Same in `BiggestExpense.svelte:4` for month boundary.

6. **Timeout without cleanup** — `src/lib/budget/BudgetOverview.svelte:46`: `setTimeout(() => saveMsg = "", 2000)` has no `onDestroy` cleanup.

7. **Fire-and-forget promises** — `src/lib/ExpenseList.svelte:86-100`: `handleSearch`, `handleFilterChange`, `changePageSize` call `fetchExpenses()` without `await` or `.catch()`.

## Scope

- Add `saving` state to AddExpense, disable button during submit
- Auto-clear success/error messages after 3 seconds or on next input
- Add `onDestroy` cleanup for debounce timer in AddExpense
- Add loading skeletons/spinners to Dashboard, Categories, TitleCleanup
- Fix DatePicker/BiggestExpense to recompute `today` reactively
- Add cleanup for BudgetOverview timeout
- Add `.catch()` to fire-and-forget `fetchExpenses()` calls
