# Task 26: Expense List Cleanup

**Track:** Frontend — UI cleanup
**Priority:** MEDIUM
**Blocked by:** nothing
**Blocks:** nothing

## Problem

Client feedback from demo:

1. **Drop "Source" column** — The classification source (DB/LLM/Manual) is internal metadata. User doesn't need to see who labeled an expense. Remove the column entirely.
2. **Widen "Category" column** — With Source gone, redistribute the freed space to Category so longer category names fit without truncation.
3. **Delete confirmation dialog** — Currently, single-expense deletion uses inline confirm (check/X icons in the row). Client wants an explicit "Are you sure?" confirmation before deleting.

## Current State

### ExpenseList.svelte
- Table columns: Checkbox | Date | Title | Amount | Category | Source | Actions
- Source column shows badges: "DB" (blue), "LLM" (purple), "Manual" (gray) from `classification_source`
- Single delete: trash icon → inline check/X confirmation in the action column
- Batch delete: "Delete N selected" button → inline banner confirmation above table

## Scope

### 1. Remove Source Column
- Delete the "Source" `<th>` and corresponding `<td>` from the table
- Remove the source badge rendering logic
- Keep `classification_source` in the data model (still useful for backend/exports), just don't display it

### 2. Widen Category Column
- Adjust column widths so Category gets more space (e.g. from `w-28` to `w-40` or let it flex)
- Ensure long category names display without truncation

### 3. Delete Confirmation Modal
- Replace inline check/X with a proper confirmation dialog/modal:
  - "Delete this expense?" with expense title shown
  - [Delete] and [Cancel] buttons
  - Delete button in red/destructive style
- For batch delete, keep the banner but make it more prominent with a modal overlay

## Files to Change

| File | Change |
|---|---|
| `src/lib/ExpenseList.svelte` | Remove Source column, widen Category, add delete confirmation modal |

## Acceptance Criteria

- Source column is gone from the expense table
- Category column is wider and displays long names without truncation
- Deleting an expense (single or batch) shows a clear confirmation dialog before proceeding
- Dark theme conventions maintained
- Svelte 5 syntax used
