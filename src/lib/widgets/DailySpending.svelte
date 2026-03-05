<script>
  import { VisXYContainer, VisArea, VisLine, VisAxis, VisTooltip } from "@unovis/svelte";
  import { Area } from "@unovis/ts";
  import { CHART_PALETTE, formatAmount } from "./chart-theme.js";
  import EmptyState from "../EmptyState.svelte";

  let { expenses, config = {}, onconfigchange = () => {} } = $props();

  const MONTH_NAMES = ["Jan","Feb","Mar","Apr","May","Jun","Jul","Aug","Sep","Oct","Nov","Dec"];

  function today() {
    return new Date().toISOString().slice(0, 7);
  }

  let selectedMonth = $derived(config.month ?? today());

  /** All distinct months that have expenses, sorted. */
  let availableMonths = $derived.by(() => {
    const set = new Set();
    for (const e of expenses) {
      const m = e.date?.slice(0, 7);
      if (m) set.add(m);
    }
    return [...set].sort();
  });

  let canPrev = $derived(availableMonths.indexOf(selectedMonth) > 0);
  let canNext = $derived.by(() => {
    const idx = availableMonths.indexOf(selectedMonth);
    return idx >= 0 && idx < availableMonths.length - 1;
  });

  function navigate(dir) {
    const idx = availableMonths.indexOf(selectedMonth);
    const next = availableMonths[idx + dir];
    if (next) onconfigchange({ ...config, month: next });
  }

  function formatMonthLabel(ym) {
    const [y, m] = ym.split("-");
    return `${MONTH_NAMES[parseInt(m) - 1]} ${y}`;
  }

  function daysInMonth(ym) {
    const [y, m] = ym.split("-").map(Number);
    return new Date(y, m, 0).getDate();
  }

  let dailyData = $derived.by(() => {
    const filtered = expenses.filter((e) => e.date?.slice(0, 7) === selectedMonth);
    if (filtered.length === 0) return [];

    const days = daysInMonth(selectedMonth);
    const totals = new Array(days).fill(0);
    for (const e of filtered) {
      const day = parseInt(e.date.slice(8, 10));
      if (day >= 1 && day <= days) {
        totals[day - 1] += Math.abs(e.amount);
      }
    }
    return totals.map((amount, i) => ({ day: i + 1, amount }));
  });

  let average = $derived.by(() => {
    if (dailyData.length === 0) return 0;
    const total = dailyData.reduce((s, d) => s + d.amount, 0);
    return total / dailyData.length;
  });

  let averageData = $derived(dailyData.map((d) => ({ ...d, avg: average })));

  const x = (d) => d.day;
  const yAmount = [(d) => d.amount];
  const yAvg = [(d) => d.avg];

  const xTickFormat = (v) => {
    const n = Math.round(v);
    if (n === 1 || n % 5 === 0) return String(n);
    return "";
  };

  const triggers = {
    [Area.selectors.area]: (d) => {
      const delta = d.amount - average;
      const sign = delta >= 0 ? "+" : "";
      return `<span style="font-weight:500">Day ${d.day}</span><br/>`
        + `${formatAmount(d.amount)}<br/>`
        + `<span style="color:${delta >= 0 ? "#f59e0b" : "#6b7280"}">${sign}${formatAmount(delta)} vs avg</span>`;
    },
  };
</script>

<div class="bg-gray-900 rounded-xl p-6 border border-gray-800">
  <div class="flex items-center justify-between mb-4">
    <h3 class="text-lg font-semibold">Daily Spending</h3>
    <div class="flex items-center gap-2">
      <button
        onclick={() => navigate(-1)}
        disabled={!canPrev}
        class="p-1 rounded transition-colors {canPrev ? 'text-gray-400 hover:text-gray-200' : 'text-gray-700 cursor-not-allowed'}"
      >
        <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="15 18 9 12 15 6"/></svg>
      </button>
      <span class="text-sm text-gray-300 min-w-[5rem] text-center">{formatMonthLabel(selectedMonth)}</span>
      <button
        onclick={() => navigate(1)}
        disabled={!canNext}
        class="p-1 rounded transition-colors {canNext ? 'text-gray-400 hover:text-gray-200' : 'text-gray-700 cursor-not-allowed'}"
      >
        <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="9 18 15 12 9 6"/></svg>
      </button>
    </div>
  </div>

  {#if dailyData.length > 0}
    {#key selectedMonth}
    <VisXYContainer data={averageData} height={180} padding={{ top: 10 }}>
      <VisArea
        {x}
        y={yAmount}
        color={CHART_PALETTE[0]}
        opacity={0.15}
        curveType="monotoneX"
      />
      <VisLine
        {x}
        y={yAmount}
        color={CHART_PALETTE[0]}
        curveType="monotoneX"
      />
      <VisLine
        {x}
        y={yAvg}
        color="#6b7280"
        lineDashArray={[4, 4]}
      />
      <VisAxis
        type="x"
        tickFormat={xTickFormat}
        numTicks={dailyData.length}
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

    <p class="text-sm text-gray-400 mt-2">
      Avg: {formatAmount(average)} / day
    </p>
  {:else}
    <EmptyState title="No expenses this month." variant="widget" />
  {/if}
</div>
