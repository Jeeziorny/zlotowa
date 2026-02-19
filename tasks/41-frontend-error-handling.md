# 41 — Frontend Error Handling

## Problem

Multiple `invoke()` calls catch errors with only `console.error`, giving users no feedback when operations fail. Destructive operations (delete, save) are especially bad — the user thinks the action succeeded.

### Locations

1. **`src/lib/ExpenseList.svelte:237-241`** — `doDelete()` catches with `console.error` only
2. **`src/lib/ExpenseList.svelte:266-271`** — `doBatchDelete()` catches with `console.error` only
3. **`src/lib/ExpenseList.svelte:71`** — empty `catch (_) {}` on category load — filter dropdown silently empty
4. **`src/lib/BudgetOverview.svelte:99-106`** — `deletePlanned()` catches with `console.error` only
5. **`src/lib/BudgetOverview.svelte:109-118`** — `deleteBudget()` catches with `console.error` only — uses native `confirm()` which may not render in Tauri
6. **`src/lib/CalendarEvents.svelte:52-55`** — `updateAmount()` catches with `console.error` only

### Also: TitleCleanup logic bug

**`src/lib/TitleCleanup.svelte:80-83`** — `confirmDelete()` sets `deleteTarget = null` on line 81, then reads `deleteTarget?.id` on line 82 to decide whether to clear the preview. Since `deleteTarget` is already null, the preview-clearing check never matches. This is a logic bug.

## Scope

- Add user-visible error feedback (inline error message or toast) for all `catch` blocks listed above
- Replace native `confirm()` in `deleteBudget()` with a custom confirmation modal (consistent with rest of UI)
- Fix TitleCleanup bug: read `deleteTarget.id` before setting it to null
- Consider a shared `handleError(error, fallbackMessage)` utility to reduce duplication
