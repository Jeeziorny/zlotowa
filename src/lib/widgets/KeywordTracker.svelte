<script>
  import { VisXYContainer, VisGroupedBar, VisAxis, VisTooltip } from "@unovis/svelte";
  import { GroupedBar } from "@unovis/ts";
  import { CHART_PALETTE, formatAmount } from "./chart-theme.js";
  import EmptyState from "../EmptyState.svelte";
  import { DATE_RANGE_PRESETS } from "../constants.js";

  let { expenses, config = {}, onconfigchange = () => {} } = $props();

  let keyword = $derived(config?.keyword || "");
  let activePreset = $derived(config?.dateRange ?? "6M");

  function selectPreset(label) {
    onconfigchange({ ...config, dateRange: label });
  }

  let keywords = $derived(
    keyword.split(",").map((k) => k.trim().toLowerCase()).filter(Boolean)
  );

  let matchingExpenses = $derived.by(() => {
    if (keywords.length === 0) return [];
    let filtered = expenses.filter((e) => {
      const title = (e.title || "").toLowerCase();
      return keywords.some((kw) => title.includes(kw));
    });
    const preset = DATE_RANGE_PRESETS.find((p) => p.label === activePreset) ?? DATE_RANGE_PRESETS[1];
    if (preset.months !== null) {
      const cutoff = new Date();
      cutoff.setDate(1);
      cutoff.setMonth(cutoff.getMonth() - (preset.months - 1));
      const cutoffStr = cutoff.toISOString().slice(0, 10);
      filtered = filtered.filter((e) => e.date >= cutoffStr);
    }
    return filtered;
  });

  let monthlyData = $derived.by(() => {
    const totals = {};
    for (const e of matchingExpenses) {
      const month = e.date?.slice(0, 7);
      if (month) {
        totals[month] = (totals[month] || 0) + Math.abs(e.amount);
      }
    }
    const months = Object.keys(totals);
    if (months.length === 0) return [];
    // Build full month range from earliest to latest
    const sorted = months.sort();
    const [startY, startM] = sorted[0].split("-").map(Number);
    const [endY, endM] = sorted[sorted.length - 1].split("-").map(Number);
    const result = [];
    let y = startY, m = startM, index = 0;
    while (y < endY || (y === endY && m <= endM)) {
      const ym = `${y}-${String(m).padStart(2, "0")}`;
      result.push({ index, ym, amount: totals[ym] || 0 });
      index++;
      m++;
      if (m > 12) { m = 1; y++; }
    }
    return result;
  });

  let totalAmount = $derived(
    matchingExpenses.reduce((sum, e) => sum + Math.abs(e.amount), 0)
  );

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
    <h3 class="text-lg font-semibold flex items-center gap-2 flex-wrap">
      <span>Keyword{keywords.length > 1 ? "s" : ""}:</span>
      {#if keywords.length === 0}
        <span class="text-gray-500">—</span>
      {:else}
        {#each keywords as kw}
          <span class="text-sm font-medium text-amber-400 bg-amber-400/10 px-2 py-0.5 rounded">{kw}</span>
        {/each}
      {/if}
    </h3>
    {#if keyword}
      <div class="flex items-center gap-0.5 bg-gray-800 rounded-lg p-0.5 shrink-0">
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
    {/if}
  </div>

  {#if !keyword}
    <EmptyState title="No keyword configured." variant="widget" />
  {:else if matchingExpenses.length === 0}
    <EmptyState title={`No expenses matching "${keyword}".`} variant="widget" />
  {:else}
    {#key activePreset}
    <VisXYContainer data={monthlyData} height={130} padding={{ top: 10 }}>
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
    {/key}

    <p class="text-sm text-gray-400 mt-4">
      {matchingExpenses.length} transaction{matchingExpenses.length === 1 ? "" : "s"}
      &middot; {totalAmount.toFixed(2)} total
    </p>
  {/if}
</div>
