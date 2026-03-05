<script>
  import { VisXYContainer, VisGroupedBar, VisAxis, VisTooltip } from "@unovis/svelte";
  import { GroupedBar } from "@unovis/ts";
  import { CHART_PALETTE, formatAmount } from "./chart-theme.js";
  import EmptyState from "../EmptyState.svelte";
  import { DATE_RANGE_PRESETS } from "../constants.js";

  let { expenses, config = {}, onconfigchange = () => {} } = $props();

  let activePreset = $derived(config.dateRange ?? "6M");

  function selectPreset(label) {
    onconfigchange({ ...config, dateRange: label });
  }

  let filteredExpenses = $derived.by(() => {
    const preset = DATE_RANGE_PRESETS.find((p) => p.label === activePreset) ?? DATE_RANGE_PRESETS[1];
    if (preset.months === null) return expenses;
    const cutoff = new Date();
    cutoff.setMonth(cutoff.getMonth() - preset.months);
    const cutoffStr = cutoff.toISOString().slice(0, 10);
    return expenses.filter((e) => e.date >= cutoffStr);
  });

  let monthlyData = $derived.by(() => {
    const months = {};
    for (const e of filteredExpenses) {
      const month = e.date?.slice(0, 7);
      if (month) {
        months[month] = (months[month] || 0) + Math.abs(e.amount);
      }
    }
    return Object.entries(months)
      .sort((a, b) => a[0].localeCompare(b[0]))
      .map(([ym, amount], index) => ({ index, ym, amount }));
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
  const y = [(d) => d.amount];
  const color = () => CHART_PALETTE[0];

  const xTickFormat = (i) => {
    const item = monthlyData[i];
    return item ? formatMonthSmart(item.ym, i) : "";
  };

  const triggers = {
    [GroupedBar.selectors.bar]: (d) =>
      `<span style="font-weight:500">${formatMonthFull(d.ym)}</span><br/>${formatAmount(d.amount)}`,
  };
</script>

<div class="bg-gray-900 rounded-xl p-6 border border-gray-800">
  <div class="flex items-center justify-between mb-4">
    <h3 class="text-lg font-semibold">Monthly Trend</h3>
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
    <VisXYContainer data={monthlyData} height={180} padding={{ top: 10 }}>
      <VisGroupedBar
        {x}
        {y}
        {color}
        roundedCorners={3}
        barMinHeight={2}
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
  {:else}
    <EmptyState title="No data for this period." variant="widget" />
  {/if}
</div>
