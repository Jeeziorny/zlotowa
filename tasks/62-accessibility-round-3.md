# 62 — Accessibility Round 3

## Problem

Remaining accessibility gaps after rounds 1 (task 33) and 2 (task 46).

### Findings

1. **Invisible keyboard actions** — `src/lib/expense-list/ExpenseTable.svelte:197-218`: Edit/Delete buttons are `opacity-0 group-hover:opacity-100` with no `:focus-within` or `:focus-visible` fallback. Keyboard users tab to invisible buttons.

2. **Missing aria-labels on icon buttons** — `ExpenseTable.svelte:149-166` (Save/Cancel edit buttons), `PaginationBar.svelte:33-55` (Prev/Next buttons) have `title` but no `aria-label`.

3. **Missing aria-label on select-all checkbox** — `ExpenseTable.svelte:82-86`: Screen readers just announce "checkbox" with no context.

4. **Unlabeled inputs** — `SearchFilterBar.svelte:36` (search input), `Categories.svelte:162` (new category input), `Categories.svelte:191` (search categories input) have no `<label>` or `aria-label`.

5. **Decorative SVGs not hidden** — `ExpenseList.svelte:159-163`: "Back to Expenses" button's SVG arrow has no `aria-hidden="true"`.

6. **Unnecessary a11y suppression** — `BudgetCreator.svelte:233`: `<!-- svelte-ignore a11y_no_static_element_interactions -->` on ICS drop zone that already has `role="button"`, `tabindex`, `aria-label`, and `onkeydown`. The comment is unnecessary.

## Scope

- Add `group-focus-within:opacity-100` to ExpenseTable action buttons
- Add `aria-label` to all icon-only buttons (Save, Cancel, Prev, Next)
- Add `aria-label="Select all expenses"` to the select-all checkbox
- Add `aria-label` or visually-hidden `<label>` to unlabeled inputs
- Add `aria-hidden="true"` to decorative SVGs
- Remove unnecessary `svelte-ignore` comment on BudgetCreator drop zone
