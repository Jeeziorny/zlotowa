<script>
  import { invoke } from "@tauri-apps/api/core";

  let {
    budgetId,
    startDate,
    endDate,
    categories,
    budgetCategories,
    calendarEvents,
    totalBudgeted,
    totalSpent,
    totalCalendar,
    allCategories,
    onrefresh,
  } = $props();

  let remaining = $derived(totalBudgeted - totalSpent);
  let overBudgetCount = $derived(
    categories.filter((c) => c.status === "over").length,
  );

  // Error states
  let deleteBudgetError = $state("");

  // Budget category editing
  let editAmounts = $state({});
  let saving = $state(false);
  let saveMsg = $state("");

  $effect(() => {
    const amounts = {};
    for (const bc of budgetCategories) {
      amounts[bc.category] = bc.amount;
    }
    editAmounts = amounts;
  });

  async function saveBudgetCategories() {
    saving = true;
    saveMsg = "";
    try {
      const cats = Object.entries(editAmounts)
        .filter(([_, amount]) => amount > 0)
        .map(([category, amount]) => ({ category, amount: Number(amount) }));
      await invoke("save_budget_categories", { budgetId, categories: cats });
      saveMsg = "Saved";
      setTimeout(() => (saveMsg = ""), 2000);
      onrefresh();
    } catch (err) {
      saveMsg = `Error: ${err}`;
    }
    saving = false;
  }

  function barWidth(spent, budgeted) {
    if (budgeted <= 0) return 0;
    return Math.min((spent / budgeted) * 100, 100);
  }

  function barColor(status) {
    if (status === "over") return "bg-red-500";
    if (status === "approaching") return "bg-amber-500";
    return "bg-emerald-500";
  }

  let deleting = $state(false);
  let showDeleteBudgetModal = $state(false);
  async function deleteBudget() {
    deleting = true;
    deleteBudgetError = "";
    try {
      await invoke("delete_budget", { id: budgetId });
      showDeleteBudgetModal = false;
      onrefresh();
    } catch (err) {
      deleteBudgetError = `Failed to delete budget: ${err}`;
    }
    deleting = false;
  }

  let calendarEventsWithAmounts = $derived(
    calendarEvents.filter((e) => e.amount != null && e.amount > 0),
  );
</script>

