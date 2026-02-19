<script>
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";

  let rules = $state([]);
  let error = $state("");
  let success = $state("");

  // Add/edit form
  let showForm = $state(false);
  let editingRule = $state(null);
  let formPattern = $state("");
  let formReplacement = $state("");
  let formIsRegex = $state(false);
  let formError = $state("");

  // Preview
  let previewRule = $state(null);
  let previews = $state([]);
  let previewLoading = $state(false);
  let selectedIds = $state(new Set());
  let applyLoading = $state(false);

  // Delete confirm
  let deleteTarget = $state(null);

  onMount(loadRules);

  async function loadRules() {
    try {
      rules = await invoke("get_title_cleanup_rules");
      error = "";
    } catch (err) {
      error = `Failed to load rules: ${err}`;
    }
  }

  function openAdd() {
    editingRule = null;
    formPattern = "";
    formReplacement = "";
    formIsRegex = false;
    formError = "";
    showForm = true;
  }

  function openEdit(rule) {
    editingRule = rule;
    formPattern = rule.pattern;
    formReplacement = rule.replacement;
    formIsRegex = rule.is_regex;
    formError = "";
    showForm = true;
  }

  async function saveForm() {
    formError = "";
    if (!formPattern.trim()) {
      formError = "Pattern cannot be empty";
      return;
    }
    try {
      const rule = {
        id: editingRule?.id ?? null,
        pattern: formPattern,
        replacement: formReplacement,
        is_regex: formIsRegex,
      };
      await invoke("save_title_cleanup_rule", { rule });
      showForm = false;
      await loadRules();
    } catch (err) {
      formError = `${err}`;
    }
  }

  async function confirmDelete() {
    try {
      await invoke("delete_title_cleanup_rule", { id: deleteTarget.id });
      deleteTarget = null;
      if (previewRule?.id === deleteTarget?.id) {
        previewRule = null;
        previews = [];
      }
      await loadRules();
    } catch (err) {
      error = `Delete failed: ${err}`;
    }
  }

  async function loadPreview(rule) {
    previewRule = rule;
    previewLoading = true;
    previews = [];
    selectedIds = new Set();
    success = "";
    try {
      previews = await invoke("preview_title_cleanup", { rule });
      selectedIds = new Set(previews.map((p) => p.expense_id));
      error = "";
    } catch (err) {
      error = `Preview failed: ${err}`;
    } finally {
      previewLoading = false;
    }
  }

  function toggleSelected(id) {
    const next = new Set(selectedIds);
    if (next.has(id)) next.delete(id);
    else next.add(id);
    selectedIds = next;
  }

  function selectAll() {
    selectedIds = new Set(previews.map((p) => p.expense_id));
  }

  function deselectAll() {
    selectedIds = new Set();
  }

  async function applySelected() {
    if (selectedIds.size === 0 || !previewRule) return;
    applyLoading = true;
    success = "";
    try {
      const count = await invoke("apply_title_cleanup", {
        ruleId: previewRule.id,
        expenseIds: [...selectedIds],
      });
      success = `Cleaned ${count} expense title${count !== 1 ? "s" : ""}`;
      await loadPreview(previewRule);
    } catch (err) {
      error = `Apply failed: ${err}`;
    } finally {
      applyLoading = false;
    }
  }
</script>

