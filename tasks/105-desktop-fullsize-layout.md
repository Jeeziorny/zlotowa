# Task 105: Desktop Full-Size Layout Optimization

**Track:** Frontend — Layout & Responsiveness
**Blocked by:** nothing
**Blocks:** nothing

## Problem

The app currently targets a ~1200px default window (set in `tauri.conf.json` line 16) and uses only `md:` (768px) Tailwind breakpoints. When the user resizes to full desktop (1440px+), several pages waste significant horizontal space:

- **Dashboard:** Widget grid already scales well — the 2-column layout naturally widens as the window grows. No changes needed here.
- **Add Expense:** Form is locked to `max-w-lg` (512px) at `AddExpense.svelte:102`, leaving ~640px empty on the right at 1440px.
- **Settings:** Content locked to `max-w-2xl` (672px) at `Settings.svelte:41`, ~480px wasted.
- **Budget Planning:** Both `BudgetOverview` and `BudgetCreator` are unconstrained in width (which is fine), but their internal content (progress bars, tables) stretch the full ~1200px content area — readable but could use a subtle max-width to prevent ultra-wide lines.
- **Window config:** No `minWidth`/`minHeight` set, so the window can be shrunk to unusable sizes.

No `lg:` (1024px) or `xl:` (1280px) breakpoints exist anywhere in the frontend. Pages that already scale well (ExpenseList table, Categories table, Rules page) don't need changes — `w-full table-fixed` naturally fills the available space.

## Current State

### Layout shell — `src/App.svelte:160-163`
```html
<div class="flex h-screen bg-gray-950 text-gray-100">
  <Sidebar ... />
  <main class="flex-1 overflow-y-auto p-8">
```
Sidebar is fixed `w-56` (224px) in `src/lib/Sidebar.svelte:49`. Main content gets the remainder via `flex-1`. At 1440px: 1440 - 224 = 1216px available, minus 64px padding = **1152px content area**.

### Dashboard grid — `src/lib/Dashboard.svelte:371`
```html
<div class="grid grid-cols-1 md:grid-cols-2 gap-6">
```
Widget sizing is controlled by `size` field in `src/lib/widgets/registry.js`:
- `"full"` → `md:col-span-2` (line 375)
- `"half"` → takes 1 grid cell

11 widgets registered, 10 are `"half"`, 1 is `"full"` (TotalStats). TotalStats has its own internal 3-column grid at `TotalStats.svelte:66`:
```html
<div class="grid grid-cols-1 md:grid-cols-3 gap-6">
```

Widget picker also uses 2-column grid at `Dashboard.svelte:262`:
```html
<div class="grid grid-cols-1 md:grid-cols-2 gap-3">
```

Skeleton loading matches: `Dashboard.svelte:353-363`.

### AddExpense form — `src/lib/AddExpense.svelte:102`
```html
<div class="max-w-lg bg-gray-900 rounded-xl p-6 border border-gray-800">
```
Single-column form with 4 fields (Date, Title, Amount, Category) stacked vertically. Form fields use `w-full` internally. The `max-w-lg` (512px) constraint means on a 1152px content area, 640px is empty.

### Settings — `src/lib/Settings.svelte:41`
```html
<div class="max-w-2xl">
```
Contains tabs (General, LLM, Data). Sub-components: `LlmSettings.svelte`, `UploadHistory.svelte`, `BackupRestore.svelte`, `DataExport.svelte`. The `max-w-2xl` (672px) wraps all tab content.

### Window config — `src-tauri/tauri.conf.json:13-21`
```json
{
  "title": "złotówa",
  "width": 1200,
  "height": 800,
  "resizable": true,
  "fullscreen": false,
  "dragDropEnabled": false
}
```
No `minWidth` or `minHeight` properties.

### Pages that already scale well (NO changes needed)
- **ExpenseList/ExpenseTable:** `w-full table-fixed` with flexible Title column — scales naturally.
- **Categories:** `w-full` table, no max-width constraint.
- **Rules:** Same pattern as Categories.
- **BulkUpload:** Steps use full width, cards use flex layouts.

## Scope

### 1. Add min window dimensions
**File:** `src-tauri/tauri.conf.json`

