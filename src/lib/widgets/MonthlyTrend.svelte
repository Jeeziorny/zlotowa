<script>
  import { VisXYContainer, VisGroupedBar, VisAxis, VisTooltip } from "@unovis/svelte";
  import { GroupedBar } from "@unovis/ts";
  import { CHART_PALETTE, formatAmount } from "./chart-theme.js";

  let { expenses } = $props();

  let monthlyData = $derived.by(() => {
    const months = {};
    for (const e of expenses) {
      const month = e.date?.slice(0, 7); // "YYYY-MM"
      if (month) {
        months[month] = (months[month] || 0) + Math.abs(e.amount);
      }
    }
    return Object.entries(months)
      .sort((a, b) => a[0].localeCompare(b[0]))
      .slice(-6)
      .map(([ym, amount], index) => ({ index, ym, amount }));
  });

  function formatMonth(ym) {
    const [y, m] = ym.split("-");
    const names = ["Jan","Feb","Mar","Apr","May","Jun","Jul","Aug","Sep","Oct","Nov","Dec"];
    return `${names[parseInt(m) - 1]} ${y.slice(2)}`;
  }

  const x = (d) => d.index;
  const y = [(d) => d.amount];
  const color = () => CHART_PALETTE[0];

  const xTickFormat = (i) => {
    const item = monthlyData[i];
    return item ? formatMonth(item.ym) : "";
  };

  const triggers = {
    [GroupedBar.selectors.bar]: (d) =>
      `<span style="font-weight:500">${formatMonth(d.ym)}</span><br/>${formatAmount(d.amount)}`,
  };
</script>

<div class="bg-gray-900 rounded-xl p-6 border border-gray-800">
  <h3 class="text-lg font-semibold mb-4">Monthly Trend</h3>

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
    <p class="text-sm text-gray-500">No data yet.</p>
  {/if}
</div>
