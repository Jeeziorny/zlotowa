<script>
  import { invoke } from "@tauri-apps/api/core";
  import { save } from "@tauri-apps/plugin-dialog";

  let { show = $bindable(), onclose } = $props();

  let exportDate = $state(true);
  let exportTitle = $state(true);
  let exportAmount = $state(true);
  let exportCategory = $state(true);
  let exportSource = $state(false);
  let exportDisplayTitle = $state(false);
  let exportError = $state("");
  let exportSuccess = $state("");
  let exporting = $state(false);

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
          display_title: exportDisplayTitle,
          amount: exportAmount,
          category: exportCategory,
          classification_source: exportSource,
        },
        path,
      });
      const filename = path.split("/").pop() || path.split("\\").pop() || path;
      exportSuccess = `Exported to ${filename}`;
      setTimeout(() => { show = false; exportSuccess = ""; }, 2000);
    } catch (err) {
      exportError = `Export failed: ${err}`;
    }
    exporting = false;
  }
</script>

{#if show}
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
        Title (raw)
      </label>
      <label class="flex items-center gap-2 text-sm text-gray-300 cursor-pointer">
        <input type="checkbox" bind:checked={exportDisplayTitle}
               class="rounded bg-gray-800 border-gray-700 text-emerald-500 focus:ring-emerald-500" />
        Display Title
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
        onclick={onclose}
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
