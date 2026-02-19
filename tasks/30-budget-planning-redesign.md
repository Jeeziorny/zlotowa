# Task 30: Budget Planning Redesign

**Track:** Full-stack — Major feature rework
**Priority:** HIGH
**Blocked by:** nothing
**Blocks:** nothing

## Problem

Client wants a significantly different budget planning experience with three distinct tabs and a reworked creation flow. Current implementation uses month-based budgets; client wants date-range-based budgets with non-overlapping periods.

## Current State

### BudgetPlanning.svelte
- Three tabs already exist: Budget | Calendar | Overview
- `MonthSelector` in header for month/year navigation
- Budget tab renders `BudgetEditor` (category limits + planned expenses)
- Calendar tab renders `CalendarEvents` (imported iCal events)
- Overview tab renders `BudgetOverview` (totals: budgeted vs spent vs planned)

### Backend (db.rs)
- `budgets` table: `id, year, month` — keyed by year+month
- `budget_categories`: per-category limits tied to a `budget_id`
- `planned_expenses`: upcoming costs tied to a `budget_id`

## Scope

### 1. Overview Tab (Default)
- Make Overview the default tab (currently Budget is default)
- Replace "Month Year" display with budget date range: "FROM — TO" (e.g. "2026-01-15 — 2026-02-14")
- For now there will be only one active budget, so just show the current one
- If no budget exists, show a prompt to create one

### 2. "Create +" Tab (replaces current "Budget" tab)
- Rename tab from "Budget" to "Create +"
- Multi-step budget creation process:

**Step 1: Set Budget Period**
- Date range picker: Start date and End date
- Validation: budgets cannot overlap with existing budgets
- Display error if overlap detected

**Step 2: Set Category Budgets**
- Table shows only categories that had expenses in the previous month (not all categories)
- Columns: Category | Avg (from `get_category_averages`) | Budget
- "Budget" column defaults to Avg value for each row
- User can adjust amounts

**Step 3: Add More Categories (optional)**
- "Add more categories" button opens a popup/modal
- Modal lists all user categories except those already in the table
- User can select additional categories to add to the budget

**Step 4: Create**
- "Create" button saves the budget
- After creation, the "Create +" tab greys out and shows message: "Current time period already has a budget."
- User must go to Overview to see/manage it

### 3. Calendar Tab
- Current iCal import works well
- Add ability to assign a monetary amount to calendar events
- Each event row gets an editable "Amount" field
- This amount represents expected spending for that event/day
- Amounts feed into budget overview as planned spending

### 4. Backend Changes
- Modify `budgets` table: replace `year, month` with `start_date, end_date` (TEXT dates)
- Add overlap validation: `SELECT COUNT(*) FROM budgets WHERE start_date < ?end AND end_date > ?start`
- Add `amount` column to `calendar_events` table (nullable REAL)
- Update `get_budget_summary` to work with date ranges instead of year/month
- Add IPC command to update calendar event amount

## Files to Change

| File | Change |
|---|---|
| `crates/core/src/db.rs` | Modify budgets schema (date ranges), add overlap check, calendar event amounts |
| `src-tauri/src/lib.rs` | Update budget IPC commands for date ranges, add calendar event amount command |
| `src/lib/BudgetPlanning.svelte` | Rework tab structure, make Overview default |
| `src/lib/BudgetEditor.svelte` | Multi-step creation flow, date range picker, category filtering |
| `src/lib/BudgetOverview.svelte` | Show date range instead of month, handle "no budget" state |
| `src/lib/CalendarEvents.svelte` | Add editable amount field per event |

## Migration Notes

- Need a DB migration to change `budgets` from `(year, month)` to `(start_date, end_date)`
- Existing budgets can be migrated: `start_date = "YYYY-MM-01"`, `end_date = last day of month`
- `calendar_events` gets new nullable `amount` column

## Acceptance Criteria

- Overview is the default tab, showing budget date range "FROM — TO"
- Budget creation is a multi-step process with date range, category defaults from averages
- Categories shown during creation are filtered to those used in previous period
- "Add more categories" popup allows adding extra categories
- After budget creation, "Create +" tab is greyed out with message
- Budgets cannot overlap (validated in backend)
- Calendar events can have monetary amounts assigned
- Dark theme conventions maintained
- Svelte 5 syntax used
