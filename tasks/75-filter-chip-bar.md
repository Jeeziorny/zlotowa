# Task 75 — Filter Chip Bar

Item: #8 (active filter state is invisible).

## Problem

When filters are active on the expense list, only a small "Clear filters" text button appears inside the filter bar. Once the user scrolls down, there's no indication that results are filtered. Active filter state is invisible.

## Solution

### Chip bar

Add a horizontal chip bar between `SearchFilterBar` and `ExpenseTable`. Renders only when at least one filter is active.

Each active filter renders as a chip showing filter name and value:
- `Category: Food ×`
- `From: 2025-01-01 ×`
- `To: 2025-12-31 ×`
- `Min amount: 50 ×`
- `Max amount: 200 ×`
- `Search: "grocery" ×`

The `×` button on each chip clears that specific filter.

When 2+ filters are active, add a "Clear all" chip at the end.

### Chip styling

```
Container: flex flex-wrap gap-2 mb-3
Chip: inline-flex items-center gap-1.5 bg-gray-800 text-gray-300 border border-gray-700 rounded-full px-3 py-1 text-xs
Label: text-gray-500 (the "Category:" part)
Value: text-gray-300 (the "Food" part)
Remove: hover:text-red-400 cursor-pointer text-gray-500
Clear all: bg-gray-800 text-gray-500 hover:text-gray-300 border border-gray-700
```

### Implementation

- The chip bar lives inside `ExpenseList.svelte` (between SearchFilterBar and ExpenseTable), not inside SearchFilterBar itself.
- It reads from the same filter state variables already tracked in ExpenseList.
- Each chip's `×` calls the same `handleFilterChange(filterName, "")` handler, clearing that specific filter.
- "Clear all" resets all filter values and calls the search reset.

### Behavior

- Chips animate in/out with `transition-opacity` or a simple conditional render.
- Removing a chip immediately updates the table results (triggers the existing debounced query).
- Search chip shows the current search term in quotes.

## Files

| File | Action |
|------|--------|
| `src/lib/ExpenseList.svelte` | Modify — add chip bar section between filter bar and table |

## Verification
1. Set a category filter → chip appears: "Category: Food ×"
2. Add a date filter → second chip appears alongside
3. Click × on category chip → filter cleared, chip removed, table updates
4. Click "Clear all" → all chips gone, all filters reset
5. No filters active → chip bar hidden entirely
