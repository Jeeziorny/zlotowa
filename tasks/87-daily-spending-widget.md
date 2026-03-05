# Task 87: Widget — Daily Spending

## Problem

The dashboard shows monthly totals (MonthlyTrend) but has no visibility into day-by-day spending within a month. Users can't spot mid-month bursts, streaks of spending, or quiet periods without scrolling through the expense list.

## Solution

Add a "Daily Spending" area chart widget showing day-by-day spending for a selectable month (defaulting to the current month). Overlay a horizontal reference line at the average daily spend for context.

### Chart

- **Type:** Unovis area chart (`VisArea` + `VisLine` for the average line)
- **X-axis:** Day of month (1–28/29/30/31)
- **Y-axis:** Total spent that day
- **Average line:** Dashed horizontal line at `totalSpent / daysInMonth`, labeled "avg"
- **Tooltips:** Day number, amount, and delta from average

### Month Selector

- Reuse the `DATE_RANGE_PRESETS`-style pill bar or a simple prev/next month navigator
- Default to current month
- Persist selected month in widget config via `onconfigchange`

### Data

- Pure frontend computation from `expenses` prop — no backend changes
- Group expenses by `date`, sum amounts per day, fill gaps with 0

### Styling

- Use `CHART_PALETTE[0]` (amber-500) for the area fill (with low opacity) and line
- Average reference line in gray-500 dashed
- Standard widget card (`bg-gray-900 rounded-xl p-6 border border-gray-800`)

## Files to Create/Modify

- **Create** `src/lib/widgets/DailySpending.svelte` — the widget component
- **Modify** `src/lib/widgets/registry.js` — register widget (`size: "half"`)

## Notes

- EmptyState for months with no expenses
- Height ~180px to match other chart widgets
