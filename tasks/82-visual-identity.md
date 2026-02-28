# Task 82 — Visual Identity & Color Palette

## Problem

The app uses a generic dark-mode color scheme — gray-950 background, gray-900 cards, emerald-400/500 as the only accent color. Every button, active state, bar chart, badge, progress indicator, and border highlight is the same emerald green. The result looks like any dark-mode admin panel template, with zero connection to the "złotówa" brand identity.

The palette needs to shift from generic emerald to a gold/amber-anchored dark theme that feels premium and intentional — tied to the "złoty" etymology while remaining a serious finance tool.

Pixel-art personality is welcome in select decorative places (empty states, favicon, branding) but the core UI should be clean and professional.

## Design Direction

**Dark + gold, not dark + green.** Think dark leather wallet, not neon terminal.

### Color System

#### Primary accent: Gold/Amber
Replace emerald as the dominant accent. Use amber/yellow tones tied to "złoty" (gold):

| Role | Old | New |
|------|-----|-----|
| Active nav item text | `text-emerald-400` | `text-amber-400` |
| Active nav item bg | `bg-gray-800` | `bg-amber-400/10` |
| Primary buttons | `bg-emerald-600 hover:bg-emerald-500` | `bg-amber-500 hover:bg-amber-400 text-gray-950` |
| Focus rings | `focus:border-emerald-500` | `focus:border-amber-500` |
| Chips/badges (accent) | `bg-emerald-900/30 text-emerald-400` | `bg-amber-900/30 text-amber-400` |
| Progress spinner | `border-t-emerald-500` | `border-t-amber-500` |
| Chart default accent | emerald gradient | amber gradient |

#### Semantic colors: Keep standard finance language
These stay — red/amber/green for budget health is universal:

| Meaning | Color | Usage |
|---------|-------|-------|
| Under budget / success | `emerald-400/500` | Budget bars, success messages |
| Warning / approaching limit | `amber-400/500` | Budget 80%+ bars |
| Over budget / error / danger | `red-400/500` | Budget overrun, delete buttons, error messages |
| Income | `cyan-400` | Income indicator in bulk upload |

