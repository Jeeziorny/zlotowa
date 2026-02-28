# 59 — Bulk Upload Error Handling

## Problem

The BulkUpload wizard silently swallows backend errors at two critical steps, leaving users stranded with no feedback.

### Locations

1. **`src/lib/BulkUpload.svelte:28`** — `handleFileInput` calls `invoke("preview_csv", ...)` with no `try/catch`. A backend parse failure (invalid CSV) becomes an unhandled promise rejection.
2. **`src/lib/BulkUpload.svelte:49-53`** — `handleMapping` has `try/finally` but no `catch`. Classification failures leave the user on the column-mapping step with no error message. The `classifying` spinner disappears but no feedback is shown.
3. **`src/lib/BulkUpload.svelte:80-81`** — `handleSave` calls `invoke("bulk_save_expenses", ...)` without a `try/catch`. The outer `doSave` in `ReviewClassified.svelte:67` has a try/catch, but `handleSave` itself doesn't guard against rejection before setting `step = "done"`.

## Scope

- Add `try/catch` with user-visible error messages to `handleFileInput` and `handleMapping`
- Add error state variable and display it in the UI (e.g. red banner above the current step)
- Ensure `handleSave` properly propagates errors
- Test: upload a malformed CSV and verify user sees a clear error message at each step
