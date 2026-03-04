<script>
  import EmptyState from "../EmptyState.svelte";

  let { expenses, config } = $props();

  let keyword = $derived(config?.keyword || "");

  let keywords = $derived(
    keyword.split(",").map((k) => k.trim().toLowerCase()).filter(Boolean)
  );

  let matchingExpenses = $derived.by(() => {
    if (keywords.length === 0) return [];
    return expenses.filter((e) => {
      const title = (e.title || "").toLowerCase();
      return keywords.some((kw) => title.includes(kw));
    });
  });

  let monthlyData = $derived.by(() => {
    const months = {};
    for (const e of matchingExpenses) {
      const month = e.date?.slice(0, 7);
      if (month) {
        months[month] = (months[month] || 0) + Math.abs(e.amount);
      }
    }
    return Object.entries(months)
      .sort((a, b) => a[0].localeCompare(b[0]))
      .slice(-6);
  });

  let maxAmount = $derived.by(() => {
    if (monthlyData.length === 0) return 1;
    return Math.max(...monthlyData.map(([, v]) => v));
  });

  let totalAmount = $derived(
    matchingExpenses.reduce((sum, e) => sum + Math.abs(e.amount), 0)
  );

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
  <h3 class="text-lg font-semibold mb-4 flex items-center gap-2 flex-wrap">
    <span>Keyword{keywords.length > 1 ? "s" : ""}:</span>
    {#if keywords.length === 0}
      <span class="text-gray-500">—</span>
    {:else}
      {#each keywords as kw}
        <span class="text-sm font-medium text-amber-400 bg-amber-400/10 px-2 py-0.5 rounded">{kw}</span>
      {/each}
    {/if}
  </h3>

  {#if !keyword}
    <EmptyState title="No keyword configured." variant="widget" />
  {:else if matchingExpenses.length === 0}
    <EmptyState title={`No expenses matching "${keyword}".`} variant="widget" />
  {:else}
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

    <p class="text-sm text-gray-400 mt-4">
      {matchingExpenses.length} transaction{matchingExpenses.length === 1 ? "" : "s"}
      &middot; {totalAmount.toFixed(2)} total
    </p>
  {/if}
</div>
