# Budget Planning

The Budget page lets you create date-range budgets with per-category spending limits, plan upcoming expenses, and import calendar events with estimated costs.

## Accessing Budget Planning

Click **Budget** in the sidebar to open the Budget Planning page.

## Tabs

The page has three tabs:

### Overview (default)

If an active budget exists (one whose date range covers today), it shows:

- **Date range** header (e.g. "2026-03-01 — 2026-03-31")
- **Summary card** with total budgeted, spent, and remaining
- **Per-category progress bars** with color coding:
  - **Green** — under 80% of budget
  - **Amber** — 80–100% of budget (approaching limit)
  - **Red** — over budget (shows "OVER by X.XX")
- **Calendar costs** — events with assigned amounts and their total
- **Planned expenses** — add, view, and delete planned expenses
- **Edit category budgets** — collapsible section to adjust budget amounts
- **Delete Budget** button to remove the current budget

If no active budget exists, a prompt directs you to the Create+ tab.

### Create +

A multi-step wizard to create a new budget:

1. **Step 1 — Dates**: Set start and end dates. The system checks for overlap with existing budgets.
2. **Step 2 — Categories**: Pre-populated from your 3-month spending averages. Adjust amounts and add more categories as needed.
3. **Step 3 — Review**: Confirm the date range, category count, and total budget before creating.

This tab is disabled when an active budget already exists.

### Calendar

Import calendar events from an `.ics` file (Google Calendar, Apple Calendar, etc.):

1. Drag & drop an `.ics` file or click **Browse files**
2. Events within the budget's date range are automatically filtered and imported
3. Re-importing replaces previous events (no duplicates)
4. Each event has an editable **Amount** column — enter estimated costs directly in the table

This tab requires an active budget.

## Dashboard Widget

Add the **Budget Status** widget to your dashboard for a quick glance. It shows:

- Date range of the active budget
- Budgeted / Spent / Remaining totals
- A progress bar with color coding
- Warning count for over-budget categories
- "No active budget" message if none exists
