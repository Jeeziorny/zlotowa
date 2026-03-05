<script>
  import { VisDonut, VisTooltip, VisSingleContainer } from "@unovis/svelte";
  import { Donut } from "@unovis/ts";
  import { CHART_PALETTE, chartColor, formatAmount } from "./chart-theme.js";
  import EmptyState from "../EmptyState.svelte";
  import { DATE_RANGE_PRESETS } from "../constants.js";

  let { expenses, config = {}, onconfigchange = () => {} } = $props();

  let chartReady = $state(false);
  $effect(() => {
    if (!chartReady) {
      const id = setTimeout(() => { chartReady = true; }, 100);
      return () => clearTimeout(id);
    }
  });

  let activePreset = $derived(config.dateRange ?? "All");

  function selectPreset(label) {
    onconfigchange({ ...config, dateRange: label });
  }

  let filteredExpenses = $derived.by(() => {
    const preset = DATE_RANGE_PRESETS.find((p) => p.label === activePreset) ?? DATE_RANGE_PRESETS.at(-1);
    if (preset.months === null) return expenses;
    const cutoff = new Date();
    cutoff.setMonth(cutoff.getMonth() - preset.months);
    const cutoffStr = cutoff.toISOString().slice(0, 10);
    return expenses.filter((e) => e.date >= cutoffStr);
  });

  let totalExpenses = $derived(
    filteredExpenses.reduce((sum, e) => sum + Math.abs(e.amount), 0)
  );

  let sortedCategories = $derived.by(() => {
    const counts = {};
    for (const e of filteredExpenses) {
      const cat = e.category || "Uncategorized";
      counts[cat] = (counts[cat] || 0) + Math.abs(e.amount);
    }
    return Object.entries(counts)
      .sort((a, b) => b[1] - a[1])
      .map(([label, value], i) => ({ label, value, color: chartColor(i) }));
  });

  const legendMax = 6;
  let legendExpanded = $state(false);
  let legendItems = $derived(legendExpanded ? sortedCategories : sortedCategories.slice(0, legendMax));
  let overflowCount = $derived(Math.max(0, sortedCategories.length - legendMax));

  const triggers = {
    [Donut.selectors.segment]: (d) =>
      `<span style="font-weight:500">${d.data.label}</span><br/>${formatAmount(d.data.value)}`,
  };
</script>

<div
  class="bg-gray-900 rounded-xl p-6 border border-gray-800 w-full text-left"
>
  <div class="flex items-center justify-between mb-4">
    <h3 class="text-lg font-semibold">Spending by Category</h3>
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

  {#if sortedCategories.length > 0}
    <div>
      <VisSingleContainer data={chartReady ? sortedCategories : sortedCategories.map(d => ({ ...d, value: 0 }))} height={180}>
        <VisDonut
          value={(d) => d.value}
          color={(d) => d.color}
          arcWidth={40}
          padAngle={0.02}
          cornerRadius={3}
          centralLabel={formatAmount(totalExpenses)}
          centralSubLabel="total"
        />
        <VisTooltip {triggers} />
      </VisSingleContainer>
    </div>

    <div class="mt-3 flex flex-wrap gap-x-4 gap-y-1">
      {#each legendItems as item}
        <div class="flex items-center gap-1.5 text-xs text-gray-400">
          <span class="w-2.5 h-2.5 rounded-sm shrink-0" style="background:{item.color}"></span>
          <span class="truncate max-w-28">{item.label}</span>
          <span class="text-gray-500">{formatAmount(item.value)}</span>
        </div>
      {/each}
      {#if overflowCount > 0}
        <button
          onclick={() => { legendExpanded = !legendExpanded; }}
          class="text-xs text-gray-500 hover:text-gray-300 transition-colors cursor-pointer"
        >
          {legendExpanded ? 'Show less' : `+${overflowCount} more`}
        </button>
      {/if}
    </div>
  {:else}
    <EmptyState title="No data for this period." variant="widget" />
  {/if}
</div>
