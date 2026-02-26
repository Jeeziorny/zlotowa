# 52 — Title Cleanup Relocation into Expenses Tab

## Problem

Title Cleanup is a niche feature that doesn't warrant its own nav tab, but users still need access to it.

## Dependencies

- **Task 51** — needs the sub-view routing pattern established there

## Changes

- Add a "Clean Titles" button in the Expenses page toolbar (secondary action, less prominent than Add/Upload)
- Clicking it opens the existing `TitleCleanup.svelte` UI as a sub-view within the Expenses page (same pattern as Add/Upload from Task 51)
- `TitleCleanup.svelte` component reused as-is

## Scope

- `src/lib/ExpenseList.svelte` — add "Clean Titles" toolbar action and sub-view routing
