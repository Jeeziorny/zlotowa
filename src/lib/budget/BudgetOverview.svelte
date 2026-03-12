<script>
  import { invoke } from "@tauri-apps/api/core";
  import { onDestroy } from "svelte";
  import ConfirmModal from "../ConfirmModal.svelte";

  let {
    budgetId,
    startDate,
    endDate,
    categories,
    budgetCategories,
    totalBudgeted,
    totalSpent,
    allCategories,
    onrefresh,
  } = $props();

  let remaining = $derived(totalBudgeted - totalSpent);
  let overBudgetCount = $derived(
    categories.filter((c) => c.status === "over").length,
  );

  // Budget category editing
  let editAmounts = $state({});
  let saving = $state(false);
  let saveMsg = $state("");
  let saveMsgTimer;

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
      clearTimeout(saveMsgTimer);
      saveMsgTimer = setTimeout(() => (saveMsg = ""), 2000);
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

  onDestroy(() => clearTimeout(saveMsgTimer));

  let showDeleteBudgetModal = $state(false);

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
      class="text-xs text-gray-600 hover:text-red-400 transition-colors"
      aria-label="Delete budget"
    >
      Delete Budget
    </button>
  </div>

  <!-- Total summary -->
  <div class="bg-gray-900 rounded-xl p-6 border border-gray-800">
    <h3 class="text-lg font-semibold mb-4">Summary</h3>
    <div class="grid grid-cols-1 sm:grid-cols-3 gap-4 text-center">
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
          {remaining >= 0 ? '+' : '−'}{Math.abs(remaining).toFixed(2)}
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
            <div class="flex items-center gap-2">
              <div class="flex-1 bg-gray-800 rounded-full h-2.5">
                <div
                  class="{barColor(cat.status)} h-2.5 rounded-full transition-all"
                  style="width: {barWidth(cat.spent, cat.budgeted)}%"
                ></div>
              </div>
              <span class="text-xs font-medium w-10 text-right {cat.status === 'over' ? 'text-red-400' : cat.status === 'approaching' ? 'text-amber-400' : 'text-emerald-400'}">
                {cat.status === 'over' ? 'Over' : cat.status === 'approaching' ? '80%+' : 'OK'}
              </span>
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
                             focus:border-amber-500"
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
              class="px-5 py-2 bg-amber-500 hover:bg-amber-400 disabled:opacity-50
                     text-gray-950 text-sm font-medium rounded-lg transition-colors"
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


</div>

<!-- Delete budget confirmation modal -->
{#if showDeleteBudgetModal}
  <ConfirmModal
    title="Delete budget?"
    onconfirm={async () => {
      await invoke("delete_budget", { id: budgetId });
      showDeleteBudgetModal = false;
      onrefresh();
    }}
    onclose={() => { showDeleteBudgetModal = false; }}
  >
    <p class="text-sm text-gray-400 mb-1">This will delete this budget and all its data.</p>
    <p class="text-sm text-gray-300 font-mono">{startDate} — {endDate}</p>
  </ConfirmModal>
{/if}
