# Task 86: Sidebar Button — Promote Import Over Single Add

## Problem

The sidebar has a prominent "+ Add" button that navigates to the single-expense form (`expenses:add`). But the more powerful and frequently used workflow is bulk upload (CSV import). The single-add form is a secondary action that doesn't warrant top-level sidebar placement, while the bulk upload — which handles the primary data ingestion path — is buried as a sub-tab inside the Expenses page.

## Solution

Replace the sidebar "+ Add" button with an "+ Import" button that navigates directly to the bulk upload flow (`expenses:bulk`). Demote single-add to a secondary action accessible from within the Expenses list view.

### Sidebar

- Rename button from "+ Add" to "+ Import" (or "+ Upload")
- Change navigation target from `expenses:add` to `expenses:bulk`
- Update keyboard shortcut hint if visible (Cmd+U is already bound to bulk upload)

### Expenses List View

- Add a small "+ Add manually" text button or link in the expenses list header/toolbar area, next to existing controls
- This replaces the sidebar as the entry point for single-expense creation
- Keep the existing `Cmd+N` shortcut working for quick single-add

### Keyboard Shortcuts

- No changes needed — `Cmd+N` (single add) and `Cmd+U` (bulk upload) already exist
- Consider whether the shortcut help overlay needs reordering to emphasize `Cmd+U`

## Files to Modify

- `src/lib/Sidebar.svelte` — rename button label, change `onnavigate("expenses:add")` to `onnavigate("expenses:bulk")`
- `src/lib/expense-list/SearchFilterBar.svelte` — add "+ Add manually" button that switches to add sub-view
- `src/App.svelte` — add `expenses:bulk` handling in `handleNavigate` (similar to existing `expenses:add`)
