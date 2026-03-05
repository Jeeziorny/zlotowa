# Task 90: Widget — Month-over-Month Comparison

## Problem

Users want a quick "how am I doing vs last month?" view. The current MonthlyTrend shows overall totals per month but doesn't break them down by category side-by-side.

## Solution

Add a "Month vs Month" grouped bar chart comparing spending per category between two months (defaulting to this month vs last month).

### Chart

- **Type:** Unovis grouped bar (`VisGroupedBar`) with two bars per category (this month + last month)
- **X-axis:** Category names
- **Y-axis:** Spending amount
- **Colors:** This month in `CHART_PALETTE[0]` (amber-500), last month in gray-500 (muted)
- **Tooltips:** Category name, this month amount, last month amount, difference (+ or -)

### Month Selector

- Two-month selector: "Current" and "Compare to" dropdowns or prev/next navigation
- Default: current month vs previous month
- Persist in widget config

### Summary Line

- Below the chart, a one-line summary: "This month: X total (+/-Y% vs last month)"

### Data

- Pure frontend computation from `expenses` prop — no backend changes
- Group expenses by `(month, category)`, filter to the two selected months
- Show only categories that appear in at least one of the two months
- Sort by this month's amount descending
- Cap at top 8 categories to keep chart readable; merge rest into "Other"

### Styling

- Standard widget card
- Height ~200px
- Legend: two items (this month / comparison month labels with color swatches)

## Files to Create/Modify

- **Create** `src/lib/widgets/MonthComparison.svelte` — the widget component
- **Modify** `src/lib/widgets/registry.js` — register widget (`size: "half"`)

## Notes

- EmptyState when both months have no data
- If only one month has data, still show bars (the other month's bars are simply absent)
- `size: "half"` but may benefit from `"full"` if many categories — decide during implementation
