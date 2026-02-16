<script>
  let { categories, plannedExpenses, totalBudgeted, totalSpent, totalPlanned } = $props();

  let remaining = $derived(totalBudgeted - totalSpent);
  let overBudgetCount = $derived(categories.filter((c) => c.status === "over").length);

  function barWidth(spent, budgeted) {
    if (budgeted <= 0) return 0;
    return Math.min((spent / budgeted) * 100, 100);
  }

  function barColor(status) {
    if (status === "over") return "bg-red-500";
    if (status === "approaching") return "bg-amber-500";
    return "bg-emerald-500";
  }
</script>

<div class="space-y-6">
  <!-- Per-category progress -->
  <div class="bg-gray-900 rounded-xl p-6 border border-gray-800">
    <h3 class="text-lg font-semibold mb-4">Budget by Category</h3>

    {#if categories.length > 0}
      <div class="space-y-4">
        {#each categories as cat}
          <div>
            <div class="flex justify-between text-sm mb-1">
              <span class="text-gray-300">{cat.category}</span>
              <span class="text-gray-400 font-mono text-xs">
                {cat.spent.toFixed(2)} / {cat.budgeted.toFixed(2)}
                {#if cat.status === "over"}
                  <span class="text-red-400 ml-1">
                    OVER by {(cat.spent - cat.budgeted).toFixed(2)}
                  </span>
                {/if}
              </span>
            </div>
            <div class="w-full bg-gray-800 rounded-full h-2.5">
              <div
                class="{barColor(cat.status)} h-2.5 rounded-full transition-all"
                style="width: {barWidth(cat.spent, cat.budgeted)}%"
              ></div>
            </div>
          </div>
        {/each}
      </div>
    {:else}
      <p class="text-sm text-gray-500">No budget categories set. Go to the Budget tab to set limits.</p>
    {/if}
  </div>

  <!-- Planned expenses -->
  {#if plannedExpenses.length > 0}
    <div class="bg-gray-900 rounded-xl p-6 border border-gray-800">
      <h3 class="text-lg font-semibold mb-3">Planned Expenses</h3>
      <div class="space-y-2">
        {#each plannedExpenses as pe}
          <div class="flex justify-between text-sm">
            <span class="text-gray-300">
              {pe.title}
              {#if pe.category}
                <span class="text-xs text-gray-500 ml-1">({pe.category})</span>
              {/if}
            </span>
            <span class="text-gray-400 font-mono">{pe.amount.toFixed(2)}</span>
          </div>
        {/each}
      </div>
      <div class="mt-3 pt-3 border-t border-gray-800 flex justify-between text-sm font-medium">
        <span class="text-gray-300">Total planned</span>
        <span class="font-mono">{totalPlanned.toFixed(2)}</span>
      </div>
    </div>
  {/if}

  <!-- Total summary -->
  <div class="bg-gray-900 rounded-xl p-6 border border-gray-800">
    <h3 class="text-lg font-semibold mb-4">Summary</h3>
    <div class="grid grid-cols-3 gap-4 text-center">
      <div>
        <div class="text-xs text-gray-500 mb-1">Budgeted</div>
        <div class="text-xl font-bold font-mono">{totalBudgeted.toFixed(2)}</div>
      </div>
      <div>
        <div class="text-xs text-gray-500 mb-1">Spent</div>
        <div class="text-xl font-bold font-mono">{totalSpent.toFixed(2)}</div>
      </div>
      <div>
        <div class="text-xs text-gray-500 mb-1">
          {remaining >= 0 ? "Remaining" : "Over budget"}
        </div>
        <div class="text-xl font-bold font-mono {remaining >= 0 ? 'text-emerald-400' : 'text-red-400'}">
          {Math.abs(remaining).toFixed(2)}
        </div>
      </div>
    </div>

    {#if overBudgetCount > 0}
      <div class="mt-4 text-sm text-red-400 text-center">
        {overBudgetCount} categor{overBudgetCount === 1 ? "y" : "ies"} over budget
      </div>
    {/if}
  </div>
</div>
