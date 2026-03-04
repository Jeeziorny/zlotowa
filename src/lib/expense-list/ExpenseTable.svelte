<script>
  import { invoke } from "@tauri-apps/api/core";
  import Autocomplete from "../Autocomplete.svelte";

  let {
    expenses,
    categories,
    loading,
    selected,
    allSelected,
    onselect,
    onselectall,
    ondelete,
    onsaved,
  } = $props();

  // Inline edit state
  let editingId = $state(null);
  let editTitle = $state("");
  let editAmount = $state("");
  let editDate = $state("");
  let editCategory = $state("");
  let editError = $state("");
  let saving = $state(false);

  function startEdit(expense) {
    editingId = expense.id;
    editTitle = expense.title;
    editAmount = String(expense.amount);
    editDate = expense.date;
    editCategory = expense.category || "";
    editError = "";
  }

  function cancelEdit() {
    editingId = null;
    editError = "";
  }

  async function saveEdit() {
    const amount = parseFloat(editAmount);
    if (isNaN(amount) || amount <= 0) {
      editError = "Amount must be greater than zero";
      return;
    }
    if (!editTitle.trim()) {
      editError = "Title cannot be empty";
      return;
    }
    if (!editDate) {
      editError = "Date is required";
      return;
    }
    saving = true;
    editError = "";
    try {
      await invoke("update_expense", {
        id: editingId,
        input: {
          title: editTitle.trim(),
          amount,
          date: editDate,
          category: editCategory.trim() || null,
          rule_pattern: null,
        },
      });
      editingId = null;
      onsaved();
    } catch (err) {
      editError = `Save failed: ${err}`;
    }
    saving = false;
  }
</script>

<div class="bg-gray-900 rounded-xl border border-gray-800 overflow-hidden {loading ? 'opacity-60' : ''}">
  <table class="w-full">
    <thead>
      <tr class="border-b border-gray-800 text-sm text-gray-400">
        <th class="px-4 py-3 w-10">
          <input
            type="checkbox"
            checked={allSelected}
            onchange={onselectall}
            aria-label="Select all expenses"
            class="rounded bg-gray-800 border-gray-700 text-amber-500 focus:ring-amber-500"
          />
        </th>
        <th class="text-left px-4 py-3">Date</th>
        <th class="text-left px-4 py-3">Title</th>
        <th class="text-right px-4 py-3">Amount</th>
        <th class="text-left px-4 py-3">Category</th>
        <th class="px-4 py-3 w-24"></th>
      </tr>
    </thead>
    <tbody>
      {#each expenses as expense (expense.id)}
        {#if editingId === expense.id}
          <!-- Edit mode row -->
          <tr class="border-b border-gray-800/50 bg-gray-800/20">
            <td class="px-4 py-2"></td>
            <td class="px-4 py-2">
              <input
                type="date"
                bind:value={editDate}
                class="w-full bg-gray-800 border border-gray-700 rounded px-2 py-1 text-sm
                       text-gray-200 focus:border-amber-500 focus:ring-1 focus:ring-amber-500 focus:outline-none"
              />
            </td>
            <td class="px-4 py-2">
              <input
                type="text"
                bind:value={editTitle}
                class="w-full bg-gray-800 border border-gray-700 rounded px-2 py-1 text-sm
                       text-gray-200 focus:border-amber-500 focus:ring-1 focus:ring-amber-500 focus:outline-none"
              />
            </td>
            <td class="px-4 py-2">
              <input
                type="number"
                step="0.01"
                min="0.01"
                bind:value={editAmount}
                class="w-full bg-gray-800 border border-gray-700 rounded px-2 py-1 text-sm text-right
                       font-mono text-gray-200 focus:border-amber-500 focus:ring-1 focus:ring-amber-500 focus:outline-none"
              />
            </td>
            <td class="px-4 py-2">
              <Autocomplete
                bind:value={editCategory}
                options={categories}
                class="w-full"
                inputClass="w-full bg-gray-800 border border-gray-700 rounded px-2 py-1 text-sm
                            text-gray-200 focus:border-amber-500 focus:ring-1 focus:ring-amber-500 focus:outline-none"
              />
            </td>
            <td class="px-4 py-2">
              <div class="flex gap-1">
                <button
                  onclick={saveEdit}
                  disabled={saving}
                  class="text-amber-400 hover:text-amber-300 disabled:opacity-50 p-1 transition-colors"
                  title="Save"
                  aria-label="Save edit"
                >
                  <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
                  </svg>
                </button>
                <button
                  onclick={cancelEdit}
                  class="text-gray-400 hover:text-gray-300 p-1 transition-colors"
                  title="Cancel"
                  aria-label="Cancel edit"
                >
                  <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
                  </svg>
                </button>
              </div>
              {#if editError}
                <div class="text-red-400 text-xs mt-1">{editError}</div>
              {/if}
            </td>
          </tr>
        {:else}
          <!-- Normal row -->
          <tr class="border-b border-gray-800/50 hover:bg-gray-800/30 group border-l-2 border-l-transparent group-hover:border-l-amber-500/40 transition-colors">
            <td class="px-4 py-3">
              <input
                type="checkbox"
                checked={selected.has(expense.id)}
                onchange={() => onselect(expense.id)}
                class="rounded bg-gray-800 border-gray-700 text-amber-500 focus:ring-amber-500"
              />
            </td>
            <td class="px-4 py-3 text-sm text-gray-400">{expense.date}</td>
            <td class="px-4 py-3">{expense.title}</td>
            <td class="px-4 py-3 text-right font-mono">{expense.amount.toFixed(2)}</td>
            <td class="px-4 py-3">
              {#if expense.category}
                <span class="bg-amber-900/30 text-amber-400 px-2 py-0.5 rounded text-sm">
                  {expense.category}
                </span>
              {:else}
                <span class="text-gray-600 text-sm">-</span>
              {/if}
            </td>
            <td class="px-4 py-3">
              <div class="flex gap-1 opacity-0 group-hover:opacity-100 group-focus-within:opacity-100 transition-opacity">
                <button
                  onclick={() => startEdit(expense)}
                  class="text-gray-500 hover:text-gray-300 p-1 transition-colors"
                  title="Edit"
                  aria-label="Edit expense"
                >
                  <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                      d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z" />
                  </svg>
                </button>
                <button
                  onclick={() => ondelete(expense)}
                  class="text-gray-500 hover:text-red-400 p-1 transition-colors"
                  title="Delete"
                  aria-label="Delete expense"
                >
                  <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                      d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
                  </svg>
                </button>
              </div>
            </td>
          </tr>
        {/if}
      {/each}
    </tbody>
  </table>
</div>
