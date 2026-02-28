# Task 72 — Table Interaction Consistency

Merges items: #5 (inline edit discoverability), #10 (table styling varies).

## Problem

Three tables have inconsistent interaction patterns:
- `ExpenseTable`: hover-reveal SVG icons (edit pencil, delete trash)
- `RulesTable`: hover-reveal SVG icons (slightly different implementation)
- `Categories`: always-visible `✕` text button + click-to-rename (different paradigm entirely)

Users must learn three mental models. And in all three tables, there's no visual hint that rows are editable until you hover.

## Solution

### Standardize action column

All three tables should use the same pattern:
- **Action column:** Last column, right-aligned, fixed width `w-24`
- **Visibility:** `opacity-0 group-hover:opacity-100 group-focus-within:opacity-100 transition-opacity`
- **Icons:** Same SVG icon set — pencil (edit), trash (delete) at `w-4 h-4`
- **Colors:** `text-gray-500 hover:text-gray-300` (edit), `text-gray-500 hover:text-red-400` (delete)
- **Row hover:** `hover:bg-gray-800/30` on all three tables

### Categories table changes

- Remove click-to-rename. Replace with pencil icon → inline edit (same pattern as ExpenseTable).
- Replace `✕` text button with trash SVG icon matching other tables.
- Edit mode: same row-replacing-with-inputs pattern as ExpenseTable.

### Editability hint

- On row hover, add a subtle left border: `border-l-2 border-transparent group-hover:border-amber-500/40 transition-colors` (per task #82 palette)
- Add a one-time instructional hint below the table header: `"Hover a row to edit or delete"` in `text-xs text-gray-600`. Store a `has_seen_edit_hint` flag in the config table. After the user's first inline edit on any table, hide the hint permanently.

### Shared row behavior

Ensure all three tables use `<tr class="group">` wrapper for hover detection.

## Decision Needed

**Edit hint persistence:** Should the "Hover a row to edit or delete" hint:
1. Disappear after the first inline edit (persisted in DB config) — more discoverable
2. Disappear after 5 views of any table page (count-based) — less intrusive
3. Never show, rely on the hover border accent alone — minimal

## Files

| File | Action |
|------|--------|
| `src/lib/expense-list/ExpenseTable.svelte` | Modify — add left border hover, edit hint |
| `src/lib/rules/RulesTable.svelte` | Modify — align icon styles, add left border |
| `src/lib/Categories.svelte` | Modify — replace click-to-rename + ✕ with standard pattern |

## Verification
1. All three tables: hover reveals same-style edit/delete icons
2. All three tables: row hover shows left amber border accent
3. Categories: pencil icon → inline edit works, trash icon → delete modal
4. Edit hint shows on first visit, disappears after first edit
