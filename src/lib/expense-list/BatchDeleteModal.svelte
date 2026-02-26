<script>
  import { invoke } from "@tauri-apps/api/core";

  let { selectedIds, ondelete, onclose } = $props();

  let batchDeleting = $state(false);
  let batchDeleteError = $state("");

  async function doBatchDelete() {
    batchDeleting = true;
    batchDeleteError = "";
    try {
      const ids = [...selectedIds];
      await invoke("delete_expenses", { ids });
      ondelete();
    } catch (err) {
      batchDeleteError = `Delete failed: ${err}`;
    }
    batchDeleting = false;
  }
</script>

<div class="fixed inset-0 bg-black/60 flex items-center justify-center z-50"
     role="presentation"
     onclick={() => { if (!batchDeleting) onclose(); }}
     onkeydown={(e) => { if (e.key === "Escape" && !batchDeleting) onclose(); }}>
  <div class="bg-gray-900 border border-gray-800 rounded-xl p-6 max-w-sm w-full mx-4 shadow-xl"
       role="dialog"
       aria-modal="true"
       aria-labelledby="batch-delete-modal-title"
       tabindex="-1"
       onclick={(e) => e.stopPropagation()}>
    <h3 id="batch-delete-modal-title" class="text-lg font-semibold text-gray-100 mb-2">Delete {selectedIds.size} expense{selectedIds.size > 1 ? "s" : ""}?</h3>
    <p class="text-sm text-gray-400 mb-5">This cannot be undone.</p>
    {#if batchDeleteError}
      <div class="text-sm bg-red-900/50 text-red-400 px-4 py-2 rounded-lg mb-3">{batchDeleteError}</div>
    {/if}
    <div class="flex gap-3 justify-end">
      <button
        onclick={onclose}
        disabled={batchDeleting}
        class="bg-gray-800 hover:bg-gray-700 text-gray-300 px-4 py-2 rounded-lg
               text-sm transition-colors disabled:opacity-50"
      >
        Cancel
      </button>
      <button
        onclick={doBatchDelete}
        disabled={batchDeleting}
        class="bg-red-600 hover:bg-red-500 disabled:opacity-50 text-white px-4 py-2
               rounded-lg text-sm font-medium transition-colors"
      >
        {batchDeleting ? "Deleting..." : "Delete"}
      </button>
    </div>
  </div>
</div>
