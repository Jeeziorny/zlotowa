# Task 74 — Empty State Component & Illustrations

Item: #7 (empty states are bare text).

## Problem

Empty states across the app are plain centered text like `"No expenses yet."` in `text-gray-500`. No icons, no call-to-action buttons, no visual weight. They feel like placeholder text, not intentional design.

## Solution

### Shared component

Create `EmptyState.svelte` with these props:
- `icon` — SVG snippet name (receipt, search, folder, calendar, upload, chart)
- `title` — primary message (e.g., "No expenses yet")
- `description` — optional secondary text
- `actionLabel` — optional CTA button text
- `onaction` — optional CTA callback

Layout: centered flex column, icon above title above description above optional button.

### Icon set

All icons should be simple 2-3 path inline SVGs, `w-12 h-12 text-gray-700 mx-auto mb-3`. Consistent stroke style (stroke-width 1.5, rounded caps). No fills.

Icons needed:
- **receipt** — a receipt/document outline
- **search** — magnifying glass
- **folder** — folder outline
- **calendar** — calendar outline
- **upload** — cloud with up arrow
- **chart** — simple bar chart outline

### Where to apply

| Location | Icon | Title | Description | CTA |
|----------|------|-------|-------------|-----|
| Dashboard (no expenses) | receipt | No expenses yet | Add an expense or upload a CSV to get started | + Add Expense → navigate to add |
| Expense list (no results) | search | No matching expenses | Try adjusting your search or filters | Clear filters (if filters active) |
| Expense list (empty DB) | receipt | No expenses yet | Add your first expense to get started | + Add Expense → switch to add subview |
| Categories (none) | folder | No categories yet | Categories are created automatically when you classify expenses | — |
| Budget (none) | calendar | No budgets yet | Create a budget to track your spending | + Create Budget → switch to create tab |
| Upload history (none) | upload | No uploads yet | Bulk upload a CSV to import expenses | — |
| KeywordTracker (no match) | search | No matching expenses | Try different keywords | — |
| SpendingByCategory (empty) | chart | No category data | Add categorized expenses to see spending breakdown | — |

### Styling

```
Container: text-center py-12
Icon: w-12 h-12 text-gray-700 mx-auto mb-3
Title: text-gray-400 font-medium mb-1
Description: text-sm text-gray-600
CTA: mt-4 bg-amber-500 hover:bg-amber-400 text-gray-950 px-4 py-2 rounded-lg text-sm (per task #82 palette)
```

## Files

| File | Action |
|------|--------|
| `src/lib/EmptyState.svelte` | Create — shared component with icon set |
| `src/lib/Dashboard.svelte` | Modify — use EmptyState |
| `src/lib/expense-list/ExpenseTable.svelte` | Modify — use EmptyState |
| `src/lib/Categories.svelte` | Modify — use EmptyState |
| `src/lib/BudgetPlanning.svelte` | Modify — use EmptyState |
| `src/lib/settings/UploadHistory.svelte` | Modify — use EmptyState |
| `src/lib/widgets/KeywordTracker.svelte` | Modify — use EmptyState |
| `src/lib/widgets/SpendingByCategory.svelte` | Modify — use EmptyState |

## Verification
1. Each location shows icon + styled text instead of bare text
2. CTA buttons navigate correctly where present
3. Icons are visually consistent (same stroke weight, size, color)
4. No empty states show a bare string anymore
