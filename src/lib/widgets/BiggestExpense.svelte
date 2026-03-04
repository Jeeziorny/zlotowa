<script>
  import EmptyState from "../EmptyState.svelte";
  let { expenses } = $props();

  function getNow() {
    return new Date();
  }
  let currentMonth = $derived.by(() => {
    const now = getNow();
    return `${now.getFullYear()}-${String(now.getMonth() + 1).padStart(2, "0")}`;
  });

  let thisMonthExpenses = $derived(
    expenses.filter((e) => e.date?.startsWith(currentMonth))
  );

  let biggest = $derived.by(() => {
    if (thisMonthExpenses.length === 0) return null;
    return thisMonthExpenses.reduce((max, e) =>
      Math.abs(e.amount) > Math.abs(max.amount) ? e : max
    );
  });
</script>

<div class="bg-gray-900 rounded-xl p-6 border border-gray-800">
  <h3 class="text-lg font-semibold mb-4">Biggest Expense This Month</h3>

  {#if biggest}
    <div class="text-3xl font-bold text-amber-400 mb-2">
      {Math.abs(biggest.amount).toFixed(2)}
    </div>
    <div class="text-gray-300">{biggest.title}</div>
    <div class="text-sm text-gray-500 mt-1">
      {biggest.date}
      {#if biggest.category}
        <span class="ml-2 px-2 py-0.5 rounded bg-amber-900/30 text-amber-400 text-xs">
          {biggest.category}
        </span>
      {/if}
    </div>
  {:else}
    <EmptyState title="No expenses this month." variant="widget" />
  {/if}
</div>
