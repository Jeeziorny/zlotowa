<script>
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";

  let expenses = $state([]);
  let totalExpenses = $state(0);
  let categoryCounts = $state({});

  let sortedCategories = $derived(
    Object.entries(categoryCounts).sort((a, b) => b[1] - a[1])
  );

  onMount(async () => {
    try {
      expenses = await invoke("get_expenses");
      totalExpenses = expenses.reduce((sum, e) => sum + Math.abs(e.amount), 0);

      const counts = {};
      for (const e of expenses) {
        const cat = e.category || "Uncategorized";
        counts[cat] = (counts[cat] || 0) + Math.abs(e.amount);
      }
      categoryCounts = counts;
    } catch (err) {
      console.error("Failed to load expenses:", err);
    }
  });
</script>

<div>
  <h2 class="text-2xl font-bold mb-6">Dashboard</h2>

  <div class="grid grid-cols-1 md:grid-cols-3 gap-6 mb-8">
    <div class="bg-gray-900 rounded-xl p-6 border border-gray-800">
      <div class="text-sm text-gray-400 mb-1">Total Expenses</div>
      <div class="text-3xl font-bold text-emerald-400">
        {totalExpenses.toFixed(2)}
      </div>
    </div>

    <div class="bg-gray-900 rounded-xl p-6 border border-gray-800">
      <div class="text-sm text-gray-400 mb-1">Transactions</div>
      <div class="text-3xl font-bold text-emerald-400">
        {expenses.length}
      </div>
    </div>

    <div class="bg-gray-900 rounded-xl p-6 border border-gray-800">
      <div class="text-sm text-gray-400 mb-1">Categories</div>
      <div class="text-3xl font-bold text-emerald-400">
        {Object.keys(categoryCounts).length}
      </div>
    </div>
  </div>

  {#if sortedCategories.length > 0}
    <div class="bg-gray-900 rounded-xl p-6 border border-gray-800">
      <h3 class="text-lg font-semibold mb-4">Spending by Category</h3>
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
                style="width: {(amount / totalExpenses) * 100}%"
              ></div>
            </div>
          </div>
        {/each}
      </div>
    </div>
  {:else}
    <div class="bg-gray-900 rounded-xl p-12 border border-gray-800 text-center text-gray-500">
      <p class="text-lg mb-2">No expenses yet</p>
      <p class="text-sm">Add an expense or do a bulk upload to get started.</p>
    </div>
  {/if}
</div>
