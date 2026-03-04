<script>
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import EmptyState from "../EmptyState.svelte";

  let batches = $state([]);
  let confirmingBatchId = $state(null);
  let batchMessage = $state("");
  let batchMessageType = $state("");

  onMount(async () => {
    try {
      batches = await invoke("get_upload_batches");
    } catch (err) {
      console.error("Failed to load upload batches:", err);
    }
  });

  function formatDate(isoStr) {
    try {
      const d = new Date(isoStr);
      return d.toLocaleDateString(undefined, {
        year: "numeric",
        month: "short",
        day: "numeric",
        hour: "2-digit",
        minute: "2-digit",
      });
    } catch {
      return isoStr;
    }
  }

  async function confirmDeleteBatch(batchId) {
    batchMessage = "";
    try {
      const deleted = await invoke("delete_batch", { batchId });
      batches = batches.filter((b) => b.id !== batchId);
      confirmingBatchId = null;
      batchMessage = `Deleted ${deleted} expense${deleted !== 1 ? "s" : ""}.`;
      batchMessageType = "success";
    } catch (err) {
      batchMessage = `Error: ${err}`;
      batchMessageType = "error";
    }
  }
</script>

<!-- Upload History -->
<div class="bg-gray-900 rounded-xl p-6 border border-gray-800">
  <h3 class="text-lg font-semibold mb-1">Upload History</h3>
  <p class="text-sm text-gray-400 mb-4">
    View past bulk uploads. Undo an upload to delete all its expenses.
  </p>

  {#if batches.length === 0}
    <EmptyState title="No bulk uploads yet." variant="widget" />
  {:else}
    <div class="space-y-3">
      {#each batches as batch}
        <div class="flex items-center justify-between bg-gray-800 rounded-lg px-4 py-3 border border-gray-700">
          <div class="min-w-0 flex-1">
            <p class="text-sm font-medium text-gray-200 truncate">
              {batch.filename || "Unknown file"}
            </p>
            <p class="text-xs text-gray-500">
              {formatDate(batch.uploaded_at)} &middot; {batch.expense_count} expense{batch.expense_count !== 1 ? "s" : ""}
            </p>
          </div>
          <div class="ml-3 flex-shrink-0">
            {#if confirmingBatchId === batch.id}
              <div class="flex items-center gap-2">
                <span class="text-xs text-red-400">Delete {batch.expense_count} expenses?</span>
                <button
                  onclick={() => confirmDeleteBatch(batch.id)}
                  class="px-2 py-1 rounded text-xs bg-red-900/50 text-red-400
                         hover:bg-red-800/50 transition-colors"
                >
                  Confirm
                </button>
                <button
                  onclick={() => (confirmingBatchId = null)}
                  class="px-2 py-1 rounded text-xs bg-gray-700 text-gray-400
                         hover:bg-gray-600 transition-colors"
                >
                  Cancel
                </button>
              </div>
            {:else}
              <button
                onclick={() => (confirmingBatchId = batch.id)}
                class="px-3 py-1 rounded text-xs bg-gray-700 text-gray-400
                       hover:bg-red-900/50 hover:text-red-400 transition-colors"
              >
                Undo
              </button>
            {/if}
          </div>
        </div>
      {/each}
    </div>
  {/if}

  {#if batchMessage}
    <div
      class="text-sm px-4 py-2 rounded-lg mt-3 {batchMessageType === 'success'
        ? 'bg-emerald-900/50 text-emerald-400'
        : 'bg-red-900/50 text-red-400'}"
    >
      {batchMessage}
    </div>
  {/if}
</div>
