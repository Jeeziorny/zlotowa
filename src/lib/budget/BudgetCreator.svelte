<script>
  import { invoke } from "@tauri-apps/api/core";
  import DatePicker from "../DatePicker.svelte";

  let { allCategories, averages, oncreated } = $props();

  let step = $state(1);

  // Step 1: Date range
  let startDate = $state("");
  let endDate = $state("");
  let dateError = $state("");
  let checking = $state(false);

  // Step 2: Category budgets
  let categoryBudgets = $state([]);
  let showAddCategories = $state(false);
  let categoryError = $state("");

  // Step 3: Creating
  let creating = $state(false);
  let createError = $state("");

  function getAverage(category) {
    const avg = averages.find((a) => a.category === category);
    return avg ? avg.average : 0;
  }

  async function validateDates() {
    dateError = "";
    if (!startDate || !endDate) {
      dateError = "Both start and end dates are required.";
      return;
    }
    if (startDate >= endDate) {
      dateError = "Start date must be before end date.";
      return;
    }
    checking = true;
    try {
      const overlaps = await invoke("check_budget_overlap", {
        startDate,
        endDate,
      });
      if (overlaps) {
        dateError = "This date range overlaps with an existing budget.";
        checking = false;
        return;
      }
      // Populate categories from averages
      categoryBudgets = averages.map((a) => ({
        category: a.category,
        average: a.average,
        amount: Math.round(a.average * 100) / 100,
      }));
      step = 2;
    } catch (err) {
      dateError = `${err}`;
    }
    checking = false;
  }

  function addCategory(cat) {
    if (categoryBudgets.find((c) => c.category === cat)) return;
    categoryBudgets = [
      ...categoryBudgets,
      { category: cat, average: 0, amount: 0 },
    ];
  }

  let remainingCategories = $derived(
    allCategories.filter(
      (cat) => !categoryBudgets.find((cb) => cb.category === cat),
    ),
  );

  function validateCategories() {
    categoryError = "";
    const invalid = categoryBudgets.filter(
      (c) => !c.amount || Number(c.amount) <= 0,
    );
    if (invalid.length > 0) {
      const names = invalid.map((c) => c.category).join(", ");
      categoryError = `Every category needs an amount > 0. Fix: ${names}`;
      return false;
    }
    return true;
  }

  async function createBudget() {
    creating = true;
    createError = "";
    try {
      const cats = categoryBudgets
        .filter((c) => c.amount > 0)
        .map((c) => ({ category: c.category, amount: Number(c.amount) }));
      await invoke("create_budget", {
        startDate,
        endDate,
        categories: cats,
      });
      oncreated();
    } catch (err) {
      createError = `${err}`;
    }
    creating = false;
  }
</script>

