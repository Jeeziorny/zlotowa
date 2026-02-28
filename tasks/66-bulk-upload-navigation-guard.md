# Task 66 — Bulk Upload Navigation Guard

## Problem

When the user is mid-way through the bulk upload wizard (any step after "input") and navigates away — e.g. clicks Settings in the sidebar — all wizard progress is silently lost. The component unmounts and all `$state()` variables reset.

## Solution

Add a confirmation popup when attempting to navigate away from an active bulk upload session. "You'll lose your upload progress. Leave anyway?"

## Scope

### 1. Track whether bulk upload is "dirty"

BulkUpload.svelte already has a `step` variable. If `step !== "input"` and `step !== "done"`, the wizard has in-progress work. Expose this as a callback or reactive signal to the parent.

### 2. Intercept navigation in App.svelte

Before changing `currentPage`, check if the expenses page has an active bulk upload. If so, show a confirmation dialog instead of navigating immediately. Only proceed if the user confirms.

### 3. Also guard the "Back to Expenses" button

ExpenseList.svelte has a back button that switches `subView` from `"bulk"` to `"list"`. This also destroys the wizard. Guard it the same way.

### 4. Confirmation UI

Use the app's existing dark-themed modal pattern (gray-900 card, emerald buttons) — consistent with delete confirmation modals already in the codebase.

## Files

- `src/lib/BulkUpload.svelte` — expose dirty state
- `src/lib/ExpenseList.svelte` — guard subView switch
- `src/App.svelte` — guard page navigation
- Possibly a small shared `ConfirmDialog.svelte` if one doesn't already exist (check existing modals first)

## Out of Scope

- Persisting/restoring wizard state across navigation
- Browser beforeunload (Tauri desktop app, not needed)
