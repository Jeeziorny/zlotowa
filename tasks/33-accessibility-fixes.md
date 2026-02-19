# 33 — Accessibility Fixes

## Problem

Several interactive elements lack keyboard support:

1. **Modals don't dismiss on Escape** — `Categories.svelte:289,330`, `TitleCleanup.svelte:381`, `ExpenseList.svelte:662,695`. Backdrops respond to click only.
2. **Drop zone not keyboard-accessible** — `BulkUpload.svelte:407-413`. Has `role="button"` and `tabindex="0"` but no `onkeydown` handler for Enter/Space to trigger file picker.
3. **Sortable table headers lack keyboard affordance** — `Categories.svelte:219-230`. Clickable `<th>` elements have no `onkeydown` for Enter/Space.

## Scope

- Add `onkeydown` handler to all modal backdrops that closes on Escape
- Add `onkeydown` to drop zone that triggers file input on Enter/Space
- Add keyboard activation to sortable table headers (or use `<button>` inside `<th>`)