<div>
  <div class="flex items-center justify-between mb-6">
    <h2 class="text-2xl font-bold">Title Cleanup</h2>
    <button
      onclick={openAdd}
      class="bg-emerald-600 hover:bg-emerald-500 text-white px-4 py-2 rounded-lg
             text-sm font-medium transition-colors"
    >
      + Add Rule
    </button>
  </div>

  {#if error}
    <div class="text-sm px-4 py-2 rounded-lg bg-red-900/50 text-red-400 mb-4">{error}</div>
  {/if}

  {#if success}
    <div class="text-sm px-4 py-2 rounded-lg bg-emerald-900/50 text-emerald-400 mb-4">{success}</div>
  {/if}

  <!-- Add/Edit form -->
  {#if showForm}
    <div class="bg-gray-900 rounded-xl p-4 border border-gray-800 mb-6">
      <h3 class="text-sm font-medium text-gray-300 mb-3">
        {editingRule ? "Edit Rule" : "New Rule"}
      </h3>
      <div class="space-y-3">
        <div>
          <label class="block text-xs text-gray-500 mb-1">Pattern (text to find)</label>
          <input
            type="text"
            bind:value={formPattern}
            placeholder="e.g. PLATNOSC KARTA"
            class="w-full bg-gray-800 border border-gray-700 rounded-lg px-4 py-2
                   text-gray-100 placeholder-gray-600 focus:outline-none focus:border-emerald-500 text-sm font-mono"
          />
        </div>
        <div>
          <label class="block text-xs text-gray-500 mb-1">Replacement (empty = remove match)</label>
          <input
            type="text"
            bind:value={formReplacement}
            placeholder="Leave empty to delete the match"
            class="w-full bg-gray-800 border border-gray-700 rounded-lg px-4 py-2
                   text-gray-100 placeholder-gray-600 focus:outline-none focus:border-emerald-500 text-sm font-mono"
          />
        </div>
        <div class="flex items-center gap-3">
          <label class="flex items-center gap-2 text-sm text-gray-300 cursor-pointer">
            <input
              type="checkbox"
              bind:checked={formIsRegex}
              class="rounded bg-gray-800 border-gray-700 text-emerald-500 focus:ring-emerald-500"
            />
            Regex pattern
          </label>
          {#if formIsRegex}
            <span class="text-xs text-amber-400">Regex mode — be careful with special characters</span>
          {/if}
        </div>
        {#if formError}
          <p class="text-sm text-red-400">{formError}</p>
        {/if}
        <div class="flex gap-3">
          <button
            onclick={saveForm}
            class="bg-emerald-600 hover:bg-emerald-500 text-white px-4 py-2 rounded-lg text-sm transition-colors"
          >
            {editingRule ? "Update" : "Save"}
          </button>
          <button
            onclick={() => (showForm = false)}
            class="bg-gray-800 hover:bg-gray-700 text-gray-300 px-4 py-2 rounded-lg text-sm transition-colors"
          >
            Cancel
          </button>
        </div>
      </div>
    </div>
  {/if}

  <!-- Rules list -->
  {#if rules.length === 0}
    <div class="bg-gray-900 rounded-xl p-12 border border-gray-800 text-center text-gray-500">
      <p class="text-lg mb-2">No cleanup rules</p>
      <p class="text-sm">
        Add rules to strip noise from bank transaction titles — card numbers, payment codes, etc.
      </p>
    </div>
  {:else}
    <div class="bg-gray-900 rounded-xl border border-gray-800 overflow-hidden mb-6">
      <table class="w-full">
        <thead>
          <tr class="border-b border-gray-800 text-sm text-gray-400">
            <th class="text-left px-4 py-3">Pattern</th>
            <th class="text-left px-4 py-3">Replacement</th>
            <th class="text-left px-4 py-3 w-20">Type</th>
            <th class="w-32 px-4 py-3"></th>
          </tr>
        </thead>
        <tbody>
          {#each rules as rule}
            <tr
              class="border-b border-gray-800/50 hover:bg-gray-800/30
                {previewRule?.id === rule.id ? 'bg-gray-800/40' : ''}"
            >
              <td class="px-4 py-3 font-mono text-sm text-gray-200 max-w-md truncate"
                  title={rule.pattern}>
                {rule.pattern}
              </td>
              <td class="px-4 py-3 font-mono text-sm max-w-48 truncate"
                  title={rule.replacement || "(remove)"}>
                {#if rule.replacement}
                  <span class="text-gray-300">{rule.replacement}</span>
                {:else}
                  <span class="text-gray-600 italic">remove</span>
                {/if}
              </td>
              <td class="px-4 py-3">
                {#if rule.is_regex}
                  <span class="text-xs px-2 py-0.5 rounded bg-amber-900/40 text-amber-400">Regex</span>
                {:else}
                  <span class="text-xs px-2 py-0.5 rounded bg-gray-800 text-gray-400">Literal</span>
                {/if}
              </td>
              <td class="px-4 py-3 text-right">
                <div class="flex items-center justify-end gap-2">
                  <button
                    onclick={() => loadPreview(rule)}
                    class="text-xs px-2 py-1 rounded bg-gray-800 hover:bg-gray-700 text-gray-300
                           transition-colors"
                  >
                    Preview
                  </button>
                  <button
                    onclick={() => openEdit(rule)}
                    class="text-gray-500 hover:text-gray-300 transition-colors text-sm"
                    title="Edit rule"
                  >
                    &#x270E;
                  </button>
                  <button
                    onclick={() => (deleteTarget = rule)}
                    class="text-gray-600 hover:text-red-400 transition-colors text-sm"
                    title="Delete rule"
                  >
                    &#x2715;
                  </button>
                </div>
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
  {/if}

  <!-- Preview section -->
  {#if previewRule}
    <div class="bg-gray-900 rounded-xl border border-gray-800 overflow-hidden">
      <div class="px-4 py-3 border-b border-gray-800 flex items-center justify-between">
        <h3 class="text-sm font-medium text-gray-300">
          Preview
          <span class="text-gray-500 font-mono ml-2">{previewRule.pattern}</span>
          <span class="text-gray-600 ml-2">({previews.length} match{previews.length !== 1 ? "es" : ""})</span>
        </h3>
        <div class="flex items-center gap-2">
          {#if previews.length > 0}
            <button
              onclick={selectAll}
              class="text-xs px-2 py-1 rounded bg-gray-800 hover:bg-gray-700 text-gray-400 transition-colors"
            >
              All
            </button>
            <button
              onclick={deselectAll}
              class="text-xs px-2 py-1 rounded bg-gray-800 hover:bg-gray-700 text-gray-400 transition-colors"
            >
              None
            </button>
            <button
              onclick={applySelected}
              disabled={selectedIds.size === 0 || applyLoading}
              class="text-xs px-3 py-1 rounded bg-emerald-600 hover:bg-emerald-500
                     disabled:opacity-40 disabled:cursor-not-allowed
                     text-white font-medium transition-colors"
            >
              {applyLoading ? "Applying..." : `Apply to ${selectedIds.size}`}
            </button>
          {/if}
        </div>
      </div>

      {#if previewLoading}
        <div class="p-8 text-center text-gray-500">Loading preview...</div>
      {:else if previews.length === 0}
        <div class="p-8 text-center text-gray-500">No matching expenses found.</div>
      {:else}
        <div class="max-h-96 overflow-y-auto">
          <table class="w-full">
            <thead>
              <tr class="border-b border-gray-800 text-xs text-gray-500 sticky top-0 bg-gray-900">
                <th class="w-10 px-4 py-2"></th>
                <th class="text-left px-4 py-2">Original</th>
                <th class="w-8 px-1 py-2"></th>
                <th class="text-left px-4 py-2">Cleaned</th>
              </tr>
            </thead>
            <tbody>
              {#each previews as p}
                <tr class="border-b border-gray-800/50 hover:bg-gray-800/20">
                  <td class="px-4 py-2">
                    <input
                      type="checkbox"
                      checked={selectedIds.has(p.expense_id)}
                      onchange={() => toggleSelected(p.expense_id)}
                      class="rounded bg-gray-800 border-gray-700 text-emerald-500 focus:ring-emerald-500"
                    />
                  </td>
                  <td class="px-4 py-2 text-sm text-red-300/80 font-mono max-w-md">
                    <span class="break-all">{p.original}</span>
                  </td>
                  <td class="px-1 py-2 text-gray-600 text-center">→</td>
                  <td class="px-4 py-2 text-sm text-emerald-300/80 font-mono max-w-md">
                    <span class="break-all">{p.cleaned}</span>
                  </td>
                </tr>
              {/each}
            </tbody>
          </table>
        </div>
      {/if}
    </div>
  {/if}
</div>

<!-- Delete confirmation modal -->
{#if deleteTarget}
  <div
    class="fixed inset-0 bg-black/60 flex items-center justify-center z-50"
    role="presentation"
    onclick={(e) => { if (e.target === e.currentTarget) deleteTarget = null; }}
    onkeydown={(e) => { if (e.key === "Escape") deleteTarget = null; }}
  >
    <div class="bg-gray-900 rounded-xl p-6 border border-gray-800 w-96" role="dialog">
      <h3 class="text-lg font-semibold mb-2">Delete rule?</h3>
      <p class="text-sm text-gray-400 mb-1">Pattern:</p>
      <p class="text-sm font-mono text-gray-200 mb-4 break-all">{deleteTarget.pattern}</p>
      <div class="flex gap-3">
        <button
          onclick={confirmDelete}
          class="flex-1 bg-red-600 hover:bg-red-500 text-white py-2 rounded-lg text-sm font-medium transition-colors"
        >
          Delete
        </button>
        <button
          onclick={() => (deleteTarget = null)}
          class="flex-1 bg-gray-800 hover:bg-gray-700 text-gray-300 py-2 rounded-lg text-sm transition-colors"
        >
          Cancel
        </button>
      </div>
    </div>
  </div>
{/if}
