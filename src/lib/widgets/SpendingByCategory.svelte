<script>
  import { VisDonut, VisTooltip, VisSingleContainer } from "@unovis/svelte";
  import { Donut } from "@unovis/ts";
  import { CHART_PALETTE, chartColor, formatAmount } from "./chart-theme.js";
  import EmptyState from "../EmptyState.svelte";
  import { DATE_RANGE_PRESETS } from "../constants.js";

  let { expenses, config = {}, onnavigate = () => {}, onconfigchange = () => {} } = $props();

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
  let legendItems = $derived(sortedCategories.slice(0, legendMax));
  let overflowCount = $derived(Math.max(0, sortedCategories.length - legendMax));

  const triggers = {
    [Donut.selectors.segment]: (d) =>
      `<span style="font-weight:500">${d.data.label}</span><br/>${formatAmount(d.data.value)}`,
  };
</script>

<button
  onclick={() => onnavigate("categories")}
  class="bg-gray-900 rounded-xl p-6 border border-gray-800 w-full text-left
         cursor-pointer hover:border-amber-500/50 hover:bg-gray-900/80 transition-all card-hover"
>
  <div class="flex items-center justify-between mb-4">
    <h3 class="text-lg font-semibold">Spending by Category</h3>
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div
      class="flex items-center gap-0.5 bg-gray-800 rounded-lg p-0.5"
      onclick={(e) => e.stopPropagation()}
    >
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
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <div onclick={(e) => e.stopPropagation()} style="pointer-events: auto;">
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
        <span class="text-xs text-gray-500">+{overflowCount} more</span>
      {/if}
    </div>
  {:else}
    <EmptyState title="No data for this period." variant="widget" />
  {/if}
</button>
