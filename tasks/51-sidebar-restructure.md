# 51 — Sidebar Restructuring & Expenses Page Consolidation

## Problem

The sidebar has 7 nav items + Settings, which is cluttered. Add Expense, Bulk Upload, and Expense List are three separate pages for one concept ("Expenses"). Dashboard doesn't need its own nav slot if the logo is clickable.

## Changes

### Sidebar

- **Remove** "Add Expense", "Expense bulk upload", "Title Cleanup" from nav items
- **Logo → Dashboard:** Make "4ccountant" logo clickable, navigates to Dashboard. Remove Dashboard from nav items list.
- **New sidebar:** Expenses, Categories, Budget (3 items + Settings at bottom)

### Expenses Page Consolidation

- **Toolbar buttons:** Add "+ Add Expense" and "Upload CSV" buttons in the Expenses page header (top-right area, next to existing search/filters)
- **Sub-routing:** Clicking toolbar buttons switches to inline sub-views (Add form or Bulk Upload wizard) within the Expenses page. A "Back to list" link returns to the expense list.
- **Component reuse:** The existing `AddExpense.svelte` and `BulkUpload.svelte` are rendered inside the Expenses page instead of as standalone pages — no rewrite needed.

### Routing Updates

- Remove standalone routes for `"add"`, `"bulk"`, `"cleanup"` from `App.svelte`
- Add sub-view state within ExpenseList (e.g. `subView = $state("list" | "add" | "bulk" | "cleanup")`)

## Scope

- `src/lib/Sidebar.svelte` — reduce nav items, make logo clickable
- `src/App.svelte` — remove standalone routes for add/bulk/cleanup
- `src/lib/ExpenseList.svelte` — add toolbar buttons, sub-view state for add/bulk/cleanup
