<script>
  let { expenses, onnavigate = () => {} } = $props();

  let totalExpenses = $derived(
    expenses.reduce((sum, e) => sum + Math.abs(e.amount), 0)
  );

  let sortedCategories = $derived.by(() => {
    const counts = {};
    for (const e of expenses) {
      const cat = e.category || "Uncategorized";
      counts[cat] = (counts[cat] || 0) + Math.abs(e.amount);
    }
    return Object.entries(counts).sort((a, b) => b[1] - a[1]);
  });
</script>

<button
  onclick={() => onnavigate("categories")}
  class="bg-gray-900 rounded-xl p-6 border border-gray-800 w-full text-left
         cursor-pointer hover:border-emerald-500/50 hover:bg-gray-900/80 transition-all"
>
  <h3 class="text-lg font-semibold mb-4">Spending by Category</h3>

  {#if sortedCategories.length > 0}
    <div class="space-y-3">
      {#each sortedCategories as [category, amount]}
        <div>
          <div class="flex justify-between text-sm mb-1">
            <span class="text-gray-300">{category}</span>
            <span class="text-gray-400">{amount.toFixed(2)}</span>
          </div>
          <div class="w-full bg-gray-800 rounded-full h-2">
            <div
              class="bg-emerald-500 h-2 rounded-full transition-all"
              style="width: {totalExpenses > 0
                ? (amount / totalExpenses) * 100
                : 0}%"
            ></div>
          </div>
        </div>
      {/each}
    </div>
  {:else}
    <p class="text-sm text-gray-500">No data yet.</p>
  {/if}
</button>
