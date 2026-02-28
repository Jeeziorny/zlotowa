# Task 80a — Input Validation: Amounts

Split from original task 80. Part 80b (character limits with graduated counters) deferred for later.

Merges item #18 (no amount validation).

## Problem

Amount inputs accept zero and negative values with no warning or guard.

## Solution

### Bulk Upload — `Math.abs()` at save time

Bulk upload must preserve negative signs during the review step because sign-based income detection relies on them. At save time (when the user clicks "Save"), apply `Math.abs()` to every amount before sending to backend.

**No error state needed** — amounts are silently normalized.

### AddExpense — block `<= 0`

- Before submit, validate `amount > 0`.
- If invalid, show inline error: `"Amount must be greater than zero"` in `text-red-400 text-xs mt-1` below the input.
- Form does not submit until corrected.

### ExpenseTable inline edit — block `<= 0`

- Validate before save. If `amount <= 0`, show inline error in the edit row.
- Same message and styling as AddExpense.

### BudgetCreator — already handled

Budget category amounts already have `min="0"` and the UI enforces non-negative. No changes needed.

## Files

| File | Action |
|------|--------|
| `src/lib/AddExpense.svelte` | Modify — amount `> 0` validation with inline error |
| `src/lib/expense-list/ExpenseTable.svelte` | Modify — amount `> 0` validation with inline error |
| `src/lib/bulk-upload/ReviewClassified.svelte` | Modify — `Math.abs()` on amounts at save time |

## Verification

1. AddExpense: enter `0` or `-5` → inline error, form doesn't submit
2. AddExpense: enter `0.01` → submits normally
3. ExpenseTable: edit amount to `0` → inline error, save blocked
4. Bulk upload: CSV with `-50` amount → shows `-50` during review → saves as `50`
5. BudgetCreator: no regression, still accepts `0` and positive values
