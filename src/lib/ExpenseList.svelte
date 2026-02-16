<script>
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { save } from "@tauri-apps/plugin-dialog";

  let expenses = $state([]);

  // Export
  let showExportModal = $state(false);
  let exportDate = $state(true);
  let exportTitle = $state(true);
  let exportAmount = $state(true);
  let exportCategory = $state(true);
  let exportSource = $state(false);
  let exportError = $state("");
  let exportSuccess = $state("");
  let exporting = $state(false);

  // Inline edit
  let editingId = $state(null);
  let editTitle = $state("");
  let editAmount = $state("");
  let editDate = $state("");
  let editCategory = $state("");
  let editError = $state("");
  let saving = $state(false);

  // Delete
  let confirmDeleteId = $state(null);
  let deleting = $state(false);

  // Batch select/delete
  let selected = $state(new Set());
  let confirmBatchDelete = $state(false);
  let batchDeleting = $state(false);

  let allSelected = $derived(expenses.length > 0 && selected.size === expenses.length);
  let someSelected = $derived(selected.size > 0);

  // Categories for edit dropdown
  let categories = $state([]);

  onMount(async () => {
    try {
      expenses = await invoke("get_expenses");
    } catch (err) {
      console.error("Failed to load expenses:", err);
    }
    try {
      categories = await invoke("get_categories");
    } catch (_) {}
  });

  function sourceLabel(source) {
    if (!source) return "";
    switch (source) {
      case "Database": return "DB";
      case "Llm": return "LLM";
      case "Manual": return "Manual";
      default: return source;
    }
  }

  function sourceBadgeClass(source) {
    switch (source) {
      case "Database": return "bg-blue-900/50 text-blue-400";
      case "Llm": return "bg-purple-900/50 text-purple-400";
      case "Manual": return "bg-gray-800 text-gray-400";
      default: return "bg-gray-800 text-gray-400";
    }
  }

  // ── Export ──

  async function doExport() {
    exporting = true;
    exportError = "";
    exportSuccess = "";
    try {
      const path = await save({
        defaultPath: `4ccountant-export-${new Date().toISOString().split("T")[0]}.csv`,
        filters: [{ name: "CSV", extensions: ["csv"] }],
      });
      if (!path) { exporting = false; return; }
      await invoke("export_expenses", {
        columns: {
          date: exportDate,
          title: exportTitle,
          amount: exportAmount,
          category: exportCategory,
          classification_source: exportSource,
        },
        path,
      });
      const filename = path.split("/").pop() || path.split("\\").pop() || path;
      exportSuccess = `Exported to ${filename}`;
      setTimeout(() => { showExportModal = false; exportSuccess = ""; }, 2000);
    } catch (err) {
      exportError = `Export failed: ${err}`;
    }
    exporting = false;
  }

  // ── Inline Edit ──

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
    if (isNaN(amount)) {
      editError = "Amount must be a valid number";
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
      // Update local state
      expenses = expenses.map(e =>
        e.id === editingId
          ? { ...e, title: editTitle.trim(), amount, date: editDate, category: editCategory.trim() || null, classification_source: "Manual" }
          : e
      );
      editingId = null;
    } catch (err) {
      editError = `Save failed: ${err}`;
    }
    saving = false;
  }

  // ── Single Delete ──

  async function doDelete(id) {
    deleting = true;
    try {
      await invoke("delete_expense", { id });
      expenses = expenses.filter(e => e.id !== id);
      selected.delete(id);
      selected = new Set(selected);
      confirmDeleteId = null;
    } catch (err) {
      console.error("Delete failed:", err);
    }
    deleting = false;
  }

  // ── Batch Delete ──

  function toggleSelect(id) {
    if (selected.has(id)) {
      selected.delete(id);
    } else {
      selected.add(id);
    }
    selected = new Set(selected);
  }

  function toggleSelectAll() {
    if (allSelected) {
      selected = new Set();
    } else {
      selected = new Set(expenses.map(e => e.id));
    }
  }

  async function doBatchDelete() {
    batchDeleting = true;
    try {
      const ids = [...selected];
      await invoke("delete_expenses", { ids });
      expenses = expenses.filter(e => !selected.has(e.id));
      selected = new Set();
      confirmBatchDelete = false;
    } catch (err) {
      console.error("Batch delete failed:", err);
    }
    batchDeleting = false;
  }
</script>

