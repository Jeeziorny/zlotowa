<script>
  import { invoke } from "@tauri-apps/api/core";

  let { year, month, categories, averages, budgetCategories, plannedExpenses, onrefresh } = $props();

  // Local editable copy of budget category amounts
  let editAmounts = $state({});
  let saving = $state(false);
  let saveMsg = $state("");

  // Planned expense form
  let peTitle = $state("");
  let peAmount = $state("");
  let peDate = $state("");
  let peCategory = $state("");
  let peError = $state("");

  // Initialize edit amounts from budgetCategories
  $effect(() => {
    const amounts = {};
    for (const bc of budgetCategories) {
      amounts[bc.category] = bc.amount;
    }
    // Also include categories that have no budget yet
    for (const cat of categories) {
      if (!(cat in amounts)) {
        amounts[cat] = 0;
      }
    }
    editAmounts = amounts;
  });

  function getAverage(category) {
    const avg = averages.find((a) => a.category === category);
    return avg ? avg.average : null;
  }

  async function saveBudget() {
    saving = true;
    saveMsg = "";
    try {
      const cats = Object.entries(editAmounts)
        .filter(([_, amount]) => amount > 0)
        .map(([category, amount]) => ({ category, amount: Number(amount) }));
      await invoke("save_budget_categories", { year, month, categories: cats });
      saveMsg = "Saved";
      setTimeout(() => (saveMsg = ""), 2000);
      onrefresh();
    } catch (err) {
      saveMsg = `Error: ${err}`;
    }
    saving = false;
  }

  async function addPlannedExpense() {
    peError = "";
    if (!peTitle.trim() || !peAmount || !peDate) {
      peError = "Title, amount, and date are required.";
      return;
    }
    try {
      await invoke("add_planned_expense", {
        year, month,
        expense: {
          title: peTitle.trim(),
          amount: Number(peAmount),
          date: peDate,
          category: peCategory || null,
        },
      });
      peTitle = "";
      peAmount = "";
      peDate = "";
      peCategory = "";
      onrefresh();
    } catch (err) {
      peError = `${err}`;
    }
  }

  async function deletePlanned(id) {
    try {
      await invoke("delete_planned_expense", { id });
      onrefresh();
    } catch (err) {
      console.error("Failed to delete planned expense:", err);
    }
  }
</script>

<div class="space-y-6">
  <!-- Category budgets -->
  <div class="bg-gray-900 rounded-xl p-6 border border-gray-800">
    <h3 class="text-lg font-semibold mb-4">Category Budgets</h3>

    {#if Object.keys(editAmounts).length === 0}
      <p class="text-sm text-gray-500">No categories found. Add expenses to create categories.</p>
    {:else}
      <table class="w-full text-sm mb-4">
        <thead>
          <tr class="border-b border-gray-700 text-gray-400">
            <th class="text-left px-3 py-2">Category</th>
            <th class="text-right px-3 py-2">Avg (3mo)</th>
            <th class="text-right px-3 py-2">Budget</th>
          </tr>
        </thead>
        <tbody>
          {#each Object.keys(editAmounts).sort() as cat}
            {@const avg = getAverage(cat)}
            <tr class="border-b border-gray-800/50">
              <td class="px-3 py-2 text-gray-300">{cat}</td>
              <td class="px-3 py-2 text-right text-gray-500 font-mono text-xs">
                {avg !== null ? avg.toFixed(2) : "—"}
              </td>
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
          onclick={saveBudget}
          disabled={saving}
          class="px-5 py-2 bg-emerald-600 hover:bg-emerald-500 disabled:opacity-50
                 text-white text-sm font-medium rounded-lg transition-colors"
        >
          {saving ? "Saving..." : "Save Budget"}
        </button>
        {#if saveMsg}
          <span class="text-sm {saveMsg.startsWith('Error') ? 'text-red-400' : 'text-emerald-400'}">
            {saveMsg}
          </span>
        {/if}
      </div>
    {/if}
  </div>

  <!-- Planned expenses -->
  <div class="bg-gray-900 rounded-xl p-6 border border-gray-800">
    <h3 class="text-lg font-semibold mb-4">Planned Expenses</h3>

    {#if plannedExpenses.length > 0}
      <table class="w-full text-sm mb-4">
        <thead>
          <tr class="border-b border-gray-700 text-gray-400">
            <th class="text-left px-3 py-2">Date</th>
            <th class="text-left px-3 py-2">Title</th>
            <th class="text-right px-3 py-2">Amount</th>
            <th class="text-left px-3 py-2">Category</th>
            <th class="px-3 py-2"></th>
          </tr>
        </thead>
        <tbody>
          {#each plannedExpenses as pe}
            <tr class="border-b border-gray-800/50">
              <td class="px-3 py-2 text-gray-400">{pe.date}</td>
              <td class="px-3 py-2 text-gray-300">{pe.title}</td>
              <td class="px-3 py-2 text-right font-mono">{pe.amount.toFixed(2)}</td>
              <td class="px-3 py-2 text-gray-400">{pe.category || "—"}</td>
              <td class="px-3 py-2 text-right">
                <button
                  onclick={() => deletePlanned(pe.id)}
                  class="text-gray-500 hover:text-red-400 text-xs transition-colors"
                >
                  Delete
                </button>
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
    {/if}

    <!-- Add planned expense form -->
    <div class="flex flex-wrap gap-2 items-end">
      <div>
        <label class="text-xs text-gray-500 block mb-1">Title</label>
        <input
          type="text"
          bind:value={peTitle}
          placeholder="e.g. Dentist"
          class="bg-gray-800 border border-gray-700 rounded px-2 py-1.5 text-sm
                 text-gray-100 placeholder-gray-600 focus:outline-none focus:border-emerald-500 w-40"
        />
      </div>
      <div>
        <label class="text-xs text-gray-500 block mb-1">Amount</label>
        <input
          type="number"
          step="0.01"
          bind:value={peAmount}
          placeholder="0.00"
          class="bg-gray-800 border border-gray-700 rounded px-2 py-1.5 text-sm
                 text-gray-100 placeholder-gray-600 focus:outline-none focus:border-emerald-500 w-24 font-mono"
        />
      </div>
      <div>
        <label class="text-xs text-gray-500 block mb-1">Date</label>
        <input
          type="date"
          bind:value={peDate}
          class="bg-gray-800 border border-gray-700 rounded px-2 py-1.5 text-sm
                 text-gray-100 focus:outline-none focus:border-emerald-500"
        />
      </div>
      <div>
        <label class="text-xs text-gray-500 block mb-1">Category</label>
        <select
          bind:value={peCategory}
          class="bg-gray-800 border border-gray-700 rounded px-2 py-1.5 text-sm
                 text-gray-100 focus:outline-none focus:border-emerald-500"
        >
          <option value="">None</option>
          {#each categories as cat}
            <option value={cat}>{cat}</option>
          {/each}
        </select>
      </div>
      <button
        onclick={addPlannedExpense}
        class="px-4 py-1.5 bg-gray-800 hover:bg-gray-700 text-gray-300 text-sm
               rounded-lg transition-colors border border-gray-700"
      >
        + Add
      </button>
    </div>

    {#if peError}
      <div class="text-sm text-red-400 mt-2">{peError}</div>
    {/if}
  </div>
</div>
