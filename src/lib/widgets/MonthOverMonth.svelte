<script>
  import { VisXYContainer, VisGroupedBar, VisAxis, VisTooltip } from "@unovis/svelte";
  import { GroupedBar } from "@unovis/ts";
  import { CHART_PALETTE, formatAmount } from "./chart-theme.js";
  import { formatMonthLabel, formatMonthShort } from "../utils/dateFormat.js";
  import EmptyState from "../EmptyState.svelte";

  let { expenses, config = {}, onconfigchange = () => {} } = $props();

  const BAR_COLORS = [CHART_PALETTE[0], "#6b7280"]; // amber = current, gray = previous

  function todayYM() {
    return new Date().toISOString().slice(0, 7);
  }

  function prevYM(ym) {
    const [y, m] = ym.split("-").map(Number);
    const d = new Date(y, m - 2, 1);
    return d.toISOString().slice(0, 7);
  }

  let selectedMonth = $derived(config.month ?? todayYM());
  let previousMonth = $derived(prevYM(selectedMonth));

  let availableMonths = $derived.by(() => {
    const set = new Set();
    for (const e of expenses) {
      const m = e.date?.slice(0, 7);
      if (m) set.add(m);
    }
    return [...set].sort();
  });

  let canPrev = $derived.by(() => {
    const idx = availableMonths.indexOf(selectedMonth);
    return idx > 0;
  });
  let canNext = $derived.by(() => {
    const idx = availableMonths.indexOf(selectedMonth);
    return idx >= 0 && idx < availableMonths.length - 1;
  });

  function navigate(dir) {
    const idx = availableMonths.indexOf(selectedMonth);
    const next = availableMonths[idx + dir];
    if (next) onconfigchange({ ...config, month: next });
  }


  const TOP_N = 6;

  let categoryData = $derived.by(() => {
    const curExpenses = expenses.filter((e) => e.date?.slice(0, 7) === selectedMonth);
    const prevExpenses = expenses.filter((e) => e.date?.slice(0, 7) === previousMonth);

    if (curExpenses.length === 0 && prevExpenses.length === 0) return [];

    const curTotals = {};
    const prevTotals = {};
    for (const e of curExpenses) {
      const cat = e.category || "Uncategorized";
      curTotals[cat] = (curTotals[cat] || 0) + Math.abs(e.amount);
    }
    for (const e of prevExpenses) {
      const cat = e.category || "Uncategorized";
      prevTotals[cat] = (prevTotals[cat] || 0) + Math.abs(e.amount);
    }

    // Union of categories, sorted by combined total descending, capped at TOP_N
    const allCats = new Set([...Object.keys(curTotals), ...Object.keys(prevTotals)]);
    const sorted = [...allCats]
      .map((cat) => ({
        cat,
        combined: (curTotals[cat] || 0) + (prevTotals[cat] || 0),
      }))
      .sort((a, b) => b.combined - a.combined)
      .slice(0, TOP_N);

    return sorted.map(({ cat }, index) => ({
      index,
      cat,
      current: curTotals[cat] || 0,
      previous: prevTotals[cat] || 0,
    }));
  });

  let totalCurrent = $derived(categoryData.reduce((s, d) => s + d.current, 0));
  let totalPrevious = $derived(categoryData.reduce((s, d) => s + d.previous, 0));
  let totalDelta = $derived(totalCurrent - totalPrevious);
  let totalPct = $derived(totalPrevious > 0 ? ((totalDelta / totalPrevious) * 100) : 0);

  const x = (d) => d.index;
  const y = [(d) => d.current, (d) => d.previous];
  const color = (_d, i) => BAR_COLORS[i];

  const xTickFormat = (i) => {
    const item = categoryData[Math.round(i)];
    if (!item) return "";
    return item.cat.length > 10 ? item.cat.slice(0, 9) + "\u2026" : item.cat;
  };

  const triggers = {
    [GroupedBar.selectors.bar]: (d, i) => {
      const amount = i === 0 ? d.current : d.previous;
      const label = i === 0 ? formatMonthShort(selectedMonth) : formatMonthShort(previousMonth);
      const delta = d.current - d.previous;
      const sign = delta >= 0 ? "+" : "";
      return `<span style="font-weight:500">${d.cat}</span><br/>`
        + `<span style="color:${BAR_COLORS[i]}">${label}</span>: ${formatAmount(amount)}<br/>`
        + `<span style="color:${delta >= 0 ? "#f59e0b" : "#6b7280"}">${sign}${formatAmount(delta)}</span>`;
    },
  };
</script>

<div class="bg-gray-900 rounded-xl p-6 border border-gray-800">
  <div class="flex items-center justify-between mb-2">
    <h3 class="text-lg font-semibold">Month over Month</h3>
    <div class="flex items-center gap-2">
      <button
        onclick={() => navigate(-1)}
        disabled={!canPrev}
        class="p-1 rounded transition-colors {canPrev ? 'text-gray-400 hover:text-gray-200' : 'text-gray-700 cursor-not-allowed'}"
        aria-label="Previous month"
      >
        <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="15 18 9 12 15 6"/></svg>
      </button>
      <span class="text-sm text-gray-300 min-w-[5rem] text-center">{formatMonthLabel(selectedMonth)}</span>
      <button
        onclick={() => navigate(1)}
        disabled={!canNext}
        class="p-1 rounded transition-colors {canNext ? 'text-gray-400 hover:text-gray-200' : 'text-gray-700 cursor-not-allowed'}"
        aria-label="Next month"
      >
        <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="9 18 15 12 9 6"/></svg>
      </button>
    </div>
  </div>

  {#if categoryData.length > 0}
    <div class="flex items-baseline gap-2 mb-3">
      <span class="text-2xl font-bold {totalDelta > 0 ? 'text-amber-400' : totalDelta < 0 ? 'text-emerald-400' : 'text-gray-300'}">
        {totalDelta >= 0 ? "+" : ""}{formatAmount(totalDelta)}
      </span>
      {#if totalPrevious > 0}
        <span class="text-sm text-gray-500">
          ({totalDelta >= 0 ? "+" : ""}{totalPct.toFixed(1)}% vs {formatMonthShort(previousMonth)})
        </span>
      {/if}
    </div>

    {#key selectedMonth}
    <VisXYContainer data={categoryData} height={180} padding={{ top: 10 }}>
      <VisGroupedBar
        {x}
        {y}
        {color}
        roundedCorners={2}
        barPadding={0.2}
        groupPadding={0.3}
      />
      <VisAxis
        type="x"
        tickFormat={xTickFormat}
        numTicks={categoryData.length}
        gridLine={false}
        domainLine={false}
      />
      <VisAxis
        type="y"
        gridLine={true}
        domainLine={false}
        tickFormat={(v) => v.toFixed(0)}
      />
      <VisTooltip {triggers} />
    </VisXYContainer>
    {/key}

    <div class="flex items-center gap-4 mt-3">
      <div class="flex items-center gap-1.5 text-xs text-gray-400">
        <span class="inline-block w-2.5 h-2.5 rounded-full shrink-0" style="background:{BAR_COLORS[0]}"></span>
        {formatMonthShort(selectedMonth)}
      </div>
      <div class="flex items-center gap-1.5 text-xs text-gray-400">
        <span class="inline-block w-2.5 h-2.5 rounded-full shrink-0" style="background:{BAR_COLORS[1]}"></span>
        {formatMonthShort(previousMonth)}
      </div>
    </div>
  {:else}
    <EmptyState title="No expenses to compare." variant="widget" />
  {/if}
</div>