<div>
  <div class="flex items-center justify-between mb-6">
    <h2 class="text-2xl font-bold">Expenses</h2>
    <div class="flex gap-2">
      {#if someSelected}
        <button
          onclick={() => confirmBatchDelete = true}
          class="bg-red-900/50 hover:bg-red-900/80 text-red-400 px-4 py-2 rounded-lg
                 text-sm font-medium transition-colors border border-red-800"
        >
          Delete {selected.size} selected
        </button>
      {/if}
      {#if expenses.length > 0}
        <button
          onclick={() => { showExportModal = !showExportModal; exportError = ""; exportSuccess = ""; }}
          class="bg-gray-800 hover:bg-gray-700 text-gray-200 px-4 py-2 rounded-lg
                 text-sm font-medium transition-colors border border-gray-700"
        >
          Export CSV
        </button>
      {/if}
    </div>
  </div>

  <!-- Batch delete confirmation -->
  {#if confirmBatchDelete}
    <div class="bg-red-900/20 border border-red-800 rounded-xl p-4 mb-4 flex items-center justify-between">
      <span class="text-red-400 text-sm">
        Delete {selected.size} expense{selected.size > 1 ? "s" : ""}? This cannot be undone.
      </span>
      <div class="flex gap-2">
        <button
          onclick={doBatchDelete}
          disabled={batchDeleting}
          class="bg-red-600 hover:bg-red-500 disabled:opacity-50 text-white px-3 py-1.5
                 rounded-lg text-sm font-medium transition-colors"
        >
          {batchDeleting ? "Deleting..." : "Confirm"}
        </button>
        <button
          onclick={() => confirmBatchDelete = false}
          class="bg-gray-800 hover:bg-gray-700 text-gray-300 px-3 py-1.5 rounded-lg
                 text-sm transition-colors"
        >
          Cancel
        </button>
      </div>
    </div>
  {/if}

  {#if showExportModal}
    <div class="bg-gray-900 rounded-xl p-6 border border-gray-800 mb-6">
      <h3 class="text-lg font-semibold mb-3">Export Settings</h3>
      <div class="space-y-2 mb-4">
        <label class="flex items-center gap-2 text-sm text-gray-300 cursor-pointer">
          <input type="checkbox" bind:checked={exportDate}
                 class="rounded bg-gray-800 border-gray-700 text-emerald-500 focus:ring-emerald-500" />
          Date
        </label>
        <label class="flex items-center gap-2 text-sm text-gray-300 cursor-pointer">
          <input type="checkbox" bind:checked={exportTitle}
                 class="rounded bg-gray-800 border-gray-700 text-emerald-500 focus:ring-emerald-500" />
          Title
        </label>
        <label class="flex items-center gap-2 text-sm text-gray-300 cursor-pointer">
          <input type="checkbox" bind:checked={exportAmount}
                 class="rounded bg-gray-800 border-gray-700 text-emerald-500 focus:ring-emerald-500" />
          Amount
        </label>
        <label class="flex items-center gap-2 text-sm text-gray-300 cursor-pointer">
          <input type="checkbox" bind:checked={exportCategory}
                 class="rounded bg-gray-800 border-gray-700 text-emerald-500 focus:ring-emerald-500" />
          Category
        </label>
        <label class="flex items-center gap-2 text-sm text-gray-300 cursor-pointer">
          <input type="checkbox" bind:checked={exportSource}
                 class="rounded bg-gray-800 border-gray-700 text-emerald-500 focus:ring-emerald-500" />
          Classification Source
        </label>
      </div>
      <div class="flex gap-3">
        <button
          onclick={doExport}
          disabled={exporting}
          class="bg-emerald-600 hover:bg-emerald-500 disabled:opacity-50 text-white
                 px-4 py-2 rounded-lg text-sm font-medium transition-colors"
        >
          {exporting ? "Exporting..." : "Download CSV"}
        </button>
        <button
          onclick={() => showExportModal = false}
          class="bg-gray-800 hover:bg-gray-700 text-gray-300 px-4 py-2 rounded-lg
                 text-sm transition-colors"
        >
          Cancel
        </button>
      </div>
      {#if exportError}
        <div class="mt-3 text-sm bg-red-900/50 text-red-400 px-4 py-2 rounded-lg">
          {exportError}
        </div>
      {/if}
      {#if exportSuccess}
        <div class="mt-3 text-sm bg-emerald-900/50 text-emerald-400 px-4 py-2 rounded-lg">
          {exportSuccess}
        </div>
      {/if}
    </div>
  {/if}

  {#if expenses.length === 0}
    <div class="bg-gray-900 rounded-xl p-12 border border-gray-800 text-center text-gray-500">
      <p class="text-lg mb-2">No expenses yet</p>
      <p class="text-sm">Add an expense or do a bulk upload to get started.</p>
    </div>
  {:else}
    <div class="bg-gray-900 rounded-xl border border-gray-800 overflow-hidden">
      <table class="w-full">
        <thead>
          <tr class="border-b border-gray-800 text-sm text-gray-400">
            <th class="px-4 py-3 w-10">
              <input
                type="checkbox"
                checked={allSelected}
                onchange={toggleSelectAll}
                class="rounded bg-gray-800 border-gray-700 text-emerald-500 focus:ring-emerald-500"
              />
            </th>
            <th class="text-left px-4 py-3">Date</th>
            <th class="text-left px-4 py-3">Title</th>
            <th class="text-right px-4 py-3">Amount</th>
            <th class="text-left px-4 py-3">Category</th>
            <th class="text-left px-4 py-3">Source</th>
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
                           text-gray-200 focus:border-emerald-500 focus:ring-1 focus:ring-emerald-500 focus:outline-none"
                  />
                </td>
                <td class="px-4 py-2">
                  <input
                    type="text"
                    bind:value={editTitle}
                    class="w-full bg-gray-800 border border-gray-700 rounded px-2 py-1 text-sm
                           text-gray-200 focus:border-emerald-500 focus:ring-1 focus:ring-emerald-500 focus:outline-none"
                  />
                </td>
                <td class="px-4 py-2">
                  <input
                    type="number"
                    step="0.01"
                    bind:value={editAmount}
                    class="w-full bg-gray-800 border border-gray-700 rounded px-2 py-1 text-sm text-right
                           font-mono text-gray-200 focus:border-emerald-500 focus:ring-1 focus:ring-emerald-500 focus:outline-none"
                  />
                </td>
                <td class="px-4 py-2">
                  <input
                    type="text"
                    bind:value={editCategory}
                    list="edit-categories"
                    class="w-full bg-gray-800 border border-gray-700 rounded px-2 py-1 text-sm
                           text-gray-200 focus:border-emerald-500 focus:ring-1 focus:ring-emerald-500 focus:outline-none"
                  />
                  <datalist id="edit-categories">
                    {#each categories as cat}
                      <option value={cat}></option>
                    {/each}
                  </datalist>
                </td>
                <td class="px-4 py-2"></td>
                <td class="px-4 py-2">
                  <div class="flex gap-1">
                    <button
                      onclick={saveEdit}
                      disabled={saving}
                      class="text-emerald-400 hover:text-emerald-300 disabled:opacity-50 p-1 transition-colors"
                      title="Save"
                    >
                      <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
                      </svg>
                    </button>
                    <button
                      onclick={cancelEdit}
                      class="text-gray-400 hover:text-gray-300 p-1 transition-colors"
                      title="Cancel"
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
              <tr class="border-b border-gray-800/50 hover:bg-gray-800/30 group">
                <td class="px-4 py-3">
                  <input
                    type="checkbox"
                    checked={selected.has(expense.id)}
                    onchange={() => toggleSelect(expense.id)}
                    class="rounded bg-gray-800 border-gray-700 text-emerald-500 focus:ring-emerald-500"
                  />
                </td>
                <td class="px-4 py-3 text-sm text-gray-400">{expense.date}</td>
                <td class="px-4 py-3">{expense.title}</td>
                <td class="px-4 py-3 text-right font-mono">{expense.amount.toFixed(2)}</td>
                <td class="px-4 py-3">
                  {#if expense.category}
                    <span class="bg-emerald-900/30 text-emerald-400 px-2 py-0.5 rounded text-sm">
                      {expense.category}
                    </span>
                  {:else}
                    <span class="text-gray-600 text-sm">-</span>
                  {/if}
                </td>
                <td class="px-4 py-3">
                  {#if expense.classification_source}
                    <span class="px-2 py-0.5 rounded text-xs {sourceBadgeClass(expense.classification_source)}">
                      {sourceLabel(expense.classification_source)}
                    </span>
                  {/if}
                </td>
                <td class="px-4 py-3">
                  {#if confirmDeleteId === expense.id}
                    <div class="flex gap-1">
                      <button
                        onclick={() => doDelete(expense.id)}
                        disabled={deleting}
                        class="text-red-400 hover:text-red-300 disabled:opacity-50 p-1 transition-colors"
                        title="Confirm delete"
                      >
                        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
                        </svg>
                      </button>
                      <button
                        onclick={() => confirmDeleteId = null}
                        class="text-gray-400 hover:text-gray-300 p-1 transition-colors"
                        title="Cancel"
                      >
                        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
                        </svg>
                      </button>
                    </div>
                  {:else}
                    <div class="flex gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
                      <button
                        onclick={() => startEdit(expense)}
                        class="text-gray-400 hover:text-emerald-400 p-1 transition-colors"
                        title="Edit"
                      >
                        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                            d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z" />
                        </svg>
                      </button>
                      <button
                        onclick={() => confirmDeleteId = expense.id}
                        class="text-gray-400 hover:text-red-400 p-1 transition-colors"
                        title="Delete"
                      >
                        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                            d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
                        </svg>
                      </button>
                    </div>
                  {/if}
                </td>
              </tr>
            {/if}
          {/each}
        </tbody>
      </table>
    </div>
  {/if}
</div>
