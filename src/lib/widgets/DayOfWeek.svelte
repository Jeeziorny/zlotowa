<script>
  import { VisXYContainer, VisGroupedBar, VisAxis, VisTooltip } from "@unovis/svelte";
  import { GroupedBar } from "@unovis/ts";
  import { CHART_PALETTE, formatAmount } from "./chart-theme.js";
  import EmptyState from "../EmptyState.svelte";
  import { DATE_RANGE_PRESETS } from "../constants.js";

  let { expenses, config = {}, onconfigchange = () => {} } = $props();

  const DAY_LABELS = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];
  const TODAY_DOW = (new Date().getDay() + 6) % 7; // JS Sun=0 -> Mon=0..Sun=6

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

  let weekdayData = $derived.by(() => {
    if (filteredExpenses.length === 0) return [];

    const totals = new Array(7).fill(0);
    const counts = new Array(7).fill(0);

    for (const e of filteredExpenses) {
      if (!e.date) continue;
      const d = new Date(e.date + "T12:00:00"); // noon to avoid timezone shifts
      const dow = (d.getDay() + 6) % 7; // Mon=0..Sun=6
      totals[dow] += Math.abs(e.amount);
      counts[dow] += 1;
    }

    return DAY_LABELS.map((label, i) => ({
      index: i,
      label,
      total: totals[i],
      count: counts[i],
      isToday: i === TODAY_DOW,
    }));
  });

  const x = (d) => d.index;
  const y = [(d) => d.total];
  const barColor = (d) => d.isToday ? "#fbbf24" : CHART_PALETTE[0]; // amber-400 vs amber-500
  const xTickFormat = (i) => DAY_LABELS[Math.round(i)] ?? "";

  const triggers = {
    [GroupedBar.selectors.bar]: (d) => {
      return `<span style="font-weight:500">${d.label}</span><br/>`
        + `${formatAmount(d.total)}<br/>`
        + `${d.count} transaction${d.count !== 1 ? "s" : ""}`;
    },
  };
</script>

<div class="bg-gray-900 rounded-xl p-6 border border-gray-800">
  <div class="flex items-center justify-between mb-4">
    <h3 class="text-lg font-semibold">Day of Week</h3>
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

  {#if weekdayData.length > 0}
    {#key activePreset}
    <VisXYContainer data={weekdayData} height={180} padding={{ top: 10 }}>
      <VisGroupedBar
        {x}
        {y}
        color={barColor}
        roundedCorners={2}
        barPadding={0.2}
      />
      <VisAxis
        type="x"
        tickFormat={xTickFormat}
        numTicks={7}
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
  {:else}
    <EmptyState title="No expenses in this period." variant="widget" />
  {/if}
</div>
