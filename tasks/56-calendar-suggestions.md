# 56 — Calendar Suggestions

## Problem

The current calendar feature stores iCal events in the database and lets users manually assign amounts — this is overengineered. Users want a lightweight approach: upload an ICS file, get smart suggestions that correlate calendar events with expense history, no DB storage.

## Dependencies

- **Task 55** — Calendar tab disabled, planned expenses removed

## Design (Needs Discussion)

### Flow

1. Upload `.ics` as optional step when creating a budget (in BudgetCreator)
2. Parse events in-memory (existing `parse_ics()` + `filter_events_by_date_range()` in `crates/core/src/ical.rs`)
3. Correlate events with historical expense data
4. Display SUGGESTIONS above the budget planning UI — a list of points summarizing insights

### Open Questions

- **Suggestion algorithm:** How should calendar events be correlated with expense history? Options:
  - Title matching (fuzzy match event summaries against past expense titles/categories)
  - Category inference (map event types to spending categories)
  - Historical pattern detection (e.g., "Last time you had a dentist appointment, you spent $X")
- **Suggestion format:** Simple text bullets? Cards with amounts? Actionable (pre-fill budget amounts)?
- **Where in the flow:** During budget creation (step 1 or 2)? Or as a persistent section in Overview?

### Backend Cleanup (Part of This Task)

- Remove `calendar_events` DB table (add `DROP TABLE IF EXISTS`)
- Remove `save_calendar_events`, `get_calendar_events`, `update_calendar_event_amount` from db.rs
- Remove `CalendarEvent` model (or repurpose as in-memory struct)
- Remove `import_calendar_events`, `update_calendar_event_amount` IPC commands
- Remove `CalendarEvents.svelte` component
- Remove calendar-related fields from `BudgetSummaryOutput`

### New Functionality

- Add IPC command that accepts ICS content + budget date range, returns suggestions
- Suggestion generation logic in `crates/core/src/` (new module or extend ical.rs)
- UI component to display suggestions in BudgetCreator

## Scope

TBD — needs design discussion before implementation.
