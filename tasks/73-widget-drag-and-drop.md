# Task 73 — Dashboard Widget Drag-and-Drop

Item: #6 (widget reorder is clunky).

## Problem

Dashboard edit mode uses `←` / `→` text buttons per widget card to reorder. With 5+ widgets, rearranging requires many clicks and has poor spatial feedback.

## Solution

Implement HTML5 drag-and-drop in edit mode.

### Drag behavior

- In edit mode, each widget card gets `draggable="true"`.
- Show a drag handle icon (⠿ grip dots or ≡ lines) at the top-left of each card in edit mode. Style: `cursor-grab text-gray-600 hover:text-gray-400`.
- On drag start: add `ring-2 ring-emerald-500/50 opacity-60` to the dragged card. Store the source index.
- On drag over another card: show a vertical insertion indicator — a `w-1 h-full bg-emerald-500 rounded` bar between cards. Use `e.preventDefault()` to allow drop.
- On drop: reorder the `activeWidgets` array, save to DB via `invoke("save_active_widgets")`.
- On drag end: remove all visual indicators.

### Grid considerations

The dashboard uses `grid grid-cols-1 md:grid-cols-2`. Drag indicators should appear:
- Between grid cells (vertical bar for side-by-side, horizontal bar for stacked)
- Use `dragover` position calculation relative to the target card center to determine insert-before vs insert-after

### Fallback

Keep the `←`/`→` buttons as keyboard-accessible fallback, but make them smaller: `text-xs text-gray-500 hover:text-gray-400` instead of primary styling. Move them to the bottom-right of the edit toolbar.

### Implementation notes

- No external library needed — HTML5 DnD API is sufficient for 3-7 items.
- `dataTransfer.setData("text/plain", index)` to track dragged item.
- `dragenter`/`dragleave` to toggle insertion indicator classes.
- Full-width widgets (`md:col-span-2`) should still drag correctly — they occupy the full row.

## Decision Needed

**Drag handle style:**
1. ⠿ (braille dots) — common in modern UIs, compact
2. ≡ (hamburger lines) — more traditional, could be confused with menu
3. ⋮⋮ (double vertical dots) — Material Design style

## Files

| File | Action |
|------|--------|
| `src/lib/Dashboard.svelte` | Modify — drag-and-drop handlers, handle icon, indicator styles |

## Verification
1. `npm run dev` → enter edit mode → drag handle visible on each widget
2. Drag a widget → visual indicators show drop target
3. Drop → widgets reorder, saved to DB
4. Refresh → order persists
5. Arrow buttons still work as fallback
6. Full-width widgets drag correctly
