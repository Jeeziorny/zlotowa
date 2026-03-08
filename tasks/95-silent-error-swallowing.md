# Task 95: Replace Silent `.catch(() => {})` With Error Feedback

## Goal

Replace 11 instances of `.catch(() => {})` in ExpenseList and Rules with user-visible error messages.

## Deliverables

### 1. ExpenseList — `src/lib/ExpenseList.svelte`

Lines 117, 127, 138, 144, 150, 157 — all `fetchExpenses().catch(() => {})`.

Replace each with `.catch((e) => { error = e; })` (or equivalent) and display an error banner in the UI. Use the existing toast/error pattern from other components.

### 2. Rules — `src/lib/Rules.svelte`

Lines 71, 77, 84, 90, 96 — all `fetchRules().catch(() => {})`.

Same approach: surface errors to users.

### 3. Pattern

Use a reactive `$state()` error variable, clear it on successful fetch, and render an error banner (red/amber) at the top of the list area. Keep it minimal — a single line like "Failed to load expenses. Try again." with a dismiss button is enough.

## Files to modify
- `src/lib/ExpenseList.svelte`
- `src/lib/Rules.svelte`

## Notes
- These catches exist to prevent uncaught promise rejections during fire-and-forget reactive updates. The fix should preserve that (still catch), but show feedback instead of swallowing.
- Don't add retry logic — just inform the user.
