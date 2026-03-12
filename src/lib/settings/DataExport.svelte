<script>
  import { invoke } from "@tauri-apps/api/core";
  import { save } from "@tauri-apps/plugin-dialog";
  import { addToast } from "../stores/toast.svelte.js";

  let exporting = $state(false);
  let error = $state("");

  async function doExport() {
    exporting = true;
    error = "";
    try {
      const path = await save({
        defaultPath: `zlotowa-export-${new Date().toISOString().slice(0, 10)}.csv`,
        filters: [{ name: "CSV", extensions: ["csv"] }],
      });
      if (!path) {
        exporting = false;
        return;
      }
      const count = await invoke("export_csv", { path });
      addToast(`Exported ${count} expenses to CSV`, "success");
    } catch (err) {
      error = `Export failed: ${err}`;
    }
    exporting = false;
  }
</script>

<div class="bg-gray-900 rounded-xl p-6 border border-gray-800">
  <h3 class="text-lg font-semibold mb-1">Data Export</h3>
  <p class="text-sm text-gray-400 mb-4">
    Export all expenses as a CSV file with date, title, amount, and category columns.
  </p>

  <button
    onclick={doExport}
    disabled={exporting}
    class="bg-amber-500 hover:bg-amber-400 disabled:opacity-50 text-gray-950
           font-medium py-2.5 px-6 rounded-lg transition-colors"
  >
    {#if exporting}
      <span class="inline-block w-4 h-4 border-2 border-white/30 border-t-white rounded-full animate-spin mr-2 align-middle"></span>
      Exporting...
    {:else}
      Export CSV
    {/if}
  </button>

  {#if error}
    <div class="text-sm px-4 py-2 rounded-lg bg-red-900/50 text-red-400 mt-4">
      {error}
    </div>
  {/if}
</div>
