# Task 25: Bulk Upload UX Overhaul

**Track:** Frontend — UX improvements
**Priority:** HIGH
**Blocked by:** nothing
**Blocks:** nothing

## Problem

Several UX issues were identified during client demo in the Bulk Upload flow:

1. **No LLM progress feedback** — When the LLM is classifying expenses, the user sees no indication that work is happening. Need a popup/overlay with a progress bar or spinner.
2. **Match keyword & category fields too small** — The inline text inputs in the review table are cramped. They need to be relocated below the row data (title, date, amount) so longer strings can be entered.
3. **Chip input for category** — Instead of a plain text field, typing a category and pressing Enter should create a chip/label below the row. Text field clears, chip appears. Limit to 1 chip for MVP (builds on Task #19).
4. **Column mapping shows too many rows** — Only 1 row of actual data should be shown during column mapping. Table headers should have placeholders (e.g. "Click to assign...") so users understand they need to take action.
5. **LLM warning popup not dismissible** — The "No LLM API key configured..." banner cannot be closed by the user. Add a close/dismiss button.
6. **Tab rename** — Change "Bulk Upload" to "Expense bulk upload" in sidebar navigation.

## Current State

### LLM Progress (Note 1)
- `BulkUpload.svelte` calls `invoke("parse_and_classify", ...)` which blocks until done
- No intermediate progress — user sees a stale review table until classification finishes
- Need: overlay/modal with spinner + message ("Classifying expenses with AI...") during the async call

### Review Table Layout (Notes 2, 3)
- Match keyword and category are inline `<input>` fields in table columns
- Works for short strings but truncates longer category names or regex patterns
- Need: move these below the row's title/date/amount, full-width inputs or chip display

### Column Mapping (Note 4)
- Currently shows 5 preview rows via `preview_rows(5)`
- Headers have color coding when assigned but no placeholder text for unassigned columns
- Need: show 1 row, add placeholder text like "Click to assign column type" on unassigned headers

### LLM Warning (Note 5)
- Banner rendered in the column mapping step
- No close button or dismiss mechanism
- Need: add an X button to dismiss, persist dismissal for the session

### Tab Name (Note 6)
- Sidebar in `App.svelte` shows "Bulk Upload"
- Need: rename to "Expense bulk upload"

## Scope

### 1. LLM Classification Progress Overlay
- When `parse_and_classify` is invoked, show a modal overlay with:
  - Spinner animation
  - Message: "Classifying expenses with AI..."
  - Disable interaction with the review table underneath
- Dismiss overlay when classification completes

### 2. Review Table Layout Redesign
- Each expense row becomes a card-like layout:
  - Top: Date | Title | Amount (read-only display)
  - Bottom: Category chip input + Match keyword input (full width)
- Category input: type text → press Enter → chip appears below, input clears. Max 1 chip for MVP.
- Match keyword: full-width text input below the row data

### 3. Column Mapping Simplification
- Change `preview_rows()` call from 5 to 1
- Add placeholder text on unassigned column headers: "Click to assign" in muted gray
- Keep the existing popover assignment mechanism

### 4. Dismissible LLM Warning
- Add X button to the "No LLM API key configured" banner
- Track dismissed state with `$state(false)` — session-only, reappears on next visit

### 5. Rename Tab
- In `App.svelte` sidebar, change "Bulk Upload" label to "Expense bulk upload"
- Update any references in navigation logic

## Files to Change

| File | Change |
|---|---|
| `src/lib/BulkUpload.svelte` | LLM progress overlay, review table layout, column mapping changes, dismissible warning |
| `src/App.svelte` | Rename "Bulk Upload" to "Expense bulk upload" in sidebar |

## Acceptance Criteria

- LLM classification shows a spinner/progress overlay while working
- Match keyword and category inputs are below row data, full-width
- Category uses chip input (1 chip max for MVP)
- Column mapping shows 1 preview row with placeholder headers
- LLM warning banner has a close button
- Sidebar tab reads "Expense bulk upload"
- Dark theme conventions maintained
- Svelte 5 syntax used
