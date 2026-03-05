# Task 88: Widget — Category Trend

## Problem

SpendingByCategory shows a snapshot of category distribution but doesn't reveal how individual categories change over time. Users can't answer "is my grocery spending creeping up?" without manually comparing months.

## Solution

Add a "Category Trend" stacked bar chart widget showing per-category spending over the last N months. Each bar is a month, segments are categories, stacked to show both totals and composition.

### Chart

- **Type:** Unovis stacked bar (`VisStackedBar`) with axes and tooltips
- **X-axis:** Month labels (e.g., "Jan 26", "Feb 26")
- **Y-axis:** Spending amount
- **Stacking:** Top 5 categories by total spend get individual colors from `CHART_PALETTE`; remaining categories merged into a single "Other" segment
- **Tooltips:** Category name, amount for that month, percentage of month total

### Time Range

- Reuse `DATE_RANGE_PRESETS` pill bar (3M / 6M / 12M / All)
- Default to 6M
- Persist in widget config

### Legend

- Color swatches + category names below the chart (same pattern as SpendingByCategory)
- Include "Other" if applicable

### Data

- Pure frontend computation from `expenses` prop — no backend changes
- Group by `(month, category)`, sum amounts, sort categories by global total, cap at top 5 + Other

### Styling

- Colors from `CHART_PALETTE` (one per category), "Other" in gray-600
- Standard widget card
- Height ~200px (slightly taller to accommodate stacked bars)

## Files to Create/Modify

- **Create** `src/lib/widgets/CategoryTrend.svelte` — the widget component
- **Modify** `src/lib/widgets/registry.js` — register widget (`size: "half"`)

## Notes

- EmptyState when no data in selected range
- `size: "half"` but consider `"full"` if stacked bars are too cramped — decide during implementation
