# Task 23: UI Polish & Widget Animations

**Track:** Frontend — Visual improvement
**Priority:** LOW
**Blocked by:** nothing (can run in parallel with backend tasks, but ideally after 20-22 to polish final UI)
**Blocks:** nothing

## Problem

The app has a solid dark theme and functional layout, but lacks visual polish:

- **No page transitions** — pages swap instantly with no animation
- **No widget animations** — widgets appear/disappear instantly on add/remove, no entrance effects
- **No drag-and-drop** for widget reordering — only arrow buttons (←/→)
- **Minimal micro-interactions** — only `transition-colors` on hover, no scale/shadow/elevation changes
- **No loading skeletons** — blank screen while data loads
- **No toast notifications** — success/error feedback is inline text that can be missed
- **Bar charts are static** — `transition-all` on width/height exists but with default timing, no stagger

## Current State

### Dashboard (`src/lib/Dashboard.svelte`)

- Widget add/remove: direct array manipulation, no enter/exit animation (lines 38-46)
- Widget reorder: `moveWidget(index, direction)` swaps array elements (lines 49-56), no flip animation
- Widget toolbar: tiny ←/→/× buttons (lines 113-139)

### Widget components (`src/lib/widgets/`)

- `MonthlyTrend.svelte`: `transition-all` on bar height — basic, no stagger
- `SpendingByCategory.svelte`: `transition-all` on bar width — basic, no stagger
- `TotalStats.svelte`: static number cards, no count-up animation
- `BiggestExpense.svelte`, `MostFrequent.svelte`: static content

### Other components

- `BulkUpload.svelte`: single `animate-spin` spinner, otherwise no animations
- `ExpenseList.svelte`: no transitions on row render
- `App.svelte`: instant page swap via `{#if currentPage === ...}` (lines 17-29)
- No shared toast/notification system

## Scope

### 1. Page transitions (`src/App.svelte`)

- Wrap page content in a keyed `{#key currentPage}` block
- Add Svelte `transition:fade` or `transition:fly` with short duration (150-200ms)
- Keep it subtle — fast crossfade, not dramatic slide

### 2. Widget entrance/exit animations (`src/lib/Dashboard.svelte`)

- Use Svelte `animate:flip` on the `{#each}` block for smooth reorder
- Add `transition:scale` or `transition:fly` for widget add/remove
- Stagger entrance on initial load: each widget fades in with a small delay

### 3. Widget drag-and-drop (`src/lib/Dashboard.svelte`)

- Add HTML5 drag-and-drop to widget cards (or use a small library if needed)
- `draggable="true"` on widget wrapper
- Show drop indicator (border highlight) while dragging over a position
- On drop: reorder `activeWidgetIds` and persist
- Keep arrow buttons as fallback for accessibility

### 4. Bar chart stagger animations (`src/lib/widgets/`)

- `MonthlyTrend.svelte`: bars animate in from 0 height with staggered delay (e.g. 50ms per bar)
- `SpendingByCategory.svelte`: bars animate in from 0 width with staggered delay
- Use `$effect` + `setTimeout` pattern or CSS `animation-delay` with `nth-child`

### 5. Number count-up animation (`src/lib/widgets/TotalStats.svelte`)

- Total amount, transaction count, category count animate from 0 to final value
- Use a simple `requestAnimationFrame` tween over ~500ms
- Only on initial load, not on every re-render

### 6. Loading skeletons

- Create a `Skeleton.svelte` component: pulsing gray rectangles matching content layout
- Use in Dashboard (widget placeholders), ExpenseList (table rows), Categories (list rows)
- Show while `loaded === false` / data is being fetched

### 7. Toast notification system

- Create `Toast.svelte` component: slides in from top-right, auto-dismisses after 3-4s
- Success (emerald), error (red), info (blue) variants
- Store toast state in a shared module (`src/lib/stores/toast.js`)
- Replace inline success/error messages in BulkUpload, ExpenseList export, Settings, Categories

### 8. Micro-interactions

- Buttons: subtle `scale(0.97)` on `:active`, `shadow` elevation on hover for primary buttons
- Cards: slight `shadow-lg` on hover for interactive cards (widget picker, category rows)
- Table rows: smoother hover transition with subtle background shift
- Sidebar: active item gets a left border accent or background highlight transition

## Files to Change

| File | Change |
|---|---|
| `src/App.svelte` | Add `{#key}` block with page transition |
| `src/lib/Dashboard.svelte` | Widget entrance/exit transitions, `animate:flip`, drag-and-drop |
| `src/lib/widgets/MonthlyTrend.svelte` | Staggered bar animation |
| `src/lib/widgets/SpendingByCategory.svelte` | Staggered bar animation |
| `src/lib/widgets/TotalStats.svelte` | Count-up number animation |
| `src/lib/Skeleton.svelte` | **New file** — reusable loading skeleton component |
| `src/lib/Toast.svelte` | **New file** — toast notification component |
| `src/lib/stores/toast.js` | **New file** — shared toast state |
| `src/lib/Sidebar.svelte` | Active item transition polish |
| Various components | Replace inline messages with toast calls |

## Implementation Notes

- Use Svelte's built-in `transition:`, `animate:`, `in:`, `out:` directives — no external animation libraries needed
- Keep all animation durations under 300ms for snappy feel
- Respect `prefers-reduced-motion` media query — wrap animations in a check or use Svelte's `reducedMotion` option
- Drag-and-drop can be plain HTML5 DnD API — no need for a library with only 5 widgets max

## Acceptance Criteria

- Pages crossfade smoothly on navigation (no jarring instant swap)
- Widgets animate in/out on add/remove and smoothly reorder on drag-and-drop
- Bar charts stagger-animate on initial render
- Stats numbers count up on first load
- Loading skeletons show while data is being fetched
- Toast notifications replace inline success/error text
- Interactions feel snappy (all transitions under 300ms)
- Animations respect `prefers-reduced-motion`
- No regression in functionality — all existing features work identically
- Svelte 5 syntax used throughout
