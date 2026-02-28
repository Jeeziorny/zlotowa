# Task 76 — Generic Confirmation Modal with Focus Trap

Merges items: #9 (three duplicate modals), #13 (focus trapping missing).

## Problem

Three near-identical modal components exist: `DeleteConfirmModal`, `BatchDeleteModal`, `RuleDeleteModal`. They share the same overlay, dialog, button, and error patterns with only the content differing. ~150 lines of duplication.

Additionally, all modals lack focus trapping — Tab key can escape the modal overlay and reach elements behind it.

## Solution

### Generic ConfirmModal

Create `ConfirmModal.svelte` with these props:
- `title` — heading text (required)
- `confirmLabel` — confirm button text (default: `"Delete"`)
- `confirmStyle` — `"danger"` (bg-red-600) or `"primary"` (bg-emerald-600), default `"danger"`
- `onconfirm` — async callback, modal manages loading state internally
- `onclose` — close callback
- `children` — Svelte snippet for body content (replaces slot)

### Internal behavior

1. Overlay: `fixed inset-0 bg-black/60 z-50`, click-outside closes (when not loading).
2. Dialog: `bg-gray-900 border border-gray-800 rounded-xl p-6 max-w-sm w-full mx-4`.
3. On confirm click: set internal `loading = true`, call `await onconfirm()`, handle errors internally (show `bg-red-900/50 text-red-400` error bar). Set `loading = false` on completion.
4. Escape key closes (when not loading).
5. Buttons disabled during loading. Confirm button shows spinner.
6. Cancel button: `bg-gray-800 hover:bg-gray-700`.

### Focus trap action

Create a `focusTrap.js` Svelte action (`use:focusTrap`):
1. On mount: query all focusable elements inside the node (`button:not([disabled]), input, select, textarea, [tabindex]:not([tabindex="-1"])`).
2. Save `document.activeElement` as the previously focused element.
3. Focus the first focusable element (or the container itself).
4. Intercept `keydown` on the node:
   - Tab on last focusable → wrap to first
   - Shift+Tab on first focusable → wrap to last
5. On destroy: restore focus to the previously saved element.

Apply `use:focusTrap` inside `ConfirmModal`.

### Migration

Refactor all three existing modals to use `ConfirmModal`:

**DeleteConfirmModal** (expenses):
```svelte
<ConfirmModal title="Delete expense?" onconfirm={handleDelete} onclose>
  <p>"{expense.title}" — ${expense.amount} on {expense.date}</p>
  <p class="text-sm text-gray-500 mt-2">This action cannot be undone.</p>
</ConfirmModal>
```

**BatchDeleteModal** (expenses):
```svelte
<ConfirmModal title="Delete {count} expenses?" confirmLabel="Delete All" onconfirm={handleBatchDelete} onclose>
  <p>This will permanently delete {count} expense{s}.</p>
</ConfirmModal>
```

**RuleDeleteModal** (rules):
```svelte
<ConfirmModal title="Delete rule?" onconfirm={handleDelete} onclose>
  <p>Pattern: <code class="font-mono">{rule.pattern}</code></p>
  <p>Category: {rule.category}</p>
</ConfirmModal>
```

Also apply to: category delete modal (in Categories.svelte), budget delete modal (in BudgetOverview.svelte), and the ConfirmLeaveModal (with `confirmStyle="danger"` and `confirmLabel="Leave"`).

After migration, delete the three original modal files.

### Other modals needing focus trap

Apply `use:focusTrap` to non-ConfirmModal dialogs too:
- Widget picker modal (Dashboard.svelte)
- Config dialog (Dashboard.svelte)
- Merge modal (Categories.svelte)

## Files

| File | Action |
|------|--------|
| `src/lib/ConfirmModal.svelte` | Create — generic modal component |
| `src/lib/actions/focusTrap.js` | Create — reusable Svelte action |
| `src/lib/expense-list/DeleteConfirmModal.svelte` | Delete — replaced by ConfirmModal |
| `src/lib/expense-list/BatchDeleteModal.svelte` | Delete — replaced by ConfirmModal |
| `src/lib/rules/RuleDeleteModal.svelte` | Delete — replaced by ConfirmModal |
| `src/lib/ExpenseList.svelte` | Modify — use ConfirmModal inline |
| `src/lib/Rules.svelte` | Modify — use ConfirmModal inline |
| `src/lib/Categories.svelte` | Modify — use ConfirmModal + focusTrap |
| `src/lib/budget/BudgetOverview.svelte` | Modify — use ConfirmModal |
| `src/lib/ConfirmLeaveModal.svelte` | Modify — use ConfirmModal or focusTrap |
| `src/lib/Dashboard.svelte` | Modify — add focusTrap to widget picker + config dialog |

## Verification
1. All delete confirmations render via ConfirmModal
2. Tab key cycles within modal — cannot reach background elements
3. Escape closes modal, focus returns to trigger element
4. Confirm button shows spinner during async operation
5. Error messages display inside modal on failure
6. Three old modal files deleted, no imports remain
