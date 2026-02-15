<script>
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";

  let expenses = $state([]);

  let showExportModal = $state(false);
  let exportDate = $state(true);
  let exportTitle = $state(true);
  let exportAmount = $state(true);
  let exportCategory = $state(true);
  let exportSource = $state(false);
  let exportError = $state("");
  let exporting = $state(false);

  onMount(async () => {
    try {
      expenses = await invoke("get_expenses");
    } catch (err) {
      console.error("Failed to load expenses:", err);
    }
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

  function downloadBlob(bytes, filename) {
    const blob = new Blob([new Uint8Array(bytes)], { type: "text/csv" });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = filename;
    a.click();
    URL.revokeObjectURL(url);
  }

  async function doExport() {
    exporting = true;
    exportError = "";
    try {
      const bytes = await invoke("export_expenses", {
        columns: {
          date: exportDate,
          title: exportTitle,
          amount: exportAmount,
          category: exportCategory,
          classification_source: exportSource,
        },
      });
      downloadBlob(bytes, `4ccountant-export-${new Date().toISOString().split("T")[0]}.csv`);
      showExportModal = false;
    } catch (err) {
      exportError = `Export failed: ${err}`;
    }
    exporting = false;
  }
</script>

<div>
  <div class="flex items-center justify-between mb-6">
    <h2 class="text-2xl font-bold">Expenses</h2>
    {#if expenses.length > 0}
      <button
        onclick={() => { showExportModal = !showExportModal; exportError = ""; }}
        class="bg-gray-800 hover:bg-gray-700 text-gray-200 px-4 py-2 rounded-lg
               text-sm font-medium transition-colors border border-gray-700"
      >
        Export CSV
      </button>
    {/if}
  </div>

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
            <th class="text-left px-6 py-3">Date</th>
            <th class="text-left px-6 py-3">Title</th>
            <th class="text-right px-6 py-3">Amount</th>
            <th class="text-left px-6 py-3">Category</th>
            <th class="text-left px-6 py-3">Source</th>
          </tr>
        </thead>
        <tbody>
          {#each expenses as expense}
            <tr class="border-b border-gray-800/50 hover:bg-gray-800/30">
              <td class="px-6 py-3 text-sm text-gray-400">{expense.date}</td>
              <td class="px-6 py-3">{expense.title}</td>
              <td class="px-6 py-3 text-right font-mono">{expense.amount.toFixed(2)}</td>
              <td class="px-6 py-3">
                {#if expense.category}
                  <span class="bg-emerald-900/30 text-emerald-400 px-2 py-0.5 rounded text-sm">
                    {expense.category}
                  </span>
                {:else}
                  <span class="text-gray-600 text-sm">-</span>
                {/if}
              </td>
              <td class="px-6 py-3">
                {#if expense.classification_source}
                  <span class="px-2 py-0.5 rounded text-xs {sourceBadgeClass(expense.classification_source)}">
                    {sourceLabel(expense.classification_source)}
                  </span>
                {/if}
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
  {/if}
</div>
