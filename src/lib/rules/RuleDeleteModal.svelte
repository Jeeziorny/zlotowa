<script>
  import { invoke } from "@tauri-apps/api/core";

  let { rule, ondelete, onclose } = $props();

  let deleting = $state(false);
  let deleteError = $state("");

  async function doDelete() {
    deleting = true;
    deleteError = "";
    try {
      await invoke("delete_rule", { id: rule.id });
      ondelete();
    } catch (err) {
      deleteError = `Delete failed: ${err}`;
    }
    deleting = false;
  }
</script>

<div class="fixed inset-0 bg-black/60 flex items-center justify-center z-50"
     role="presentation"
     onclick={() => { if (!deleting) onclose(); }}
     onkeydown={(e) => { if (e.key === "Escape" && !deleting) onclose(); }}>
  <div class="bg-gray-900 border border-gray-800 rounded-xl p-6 max-w-sm w-full mx-4 shadow-xl"
       role="dialog"
       aria-modal="true"
       aria-labelledby="delete-rule-modal-title"
       tabindex="-1"
       onclick={(e) => e.stopPropagation()}>
    <h3 id="delete-rule-modal-title" class="text-lg font-semibold text-gray-100 mb-2">Delete rule?</h3>
    <p class="text-sm text-gray-400 mb-1">This cannot be undone.</p>
    <div class="text-sm text-gray-300 mb-5 space-y-1">
      <p class="break-all"><span class="text-gray-500">Pattern:</span> <code class="font-mono">{rule.pattern}</code></p>
      <p><span class="text-gray-500">Category:</span> {rule.category}</p>
    </div>
    {#if deleteError}
      <div class="text-sm bg-red-900/50 text-red-400 px-4 py-2 rounded-lg mb-3">{deleteError}</div>
    {/if}
    <div class="flex gap-3 justify-end">
      <button
        onclick={onclose}
        disabled={deleting}
        class="bg-gray-800 hover:bg-gray-700 text-gray-300 px-4 py-2 rounded-lg
               text-sm transition-colors disabled:opacity-50"
      >
        Cancel
      </button>
      <button
        onclick={doDelete}
        disabled={deleting}
        class="bg-red-600 hover:bg-red-500 disabled:opacity-50 text-white px-4 py-2
               rounded-lg text-sm font-medium transition-colors"
      >
        {deleting ? "Deleting..." : "Delete"}
      </button>
    </div>
  </div>
</div>
