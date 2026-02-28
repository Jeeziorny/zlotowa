# Task 71 — Shared Autocomplete Component

Item: #4 (fragile blur-timeout category autocomplete).

## Problem

Category autocomplete across `AddExpense`, `ExpenseTable`, `RulesTable`, and `ReviewClassified` uses a fragile pattern: dropdown closes on `blur` with `setTimeout(150ms)` to let `onmousedown` fire first. This races and can fail on slower devices or with keyboard navigation. The logic is duplicated in 4+ places.

## Solution

Create a reusable `Autocomplete.svelte` component that replaces all instances.

### Component API

```svelte
<Autocomplete
  value={category}
  options={categories}
  placeholder="Category"
  onselect={(val) => category = val}
/>
```

### Props
- `value` — current text value (bindable)
- `options` — array of strings to filter/suggest
- `placeholder` — input placeholder
- `onselect` — callback when an option is selected
- `class` — optional extra CSS classes for the wrapper
- `inputClass` — optional extra CSS classes for the input

### Internal behavior

1. Track `open` state explicitly with `$state()` — no blur timeout.
2. On input focus → open dropdown (if options exist).
3. On input blur → check `event.relatedTarget`. If relatedTarget is inside the dropdown container (use a wrapper ref), keep open. If outside, close.
4. Dropdown options use `tabindex="-1"` so they can receive focus.
5. Option click → set value, close dropdown, refocus input.
6. Keyboard navigation:
   - Arrow Down/Up — highlight next/previous option
   - Enter — select highlighted option (or first if none highlighted)
   - Escape — close dropdown without selecting
7. Highlighted option uses `bg-gray-700` visual indicator.
8. Filter options as user types (case-insensitive substring match).

### Styling

Match existing input + dropdown styles:
- Input: `bg-gray-800 border border-gray-700 rounded-lg text-sm focus:border-amber-500` (per task #82 palette)
- Dropdown: `absolute z-30 w-full mt-1 bg-gray-800 border border-gray-700 rounded-lg shadow-lg max-h-48 overflow-y-auto`
- Option: `px-4 py-2 text-sm text-gray-200 hover:bg-gray-700 cursor-pointer`

### Migration

Replace the duplicated autocomplete logic in:
1. `AddExpense.svelte` — category input section
2. `ExpenseTable.svelte` — inline edit category cell
3. `RulesTable.svelte` — inline edit category cell
4. `ReviewClassified.svelte` — per-expense category input

Remove all `setTimeout`-based blur workarounds.

## Files

| File | Action |
|------|--------|
| `src/lib/Autocomplete.svelte` | Create — shared component |
| `src/lib/AddExpense.svelte` | Modify — use Autocomplete |
| `src/lib/expense-list/ExpenseTable.svelte` | Modify — use Autocomplete |
| `src/lib/rules/RulesTable.svelte` | Modify — use Autocomplete |
| `src/lib/bulk-upload/ReviewClassified.svelte` | Modify — use Autocomplete |

## Verification
1. `npm run dev` — all 4 consumers render category autocomplete
2. Click an option → value set, dropdown closes, no race condition
3. Arrow keys navigate options, Enter selects, Escape closes
4. Focus + type to filter → dropdown narrows options
5. Tab away → dropdown closes cleanly
6. No `setTimeout` calls remain in any consumer for blur handling