<div class="space-y-6">
  <!-- Step indicator -->
  <div class="flex items-center gap-2 mb-2">
    {#each [1, 2, 3] as s}
      <div
        class="flex items-center gap-2 {s <= step ? 'text-emerald-400' : 'text-gray-600'}"
      >
        <div
          class="w-7 h-7 rounded-full flex items-center justify-center text-xs font-bold border
          {s === step
            ? 'border-emerald-400 bg-emerald-400/10'
            : s < step
              ? 'border-emerald-600 bg-emerald-600/20'
              : 'border-gray-700'}"
        >
          {s}
        </div>
        <span class="text-sm {s === step ? 'font-medium' : ''}">
          {s === 1 ? "Dates" : s === 2 ? "Categories" : "Review"}
        </span>
      </div>
      {#if s < 3}
        <div class="flex-1 h-px {s < step ? 'bg-emerald-600' : 'bg-gray-800'}">
        </div>
      {/if}
    {/each}
  </div>

  {#if step === 1}
    <div class="bg-gray-900 rounded-xl p-6 border border-gray-800">
      <h3 class="text-lg font-semibold mb-4">Set Budget Period</h3>
      <div class="flex gap-4 items-end">
        <div>
          <label for="budget-start-date" class="text-xs text-gray-500 block mb-1">Start Date</label>
          <div class="w-48">
            <DatePicker
              value={startDate}
              onchange={(d) => (startDate = d)}
              id="budget-start-date"
            />
          </div>
        </div>
        <div>
          <label for="budget-end-date" class="text-xs text-gray-500 block mb-1">End Date</label>
          <div class="w-48">
            <DatePicker
              value={endDate}
              onchange={(d) => (endDate = d)}
              id="budget-end-date"
            />
          </div>
        </div>
        <button
          onclick={validateDates}
          disabled={checking}
          class="px-5 py-2 bg-emerald-600 hover:bg-emerald-500 disabled:opacity-50
                 text-white text-sm font-medium rounded-lg transition-colors"
        >
          {checking ? "Checking..." : "Next"}
        </button>
      </div>
      {#if dateError}
        <div class="text-sm text-red-400 mt-3">{dateError}</div>
      {/if}
    </div>
  {:else if step === 2}
    <div class="bg-gray-900 rounded-xl p-6 border border-gray-800">
      <h3 class="text-lg font-semibold mb-4">Category Budgets</h3>
      <p class="text-sm text-gray-400 mb-4">
        Budget amounts are pre-filled from 3-month averages. Adjust as needed.
      </p>

      {#if categoryBudgets.length > 0}
        <table class="w-full text-sm mb-4">
          <thead>
            <tr class="border-b border-gray-700 text-gray-400">
              <th class="text-left px-3 py-2">Category</th>
              <th class="text-right px-3 py-2">Avg (3mo)</th>
              <th class="text-right px-3 py-2">Budget</th>
            </tr>
          </thead>
          <tbody>
            {#each categoryBudgets as cb, i}
              <tr class="border-b border-gray-800/50">
                <td class="px-3 py-2 text-gray-300">{cb.category}</td>
                <td
                  class="px-3 py-2 text-right text-gray-500 font-mono text-xs"
                >
                  {cb.average > 0 ? cb.average.toFixed(2) : "—"}
                </td>
                <td class="px-3 py-2 text-right">
                  <input
                    type="number"
                    step="0.01"
                    min="0"
                    bind:value={categoryBudgets[i].amount}
                    class="w-28 bg-gray-800 border border-gray-700 rounded px-2 py-1
                           text-right text-gray-100 font-mono focus:outline-none
                           focus:border-emerald-500"
                  />
                </td>
              </tr>
            {/each}
          </tbody>
        </table>
      {:else}
        <p class="text-sm text-gray-500 mb-4">
          No categories with recent data. Add categories below.
        </p>
      {/if}

      {#if remainingCategories.length > 0}
        <div class="mb-4">
          <button
            onclick={() => (showAddCategories = !showAddCategories)}
            class="text-sm text-emerald-400 hover:text-emerald-300 transition-colors"
          >
            {showAddCategories ? "Hide" : "+ Add more categories"}
          </button>
          {#if showAddCategories}
            <div class="mt-2 flex flex-wrap gap-2">
              {#each remainingCategories as cat}
                <button
                  onclick={() => addCategory(cat)}
                  class="px-3 py-1 text-xs bg-gray-800 hover:bg-gray-700 text-gray-300
                         rounded-lg border border-gray-700 transition-colors"
                >
                  + {cat}
                </button>
              {/each}
            </div>
          {/if}
        </div>
      {/if}

      <div class="flex gap-3">
        <button
          onclick={() => (step = 1)}
          class="px-4 py-2 text-sm text-gray-400 hover:text-gray-200 transition-colors"
        >
          Back
        </button>
        <button
          onclick={() => {
            if (validateCategories()) step = 3;
          }}
          class="px-5 py-2 bg-emerald-600 hover:bg-emerald-500
                 text-white text-sm font-medium rounded-lg transition-colors"
        >
          Next
        </button>
      </div>

      {#if categoryError}
        <div class="text-sm text-red-400 mt-3">{categoryError}</div>
      {/if}
    </div>
  {:else if step === 3}
    <div class="bg-gray-900 rounded-xl p-6 border border-gray-800">
      <h3 class="text-lg font-semibold mb-4">Review Budget</h3>

      <div class="space-y-3 mb-6">
        <div class="flex justify-between text-sm">
          <span class="text-gray-400">Period</span>
          <span class="text-gray-200 font-mono">{startDate} — {endDate}</span>
        </div>
        <div class="flex justify-between text-sm">
          <span class="text-gray-400">Categories</span>
          <span class="text-gray-200"
            >{categoryBudgets.filter((c) => c.amount > 0).length}</span
          >
        </div>
        <div class="flex justify-between text-sm">
          <span class="text-gray-400">Total budgeted</span>
          <span class="text-gray-200 font-mono font-bold">
            {categoryBudgets
              .reduce((sum, c) => sum + Number(c.amount), 0)
              .toFixed(2)}
          </span>
        </div>
      </div>

      {#if categoryBudgets.filter((c) => c.amount > 0).length > 0}
        <div class="mb-6 space-y-1">
          {#each categoryBudgets.filter((c) => c.amount > 0) as cb}
            <div class="flex justify-between text-sm">
              <span class="text-gray-400">{cb.category}</span>
              <span class="text-gray-300 font-mono"
                >{Number(cb.amount).toFixed(2)}</span
              >
            </div>
          {/each}
        </div>
      {/if}

      {#if createError}
        <div class="text-sm text-red-400 mb-4">{createError}</div>
      {/if}

      <div class="flex gap-3">
        <button
          onclick={() => (step = 2)}
          class="px-4 py-2 text-sm text-gray-400 hover:text-gray-200 transition-colors"
        >
          Back
        </button>
        <button
          onclick={createBudget}
          disabled={creating}
          class="px-5 py-2 bg-emerald-600 hover:bg-emerald-500 disabled:opacity-50
                 text-white text-sm font-medium rounded-lg transition-colors"
        >
          {creating ? "Creating..." : "Create Budget"}
        </button>
      </div>
    </div>
  {/if}
</div>