<div class="space-y-6">
  <!-- Header with date range -->
  <div class="flex items-center justify-between">
    <div>
      <span class="text-sm text-gray-400 font-mono"
        >{startDate} — {endDate}</span
      >
    </div>
    <button
      onclick={() => showDeleteBudgetModal = true}
      disabled={deleting}
      class="text-xs text-gray-600 hover:text-red-400 transition-colors"
    >
      Delete Budget
    </button>
  </div>

  <!-- Total summary -->
  <div class="bg-gray-900 rounded-xl p-6 border border-gray-800">
    <h3 class="text-lg font-semibold mb-4">Summary</h3>
    <div class="grid grid-cols-3 gap-4 text-center">
      <div>
        <div class="text-xs text-gray-500 mb-1">Budgeted</div>
        <div class="text-xl font-bold font-mono">
          {totalBudgeted.toFixed(2)}
        </div>
      </div>
      <div>
        <div class="text-xs text-gray-500 mb-1">Spent</div>
        <div class="text-xl font-bold font-mono">{totalSpent.toFixed(2)}</div>
      </div>
      <div>
        <div class="text-xs text-gray-500 mb-1">
          {remaining >= 0 ? "Remaining" : "Over budget"}
        </div>
        <div
          class="text-xl font-bold font-mono {remaining >= 0
            ? 'text-emerald-400'
            : 'text-red-400'}"
        >
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

  <!-- Per-category progress -->
  <div class="bg-gray-900 rounded-xl p-6 border border-gray-800">
    <h3 class="text-lg font-semibold mb-4">Budget by Category</h3>

    {#if categories.length > 0}
      <div class="space-y-4 mb-6">
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

      <!-- Edit category budgets -->
      <details class="group">
        <summary
          class="text-sm text-gray-500 hover:text-gray-300 cursor-pointer transition-colors"
        >
          Edit category budgets
        </summary>
        <div class="mt-4">
          <table class="w-full text-sm mb-4">
            <thead>
              <tr class="border-b border-gray-700 text-gray-400">
                <th class="text-left px-3 py-2">Category</th>
                <th class="text-right px-3 py-2">Budget</th>
              </tr>
            </thead>
            <tbody>
              {#each Object.keys(editAmounts).sort() as cat}
                <tr class="border-b border-gray-800/50">
                  <td class="px-3 py-2 text-gray-300">{cat}</td>
                  <td class="px-3 py-2 text-right">
                    <input
                      type="number"
                      step="0.01"
                      min="0"
                      bind:value={editAmounts[cat]}
                      class="w-28 bg-gray-800 border border-gray-700 rounded px-2 py-1
                             text-right text-gray-100 font-mono focus:outline-none
                             focus:border-emerald-500"
                    />
                  </td>
                </tr>
              {/each}
            </tbody>
          </table>
          <div class="flex items-center gap-3">
            <button
              onclick={saveBudgetCategories}
              disabled={saving}
              class="px-5 py-2 bg-emerald-600 hover:bg-emerald-500 disabled:opacity-50
                     text-white text-sm font-medium rounded-lg transition-colors"
            >
              {saving ? "Saving..." : "Save Changes"}
            </button>
            {#if saveMsg}
              <span
                class="text-sm {saveMsg.startsWith('Error')
                  ? 'text-red-400'
                  : 'text-emerald-400'}"
              >
                {saveMsg}
              </span>
            {/if}
          </div>
        </div>
      </details>
    {:else}
      <p class="text-sm text-gray-500">
        No budget categories set. Edit category budgets above.
      </p>
    {/if}
  </div>

  <!-- Calendar amounts summary -->
  {#if calendarEventsWithAmounts.length > 0}
    <div class="bg-gray-900 rounded-xl p-6 border border-gray-800">
      <h3 class="text-lg font-semibold mb-3">Calendar Costs</h3>
      <div class="space-y-2">
        {#each calendarEventsWithAmounts as event}
          <div class="flex justify-between text-sm">
            <span class="text-gray-300">
              {event.summary}
              <span class="text-xs text-gray-500 ml-1"
                >({event.start_date})</span
              >
            </span>
            <span class="text-gray-400 font-mono"
              >{event.amount.toFixed(2)}</span
            >
          </div>
        {/each}
      </div>
      <div
        class="mt-3 pt-3 border-t border-gray-800 flex justify-between text-sm font-medium"
      >
        <span class="text-gray-300">Total calendar costs</span>
        <span class="font-mono">{totalCalendar.toFixed(2)}</span>
      </div>
    </div>
  {/if}

</div>

<!-- Delete budget confirmation modal -->
{#if showDeleteBudgetModal}
  <div class="fixed inset-0 bg-black/60 flex items-center justify-center z-50"
       role="presentation"
       onclick={(e) => { if (e.target === e.currentTarget && !deleting) { showDeleteBudgetModal = false; deleteBudgetError = ""; } }}
       onkeydown={(e) => { if (e.key === "Escape" && !deleting) { showDeleteBudgetModal = false; deleteBudgetError = ""; } }}>
    <div class="bg-gray-900 border border-gray-800 rounded-xl p-6 max-w-sm w-full mx-4 shadow-xl"
         role="dialog"
         aria-modal="true"
         aria-labelledby="delete-budget-modal-title">
      <h3 id="delete-budget-modal-title" class="text-lg font-semibold text-gray-100 mb-2">Delete budget?</h3>
      <p class="text-sm text-gray-400 mb-1">This will delete this budget and all its data.</p>
      <p class="text-sm text-gray-300 mb-5 font-mono">{startDate} — {endDate}</p>
      {#if deleteBudgetError}
        <div class="text-sm bg-red-900/50 text-red-400 px-4 py-2 rounded-lg mb-3">{deleteBudgetError}</div>
      {/if}
      <div class="flex gap-3 justify-end">
        <button
          onclick={() => { showDeleteBudgetModal = false; deleteBudgetError = ""; }}
          disabled={deleting}
          class="bg-gray-800 hover:bg-gray-700 text-gray-300 px-4 py-2 rounded-lg
                 text-sm transition-colors disabled:opacity-50"
        >
          Cancel
        </button>
        <button
          onclick={deleteBudget}
          disabled={deleting}
          class="bg-red-600 hover:bg-red-500 disabled:opacity-50 text-white px-4 py-2
                 rounded-lg text-sm font-medium transition-colors"
        >
          {deleting ? "Deleting..." : "Delete"}
        </button>
      </div>
    </div>
  </div>
{/if}
