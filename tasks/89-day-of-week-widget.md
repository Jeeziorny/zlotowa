# Task 89: Widget — Spending by Day of Week

## Problem

Users have no way to see behavioral spending patterns across the week. Weekend splurges, Monday grocery runs, or Friday dining out are invisible without manual analysis.

## Solution

Add a "Day of Week" bar chart widget showing aggregate spending per weekday (Mon–Sun). Helps reveal habitual spending patterns.

### Chart

- **Type:** Unovis grouped bar (`VisGroupedBar`) with axes and tooltips
- **X-axis:** Day labels (Mon, Tue, Wed, Thu, Fri, Sat, Sun)
- **Y-axis:** Total or average spending for that weekday
- **Highlight:** The current day's bar uses a brighter/different shade
- **Tooltips:** Day name, total amount, transaction count, average per occurrence

### Toggle: Total vs Average

- Small toggle or pill bar: "Total" / "Avg" (average per week)
- Default to "Total"
- Persist in widget config

### Time Range

- Reuse `DATE_RANGE_PRESETS` pill bar (3M / 6M / 12M / All)
- Default to 6M
- Persist in widget config

### Data

- Pure frontend computation from `expenses` prop — no backend changes
- Parse `date` string to `Date` object, use `getDay()` to bucket into weekdays
- Compute both total and average (total / number of weeks in range) for the toggle

### Styling

- Bars in `CHART_PALETTE[0]`, today's bar in a slightly different amber shade or with a subtle border
- Standard widget card
- Height ~180px

## Files to Create/Modify

- **Create** `src/lib/widgets/DayOfWeek.svelte` — the widget component
- **Modify** `src/lib/widgets/registry.js` — register widget (`size: "half"`)

## Notes

- EmptyState when no expenses in range
- Week starts Monday (ISO standard) — consistent with most finance apps
