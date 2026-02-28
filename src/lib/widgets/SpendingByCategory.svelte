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
         cursor-pointer hover:border-amber-500/50 hover:bg-gray-900/80 transition-all"
>
  <h3 class="text-lg font-semibold mb-4">Spending by Category</h3>

  {#if sortedCategories.length > 0}
    <div class="space-y-3">
      {#each sortedCategories as [category, amount]}
        {@const pct = totalExpenses > 0 ? (amount / totalExpenses) * 100 : 0}
        <div class="group rounded-lg px-1 -mx-1 transition-all duration-200 hover:brightness-110">
          <div class="flex justify-between text-sm mb-1">
            <span class="text-gray-300">{category}</span>
            <span class="text-gray-400">{amount.toFixed(2)}</span>
          </div>
          <div class="flex items-center gap-2">
            <div class="flex-1 bg-gray-800 rounded-full h-3">
              <div
                class="bg-gradient-to-r from-amber-600 to-amber-400 h-3 rounded-full transition-all duration-200"
                style="width: {pct}%"
              ></div>
            </div>
            <span class="text-xs text-gray-400 opacity-0 group-hover:opacity-100 transition-opacity duration-200 w-10 text-right">
              {pct.toFixed(0)}%
            </span>
          </div>
        </div>
      {/each}
    </div>
  {:else}
    <p class="text-sm text-gray-500">No data yet.</p>
  {/if}
</button>
