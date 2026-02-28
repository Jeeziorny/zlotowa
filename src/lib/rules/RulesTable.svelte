<script>
  import { invoke } from "@tauri-apps/api/core";

  let {
    rules,
    categories,
    loading,
    ondelete,
    onsaved,
  } = $props();

  // Inline edit state
  let editingId = $state(null);
  let editPattern = $state("");
  let editCategory = $state("");
  let editError = $state("");
  let saving = $state(false);

  function startEdit(rule) {
    editingId = rule.id;
    editPattern = rule.pattern;
    editCategory = rule.category;
    editError = "";
  }

  function cancelEdit() {
    editingId = null;
    editError = "";
  }

  async function saveEdit() {
    if (!editPattern.trim()) {
      editError = "Pattern cannot be empty";
      return;
    }
    if (!editCategory.trim()) {
      editError = "Category cannot be empty";
      return;
    }
    saving = true;
    editError = "";
    try {
      await invoke("update_rule", {
        id: editingId,
        pattern: editPattern.trim(),
        category: editCategory.trim(),
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
        <th class="text-left px-4 py-3">Pattern</th>
        <th class="text-left px-4 py-3">Category</th>
        <th class="text-right px-4 py-3">Matches</th>
        <th class="px-4 py-3 w-24"></th>
      </tr>
    </thead>
    <tbody>
      {#each rules as rule (rule.id)}
        {#if editingId === rule.id}
          <tr class="border-b border-gray-800/50 bg-gray-800/20">
            <td class="px-4 py-2">
              <input
                type="text"
                bind:value={editPattern}
                class="w-full bg-gray-800 border border-gray-700 rounded px-2 py-1 text-sm
                       font-mono text-gray-200 focus:border-emerald-500 focus:ring-1
                       focus:ring-emerald-500 focus:outline-none"
              />
            </td>
            <td class="px-4 py-2">
              <input
                type="text"
                bind:value={editCategory}
                list="edit-rule-categories"
                class="w-full bg-gray-800 border border-gray-700 rounded px-2 py-1 text-sm
                       text-gray-200 focus:border-emerald-500 focus:ring-1
                       focus:ring-emerald-500 focus:outline-none"
              />
              <datalist id="edit-rule-categories">
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
          <tr class="border-b border-gray-800/50 hover:bg-gray-800/30 group">
            <td class="px-4 py-3 font-mono text-sm text-gray-300">{rule.pattern}</td>
            <td class="px-4 py-3">
              <span class="bg-emerald-900/30 text-emerald-400 px-2 py-0.5 rounded text-sm">
                {rule.category}
              </span>
            </td>
            <td class="px-4 py-3 text-right text-sm text-gray-400">{rule.match_count}</td>
            <td class="px-4 py-3">
              <div class="flex gap-1 opacity-0 group-hover:opacity-100 group-focus-within:opacity-100 transition-opacity">
                <button
                  onclick={() => startEdit(rule)}
                  class="text-gray-400 hover:text-emerald-400 p-1 transition-colors"
                  title="Edit"
                  aria-label="Edit rule"
                >
                  <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                      d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z" />
                  </svg>
                </button>
                <button
                  onclick={() => ondelete(rule)}
                  class="text-gray-400 hover:text-red-400 p-1 transition-colors"
                  title="Delete"
                  aria-label="Delete rule"
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