#### Secondary accents: Classification sources
Keep existing semantic colors for classification badges (they're distinct from the accent):
- Rule-matched: `blue-400`
- LLM-classified: `purple-400`
- Manual/unclassified: `yellow-400`

#### Chart palette: Multi-color
Replace the monochrome emerald chart palette with a warm-toned multi-color set that harmonizes with gold:

```javascript
export const CHART_PALETTE = [
  "#f59e0b", // amber-500 (primary)
  "#d97706", // amber-600
  "#b45309", // amber-700
  "#92400e", // amber-800
  "#eab308", // yellow-500
  "#ca8a04", // yellow-600
  "#a16207", // yellow-700
  "#854d0e", // yellow-800
];
```

This gives category charts visual variety while staying in the warm gold family. For more than 8 categories, cycle with reduced opacity.

### Surface Hierarchy

Introduce subtle elevation tiers instead of flat `bg-gray-900` everywhere:

| Level | Usage | Style |
|-------|-------|-------|
| Base | Page background | `bg-gray-950` (unchanged) |
| Surface 1 | Cards, sidebar | `bg-gray-900` (unchanged) |
| Surface 2 | Interactive cards on hover, widget picker items | `bg-gray-850` or `bg-gray-800/60` |
| Surface 3 | Dropdowns, popovers, modals | `bg-gray-800` with `shadow-lg` |

### Sidebar Refinement

- Active item: left border accent `border-l-2 border-amber-400` + `text-amber-400` + `bg-amber-400/5`
- Hover: `hover:bg-gray-800/50 hover:text-gray-200` (unchanged)
- Logo area: pixel-art coin (from task #69) sits naturally with gold palette
- Footer: subtle `border-t border-gray-800/50` separator

### Icons

Replace Unicode characters (☰, ▤, ◎, ⚡, ⚙) with simple inline SVG icons:

| Nav Item | Unicode | SVG replacement |
|----------|---------|-----------------|
| Expenses | ☰ | Receipt/list icon (3 horizontal lines with left margin) |
| Categories | ▤ | Tag/folder icon |
| Budget | ◎ | Pie chart / target icon |
| Rules | ⚡ | Lightning bolt SVG |
| Settings | ⚙ | Gear SVG |

All icons: `w-5 h-5`, stroke-based, `stroke-width="1.5"`, `stroke-linecap="round"`, `stroke-linejoin="round"`. Inherit text color via `stroke="currentColor"`.

### Pixel-Art Touches (Restrained)

Pixel-art personality shows up in these specific places only:
- **Logo/favicon**: the złotówa coin (task #69)
- **Empty states**: small pixel-art illustrations (coin stack, empty wallet, etc.) — task #74 can adopt this
- **404 / zero-data dashboard**: a pixel coin character

NOT in: buttons, icons, typography, charts, tables, form elements.

### Typography

Keep system sans-serif for the UI. Optionally use a pixel/retro font for the "złotówa" wordmark in the sidebar only (the logo text, not nav labels).

## Scope

### Phase 1: Color token swap (this task)

1. **CSS custom properties** — define the palette as CSS variables in `app.css` for easy future theming:
   ```css
   :root {
     --accent: theme(colors.amber.400);
     --accent-hover: theme(colors.amber.300);
     --accent-bg: theme(colors.amber.400 / 0.1);
     --accent-border: theme(colors.amber.500 / 0.5);
     --accent-strong: theme(colors.amber.500);
     --success: theme(colors.emerald.400);
     --warning: theme(colors.amber.400);
     --danger: theme(colors.red.400);
   }
   ```
   Note: components can still use Tailwind classes directly — the CSS vars are for documentation and potential future theming.

2. **Global find-and-replace** — swap emerald accent usage to amber across all `.svelte` files:
   - `text-emerald-400` → `text-amber-400` (accent text, NOT semantic success)
   - `bg-emerald-600` → `bg-amber-500` (primary buttons) + add `text-gray-950` for contrast
   - `hover:bg-emerald-500` → `hover:bg-amber-400` (button hover)
   - `bg-emerald-900/30` → `bg-amber-900/30` (badge backgrounds)
   - `bg-emerald-400/10` → `bg-amber-400/10` (chip backgrounds)
   - `focus:border-emerald-500` → `focus:border-amber-500` (input focus)
   - `border-emerald-500/50` → `border-amber-500/50` (hover borders)
   - `border-t-emerald-500` → `border-t-amber-500` (spinner)

   **Do NOT replace** semantic emerald (budget "OK" bars, success messages) — only accent usage.

3. **Sidebar SVG icons** — replace Unicode characters with inline SVG icons.

4. **Sidebar active state** — add left border accent.

5. **Button text contrast** — amber-500 background needs dark text (`text-gray-950`) unlike emerald-600 which worked with white text.

### Phase 2: Deferred (separate tasks)

- Chart palette update → handled by task #70 (update its palette definition)
- Empty state pixel-art illustrations → task #74
- Surface hierarchy refinement → can be a follow-up

## Files

| File | Action |
|------|--------|
| `src/app.css` | Modify — add CSS custom properties for palette |
| `src/lib/Sidebar.svelte` | Modify — SVG icons, active state left border, amber accents |
| `src/lib/Dashboard.svelte` | Modify — accent color swap |
| `src/lib/Categories.svelte` | Modify — accent color swap |
| `src/lib/BudgetPlanning.svelte` | Modify — accent color swap (keep semantic green) |
| `src/lib/ExpenseList.svelte` | Modify — accent color swap |
| `src/lib/AddExpense.svelte` | Modify — accent color swap |
| `src/lib/Settings.svelte` | Modify — accent color swap |
| `src/lib/Rules.svelte` | Modify — accent color swap |
| `src/lib/DatePicker.svelte` | Modify — accent color swap |
| `src/lib/ConfirmLeaveModal.svelte` | Modify — accent color swap |
| `src/lib/widgets/*.svelte` | Modify — accent color swap |
| `src/lib/bulk-upload/*.svelte` | Modify — accent color swap |
| `src/lib/expense-list/*.svelte` | Modify — accent color swap |
| `src/lib/budget/*.svelte` | Modify — accent color swap (keep semantic green) |
| `src/lib/settings/*.svelte` | Modify — accent color swap |
| `src/lib/rules/*.svelte` | Modify — accent color swap |

## Verification

1. `npm run dev` — app renders with gold/amber accents, no emerald accent remnants
2. Sidebar: SVG icons render cleanly, active item has left amber border
3. Buttons: amber-500 with dark text, hover amber-400 — legible and clickable
4. Budget bars: still show green/amber/red semantic colors (not changed)
5. Classification badges: blue/purple/yellow still distinct
6. Focus rings: amber on inputs
7. Chips/badges: amber tints
8. No accessibility regression — contrast ratios still meet WCAG AA
9. `grep -r "emerald" src/` — only hits should be semantic (budget OK, success messages)

## Ordering

- **Blocks:** Nothing — this task should be done FIRST among FE tasks
- **Blocked by:** Nothing
- **Should run before:** #69 (branding), #70 (charts), #71-78 (all FE tasks) — so they inherit the correct palette

Ideally execute this task before any other FE task to avoid double-work on color values.
