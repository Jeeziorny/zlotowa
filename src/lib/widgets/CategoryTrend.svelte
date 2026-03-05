<script>
  import { VisXYContainer, VisLine, VisAxis, VisTooltip, VisScatter } from "@unovis/svelte";
  import { Line } from "@unovis/ts";
  import { CHART_PALETTE, formatAmount } from "./chart-theme.js";
  import EmptyState from "../EmptyState.svelte";
  import { DATE_RANGE_PRESETS } from "../constants.js";

  const TOP_N = 3;
  const LINE_COLORS = [CHART_PALETTE[0], CHART_PALETTE[2], CHART_PALETTE[4]];

  let { expenses, config = {}, onconfigchange = () => {} } = $props();

  let activePreset = $derived(config.dateRange ?? "6M");

  function selectPreset(label) {
    onconfigchange({ ...config, dateRange: label });
  }

  let filteredExpenses = $derived.by(() => {
    const preset = DATE_RANGE_PRESETS.find((p) => p.label === activePreset) ?? DATE_RANGE_PRESETS[1];
    if (preset.months === null) return expenses;
    const cutoff = new Date();
    cutoff.setDate(1);
    cutoff.setMonth(cutoff.getMonth() - (preset.months - 1));
    const cutoffStr = cutoff.toISOString().slice(0, 10);
    return expenses.filter((e) => e.date >= cutoffStr);
  });

  let topCategories = $derived.by(() => {
    const totals = {};
    for (const e of filteredExpenses) {
      const cat = e.category || "Uncategorized";
      totals[cat] = (totals[cat] || 0) + Math.abs(e.amount);
    }
    return Object.entries(totals)
      .sort((a, b) => b[1] - a[1])
      .slice(0, TOP_N)
      .map(([name]) => name);
  });

  let monthlyData = $derived.by(() => {
    const map = {};
    for (const e of filteredExpenses) {
      const month = e.date?.slice(0, 7);
      if (!month) continue;
      const cat = e.category || "Uncategorized";
      if (!topCategories.includes(cat)) continue;
      if (!map[month]) map[month] = {};
      map[month][cat] = (map[month][cat] || 0) + Math.abs(e.amount);
    }

    return Object.entries(map)
      .sort((a, b) => a[0].localeCompare(b[0]))
      .map(([ym, cats], index) => ({ index, ym, ...cats }));
  });

  const MONTH_NAMES = ["Jan","Feb","Mar","Apr","May","Jun","Jul","Aug","Sep","Oct","Nov","Dec"];

  function formatMonthFull(ym) {
    const [y, m] = ym.split("-");
    return `${MONTH_NAMES[parseInt(m) - 1]} ${y.slice(2)}`;
  }

  function formatMonthSmart(ym, i) {
    const [y, m] = ym.split("-");
    const name = MONTH_NAMES[parseInt(m) - 1];
    if (i === 0) return `${name} ${y.slice(2)}`;
    const prevYm = monthlyData[i - 1]?.ym;
    if (prevYm && prevYm.slice(0, 4) !== y) return `${name} ${y.slice(2)}`;
    return name;
  }

  const x = (d) => d.index;

  let yAccessors = $derived(
    topCategories.map((cat) => (d) => d[cat] || 0)
  );

  const color = (_d, i) => LINE_COLORS[i] ?? CHART_PALETTE[0];

  const xTickFormat = (i) => {
    const item = monthlyData[i];
    return item ? formatMonthSmart(item.ym, i) : "";
  };

  const triggers = {
    [Line.selectors.line]: (d, i) => {
      const cat = topCategories[i];
      const amount = d[cat] || 0;
      return `<span style="font-weight:500">${formatMonthFull(d.ym)}</span><br/>`
        + `<span style="color:${LINE_COLORS[i]}">${cat}</span>: ${formatAmount(amount)}`;
    },
  };
</script>

<div class="bg-gray-900 rounded-xl p-6 border border-gray-800">
  <div class="flex items-center justify-between mb-4">
    <h3 class="text-lg font-semibold">Category Trend</h3>
    <div class="flex items-center gap-0.5 bg-gray-800 rounded-lg p-0.5">
      {#each DATE_RANGE_PRESETS as preset}
        <button
          onclick={() => selectPreset(preset.label)}
          class="px-2.5 py-1 text-xs rounded-md transition-colors {activePreset === preset.label
            ? 'bg-amber-500 text-gray-950 font-medium'
            : 'text-gray-400 hover:text-gray-200'}"
        >
          {preset.label}
        </button>
      {/each}
    </div>
  </div>

  {#if monthlyData.length > 0}
    {#key activePreset}
    <VisXYContainer data={monthlyData} height={200} padding={{ top: 10 }}>
      <VisLine
        {x}
        y={yAccessors}
        {color}
        curveType="monotoneX"
        lineWidth={2}
      />
      <VisScatter
        {x}
        y={yAccessors}
        {color}
        size={4}
      />
      <VisAxis
        type="x"
        tickFormat={xTickFormat}
        numTicks={monthlyData.length}
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

    <div class="flex flex-wrap gap-x-4 gap-y-1 mt-3">
      {#each topCategories as cat, i}
        <div class="flex items-center gap-1.5 text-xs text-gray-400">
          <span
            class="inline-block w-2.5 h-2.5 rounded-full shrink-0"
            style="background:{LINE_COLORS[i]}"
          ></span>
          {cat}
        </div>
      {/each}
    </div>
  {:else}
    <EmptyState title="No data for this period." variant="widget" />
  {/if}
</div>
