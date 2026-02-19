# Task 27: Fix Navigation Resetting to Dashboard

**Track:** Frontend — Bug fix
**Priority:** HIGH
**Blocked by:** nothing
**Blocks:** nothing

## Problem

Client reports: "Why the hell every time I change the app app goes to dashboard???!!!"

When interacting with the app (e.g. saving data, performing actions), the view unexpectedly resets to the Dashboard. This is highly disruptive to workflow.

## Current State

### App.svelte
- `currentPage` is a `$state("dashboard")` string
- Sidebar emits `onnavigate(page)` callback which sets `currentPage`
- Page components are swapped with `{#if}` / `{:else if}` chain
- Components are destroyed and recreated on every navigation (no state preservation)

### Likely Cause
- Some operation (e.g. data save, IPC callback, reactive effect) is resetting `currentPage` back to `"dashboard"`
- Could be a reactive `$effect` or `$derived` that unintentionally overwrites `currentPage`
- Could be a callback from a child component that triggers navigation

## Scope

### 1. Investigate Root Cause
- Search for any code that sets `currentPage = "dashboard"` outside of initial load
- Check for reactive effects that might reset the page state
- Check if child components emit navigation events unintentionally
- Test: perform actions on each page (save budget, delete expense, etc.) and verify page stays

### 2. Fix
- Ensure `currentPage` only changes when the user explicitly clicks a sidebar nav item
- If child components need to navigate (e.g. dashboard widget → expenses), they should do so through an explicit `onnavigate` prop, not by side effect

## Files to Change

| File | Change |
|---|---|
| `src/App.svelte` | Fix `currentPage` state management — prevent unintended resets |
| Possibly child components | Remove any accidental navigation triggers |

## Acceptance Criteria

- Navigating to any tab stays on that tab until user explicitly navigates away
- Saving data, deleting items, or performing any action does NOT reset to Dashboard
- Dashboard is only shown on initial app load or when user clicks "Dashboard" in sidebar
