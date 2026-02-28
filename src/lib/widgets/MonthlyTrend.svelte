<script>
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
      .slice(-6); // last 6 months
  });

  let maxAmount = $derived.by(() => {
    const data = monthlyData;
    if (data.length === 0) return 1;
    return Math.max(...data.map(([, v]) => v));
  });

  function formatMonth(ym) {
    const [y, m] = ym.split("-");
    const months = [
      "Jan", "Feb", "Mar", "Apr", "May", "Jun",
      "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ];
    return `${months[parseInt(m) - 1]} ${y.slice(2)}`;
  }
</script>

<div class="bg-gray-900 rounded-xl p-6 border border-gray-800">
  <h3 class="text-lg font-semibold mb-4">Monthly Trend</h3>

  {#if monthlyData.length > 0}
    <div class="flex items-end gap-2 h-32">
      {#each monthlyData as [month, amount]}
        <div class="flex-1 flex flex-col items-center gap-1">
          <span class="text-xs text-gray-400">{amount.toFixed(0)}</span>
          <div
            class="w-full bg-amber-500 rounded-t transition-all min-h-1"
            style="height: {(amount / maxAmount) * 100}%"
          ></div>
          <span class="text-xs text-gray-500">{formatMonth(month)}</span>
        </div>
      {/each}
    </div>
  {:else}
    <p class="text-sm text-gray-500">No data yet.</p>
  {/if}
</div>
