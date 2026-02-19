# 46 — Accessibility Round 2

## Problem

Task 33 fixed escape-to-close, keyboard drop zones, and sortable headers. This task covers remaining a11y gaps found in the audit.

### Missing `aria-label` on icon-only buttons

1. **`src/lib/Dashboard.svelte:117-139`** — Move left, Move right, Remove widget buttons have `title` but no `aria-label`
2. **`src/lib/BulkUpload.svelte:356-359`** — Category remove button (`×`) — no `aria-label` or `title`
3. **`src/lib/BulkUpload.svelte:525-527`** — LLM warning dismiss button (`×`) — no `aria-label`
4. **`src/lib/Categories.svelte:285`** — Delete category button (`✕`) — has `title` but no `aria-label`
5. **`src/lib/TitleCleanup.svelte:302-309`** — Edit (✎) and Delete (✕) rule buttons — have `title` but no `aria-label`
6. **`src/lib/budget/MonthSelector.svelte:26-42`** — Previous/Next month buttons — no `aria-label` or `title`
7. **`src/lib/DatePicker.svelte:95-106`** — Previous/Next month buttons — no `aria-label`

### Missing label associations

8. **`src/lib/AddExpense.svelte:95`** — Date label has no `for` attribute
9. **`src/lib/BudgetOverview.svelte:344-393`** — Planned expense form labels (Title, Amount, Date, Category) no `for` attributes
10. **`src/lib/BudgetCreator.svelte:128-143`** — Start/End Date labels no `for` attributes
11. **`src/lib/TitleCleanup.svelte:192-201`** — Pattern and Replacement labels no `for` attributes

### Missing dialog semantics

12. **`src/lib/ExpenseList.svelte:662-726`** — Delete and batch delete modals missing `aria-modal="true"` and `aria-labelledby`
13. **`src/lib/Categories.svelte:304,344`** — Delete and merge modals same issue
14. **`src/lib/TitleCleanup.svelte:408`** — Delete rule modal same issue

### Missing keyboard handler

15. **`src/lib/CalendarEvents.svelte:68-76`** — Drop zone has `role="button"` and `tabindex="0"` but no `onkeydown` handler for Enter/Space activation

## Scope

- Add `aria-label` to all icon-only buttons listed above
- Add `for` attributes (or wrap inputs in labels) for all form labels
- Add `aria-modal="true"` and `aria-labelledby` to all dialog modals
- Add `onkeydown` handler to CalendarEvents drop zone
