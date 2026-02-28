# Task 70 — Chart Visualizations (Unovis)

Item: #2 (chart visualizations feel flat).

## Problem

`SpendingByCategory` and `MonthlyTrend` widgets use flat CSS bars with no hover interaction, no value callouts, and no visual depth. They feel basic compared to the rest of the UI.

## Library Research

### Why not the alternatives?

| Library | Bundle | Dark Theme | Svelte 5 | Boilerplate | Verdict |
|---------|--------|-----------|----------|-------------|---------|
| **Chart.js** + `svelte5-chartjs` | ~45 KB (claimed 11 KB is marketing) | Manual rebuild per `chart.update()` | 14-star wrapper, risky | ~30 lines | Canvas issues in Tauri (#4891, #5761, #9373). No built-in dark mode. Skip. |
| **Apache ECharts** + `svelte-echarts` | ~170 KB (600+ KB full) | Built-in `darkMode: true` | Unofficial wrapper, Svelte 3/4 era | ~20 lines | Overkill for an expense tracker. |
| **LayerCake** | Lightweight | Full control (manual) | Svelte 5 supported | ~150-200 lines | Headless framework — too much DIY for two charts. |
| **Plain SVG + D3** | ~10 KB | You control it | N/A | ~50 lines/chart | Viable for 2 charts but becomes maintenance debt. |
| **Unovis** | ~80-100 KB | CSS variables, built-in | Official `@unovis/svelte` | ~15 lines | Winner. |

### Why Unovis?

1. **Minimal boilerplate** — ~15 lines for a donut chart vs ~50 for D3, ~150 for LayerCake
2. **Built-in dark theme** via `--vis-dark-*` CSS variables — no manual color rebuilding
3. **Official Svelte support** — first-class `@unovis/svelte` package, not a community wrapper
4. **Actively maintained** — 2.7k stars, v1.6.4 released Jan 31, 2026
5. **TypeScript-first**, production-ready
6. **~80 KB bundle** — acceptable for a desktop Tauri app (no network penalty)
7. **Chart.js rejected** — fake bundle claims, canvas issues in Tauri, 14-star Svelte 5 wrapper
8. **ECharts rejected** — overkill, unofficial Svelte wrapper
9. **D3 was the other strong contender** but more verbose, harder to maintain if more chart types are added later

## Solution

### Install

```bash
npm install @unovis/svelte @unovis/ts --legacy-peer-deps
```

`--legacy-peer-deps` needed because `@unovis/svelte` declares `"svelte": "^3.48.0 || ^4.0.0"`. Svelte 5's backward-compat compiler handles the Svelte 4 component syntax fine. Deprecation warnings for `$$Generic`/`$$restProps` in dev console are expected and harmless.

Add `.npmrc` with `legacy-peer-deps=true` to silence warnings permanently.

### Shared Theme: `src/lib/widgets/chart-theme.js`

```javascript
export const EMERALD_PALETTE = [
  "#34d399", // emerald-400
  "#10b981", // emerald-500
  "#059669", // emerald-600
  "#047857", // emerald-700
  "#065f46", // emerald-800
  "#6ee7b7", // emerald-300
  "#a7f3d0", // emerald-200
  "#064e3b", // emerald-900
];

export function emeraldColor(index) {
  return EMERALD_PALETTE[index % EMERALD_PALETTE.length];
}

export function formatAmount(value) {
  return value.toFixed(2);
}
```

### CSS Variables in `src/app.css`

```css
/* Unovis dark theme overrides */
:root {
  --vis-tooltip-background-color: #1f2937;
  --vis-tooltip-text-color: #d1d5db;
  --vis-tooltip-shadow-color: rgba(0, 0, 0, 0.5);
  --vis-tooltip-padding: 8px 12px;
  --vis-font-family: inherit;
  --vis-axis-tick-label-color: #9ca3af;
  --vis-axis-grid-line-color: rgba(55, 65, 81, 0.4);
  --vis-axis-domain-line-color: #374151;
  --vis-axis-label-color: #9ca3af;
  --vis-axis-tick-line-color: #374151;
  --vis-donut-central-label-text-color: #e5e7eb;
  --vis-donut-central-sub-label-text-color: #9ca3af;
  --vis-donut-background-color: #1f2937;
}
```

`:root` instead of `.theme-dark` because the app always runs in dark mode.

### SpendingByCategory → Donut Chart

Full rewrite. Key design decisions:
- `VisDonut` with `arcWidth={40}`, `padAngle={0.02}`, `cornerRadius={3}`
- `centralLabel` shows total amount, `centralSubLabel` shows "total"
- Legend below donut, limited to 6 items with "+N more" overflow
- `VisTooltip` on segment hover showing category name and amount
- `pointer-events: auto` on chart container inside the clickable `<button>`
- Fixed `height={180}` for consistent rendering

### MonthlyTrend → Bar Chart

Full rewrite. Key design decisions:
- `VisGroupedBar` with `roundedCorners={3}`, `barMinHeight={2}` (zero-value months show a sliver)
- `y={[y]}` — array wrapping required by `VisGroupedBar` even for single series
- Data items use an `index` property for x-axis (months are ordinal, not numeric dates)
- X-axis: month labels (`Jan 26`), no gridlines, no domain line
- Y-axis: numeric values, horizontal gridlines, no domain line
- `VisTooltip` on bar hover showing month and amount
- `padding={{ top: 10 }}` prevents bars touching top edge
- Fixed `height={180}` for consistency with donut

## Files

| File | Action | What changes |
|------|--------|-------------|
| `package.json` | Modify | Add `@unovis/svelte` + `@unovis/ts` |
| `.npmrc` | Create | `legacy-peer-deps=true` |
| `src/app.css` | Modify | Add ~15 lines of Unovis CSS variable overrides |
| `src/lib/widgets/chart-theme.js` | Create | Shared emerald palette + `formatAmount` helper |
| `src/lib/widgets/SpendingByCategory.svelte` | Rewrite | CSS bars → `VisDonut` + legend |
| `src/lib/widgets/MonthlyTrend.svelte` | Rewrite | CSS bars → `VisGroupedBar` + axes |

Zero backend changes. No new IPC commands, no Rust changes, no schema changes.

## Verification

1. `npm install` succeeds (with `--legacy-peer-deps` or `.npmrc`)
2. `npm run dev` — both widgets render correctly
3. SpendingByCategory: donut segments colored by category, hover shows tooltip, center shows total
4. MonthlyTrend: bars with rounded corners, Y-axis gridlines, hover shows tooltip
5. Both charts match dark theme (gray-900 cards, emerald accents)
6. No console errors (deprecation warnings for `$$Generic` are expected)

## Fallback Plan

If `@unovis/svelte` components fail under Svelte 5, use `@unovis/ts` directly with imperative mounting:

```svelte
<script>
  import { onMount } from "svelte";
  import { SingleContainer, Donut } from "@unovis/ts";

  let chartEl;
  let container;

  onMount(() => {
    const donut = new Donut({ value: d => d.amount });
    container = new SingleContainer(chartEl, { component: donut }, data);
    return () => container?.destroy();
  });

  $effect(() => {
    container?.setData(sortedCategories);
  });
</script>

<div bind:this={chartEl} style="height: 180px;"></div>
```

This avoids the Svelte wrapper entirely and uses `$effect` for reactivity. More verbose but eliminates all compatibility concerns.

### Risk Matrix

| Risk | Likelihood | Mitigation |
|------|-----------|------------|
| `@unovis/svelte` fails to compile under Svelte 5 | Low | Svelte 5 has full backward compat. Fallback: `@unovis/ts` imperatively. |
| Tooltip doesn't work inside `<button>` wrapper | Medium | Change to `<div role="button" tabindex="0">` or set tooltip `container` to `document.body`. |
| Peer dependency warning on every `npm install` | Certain | `.npmrc` with `legacy-peer-deps=true`. |
| Chart renders at 0 height | Low | Fixed `height={180}` on containers. |
| Performance with 10k+ expenses | Low | Aggregation in `$derived` — charts see at most ~20 data points. |
