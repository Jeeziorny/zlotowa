# Task 29: Dashboard Widget Click-to-Navigate

**Track:** Frontend — UX improvement
**Priority:** MEDIUM
**Blocked by:** nothing
**Blocks:** nothing

## Problem

Client wants dashboard widgets to act as navigation shortcuts:
1. Clicking **Total Expenses** widget → opens "Expenses" tab
2. Clicking **Transactions** (Total Stats shows transaction count) → opens "Expenses" tab
3. Clicking **Categories** (Spending by Category widget) → opens "Categories" tab

## Current State

### Dashboard.svelte
- Widgets are rendered as cards with a toolbar (reorder/remove buttons)
- Widget content is NOT clickable — no click handlers on the widget body
- Available widgets: `total-stats`, `spending-by-category`, `biggest-expense`, `monthly-trend`, `most-frequent`, `budget-status`

### Navigation
- `App.svelte` holds `currentPage` state
- Dashboard receives an `onnavigate` callback prop (need to verify)
- If not, Dashboard needs a way to trigger navigation

## Scope

### 1. Add `onnavigate` Prop to Dashboard
- Dashboard component should accept an `onnavigate(page)` callback
- App.svelte passes its navigation handler to Dashboard

### 2. Make Widgets Clickable
- `total-stats` widget: clicking the Total Expenses or Transactions area navigates to `"expenses"`
- `spending-by-category` widget: clicking navigates to `"categories"`
- Add cursor pointer and subtle hover effect (slight brightness/scale) on clickable widgets
- Other widgets (`biggest-expense`, `monthly-trend`, `most-frequent`, `budget-status`) — no click action for now

### 3. Visual Affordance
- Clickable widgets get `cursor-pointer` and a subtle hover effect
- Non-clickable widgets remain as-is

## Files to Change

| File | Change |
|---|---|
| `src/App.svelte` | Pass `onnavigate` to Dashboard component |
| `src/lib/Dashboard.svelte` | Accept `onnavigate`, pass to clickable widgets |
| `src/lib/widgets/TotalStats.svelte` | Add click handler to navigate to expenses |
| `src/lib/widgets/SpendingByCategory.svelte` | Add click handler to navigate to categories |

## Acceptance Criteria

- Clicking Total Expenses or Transactions count on dashboard navigates to Expenses tab
- Clicking Spending by Category widget navigates to Categories tab
- Clickable widgets show cursor pointer and hover feedback
- Navigation works correctly (doesn't reset state)
