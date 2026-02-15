# 11 — UX Polish: Bulk Import Warning & Date Picker

## 1. Bulk Import: warn when no API key configured

**Where:** `src/lib/BulkUpload.svelte`, step 2 (column-mapping), above the "Next: Classify & Review" button.

**What:** On entering step 2, check LLM config via `invoke("get_llm_config")`. If no API key is set, show a non-blocking amber info bar:

> No LLM API key configured. Expenses not matched by rules will need manual categorization.

User can still proceed — this is informational only. ~15 lines.

## 2. AddExpense: replace native date picker with custom component

**Problem:** `<input type="date">` with `[color-scheme:dark]` uses the native WebKit picker, which looks out of place in the dark theme.

**Fix:** Create `src/lib/DatePicker.svelte` — a small inline calendar dropdown:

- Text input showing `YYYY-MM-DD`, clicking it opens a popover grid of days
- Month/year navigation with prev/next buttons
- Styled to match the app: gray-900 popover, gray-800 day cells, emerald-500 selected/today, gray-700 borders
- Closes on outside click or date selection
- Emits selected date as `YYYY-MM-DD` string via callback prop

Replace the native `<input type="date">` in `src/lib/AddExpense.svelte` (lines 89-96) with the new component. ~80-100 lines for the component.