Add `minWidth` and `minHeight` to the window config to prevent shrinking below usable sizes:
```json
{
  "width": 1200,
  "height": 800,
  "minWidth": 900,
  "minHeight": 600,
  "resizable": true
}
```
This prevents the sidebar + content from collapsing to unreadable widths.

### 2. AddExpense: responsive two-column layout at `lg:`
**File:** `src/lib/AddExpense.svelte`

Change the form container (line 102) from `max-w-lg` to `max-w-2xl`:
```html
<div class="max-w-2xl bg-gray-900 rounded-xl p-6 border border-gray-800">
```

Then arrange Date and Amount side-by-side on wider screens. Wrap those two fields in a responsive grid:
```html
<div class="grid grid-cols-1 lg:grid-cols-2 gap-4">
  <div><!-- Date field --></div>
  <div><!-- Amount field --></div>
</div>
```

Title and Category remain full-width (they benefit from horizontal space). The submit button also stays full-width.

This keeps the form compact on default 1200px but uses the extra width on 1440px+ to put related short fields side by side, which is a natural desktop form pattern.

### 3. Settings: widen constraint
**File:** `src/lib/Settings.svelte`

Change `max-w-2xl` (line 41) to `max-w-4xl`:
```html
<div class="max-w-4xl">
```

Settings content (checkboxes, text inputs, provider selects) doesn't stretch awkwardly because sub-components use their own internal constraints (e.g., LLM settings has form fields that don't expand beyond their natural widths). The wider container just prevents the tab bar and section cards from looking cramped on wide screens.

### 4. Budget Planning: add a max-width to prevent ultra-wide stretching
**File:** `src/lib/BudgetPlanning.svelte`

The budget pages currently have no max-width constraint at all, which means progress bars and summary grids stretch to 1152px on the default window. On even wider screens this becomes hard to read.

Add a `max-w-4xl` wrapper around the content (after the heading):
```svelte
<div class="max-w-4xl">
  <!-- tabs + content -->
</div>
```

This caps the content at 896px, which is comfortable for the data-dense budget tables and progress bars without looking too narrow.

## Files to Change

| File | Change |
|---|---|
| `src-tauri/tauri.conf.json` | Add `minWidth: 900`, `minHeight: 600` |
| `src/lib/AddExpense.svelte` | Widen container to `max-w-2xl`, arrange Date+Amount in 2-col grid at `lg:` |
| `src/lib/Settings.svelte` | Change `max-w-2xl` to `max-w-4xl` |
| `src/lib/BudgetPlanning.svelte` | Add `max-w-4xl` wrapper around tab content |

## Test Scenarios

### Frontend (manual UI tests)

1. **Default window (1200x800):** Open the app at its default size. Verify all pages look identical to how they look today — no layout shifts, no broken spacing. AddExpense form should show Date and Amount stacked (below `lg:` breakpoint).

2. **Full-size window (1440x900+):** Resize the window to 1440px or wider.
   - **Dashboard:** Widget grid should remain 2 columns, with each widget naturally wider (~564px each). No layout changes from current behavior.
   - **AddExpense:** Form should be wider (up to 672px). Date and Amount fields should appear side-by-side. Title and Category remain full-width below them.
   - **Settings:** Content should be wider, tab bar extends. No awkward stretching of individual form elements.
   - **Budget Planning:** Content should not exceed ~896px width. Progress bars and summary grid should be comfortably readable.

3. **Window shrink to minimum:** Try to resize the window below 900x600. Window should stop resizing at the minimum. All content should still be usable at 900px width.

4. **Expense table unaffected:** At 1440px, verify the ExpenseList table still fills the full width with the Title column expanding. No layout changes should be visible.

5. **Bulk upload unaffected:** At 1440px, go through the bulk upload flow. All steps should render correctly without layout issues.

## Acceptance Criteria

- No regressions at the default 1200x800 window size — everything looks the same as before
- Dashboard keeps its 2-column layout, widgets naturally widen on larger screens
- AddExpense form uses 2-column layout for Date+Amount at `lg:` breakpoint
- Settings content area widened to `max-w-4xl`
- Budget Planning content capped at `max-w-4xl`
- Window cannot be resized below 900x600
