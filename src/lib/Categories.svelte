<script>
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";
  import ConfirmModal from "./ConfirmModal.svelte";
  import { focusTrap } from "./actions/focusTrap.js";
  import EmptyState from "./EmptyState.svelte";
  import Skeleton from "./Skeleton.svelte";

  let categories = $state([]);
  let loaded = $state(false);
  let search = $state("");
  let sortBy = $state("name");
  let sortAsc = $state(true);
  let error = $state("");

  // Create
  let showCreate = $state(false);
  let newName = $state("");
  let createError = $state("");

  // Inline edit
  let editingIndex = $state(null);
  let editingName = $state("");
  let editSaving = $state(false);
  let sortAnnouncement = $state("");

  // Delete modal
  let deleteTarget = $state(null);
  let deleteReplacement = $state("");

  // Merge
  let selected = $state(new Set());
  let showMerge = $state(false);
  let mergeTarget = $state("");
  let mergeError = $state("");

  let filtered = $derived(
    categories
      .filter(c => c.name.toLowerCase().includes(search.toLowerCase()))
      .sort((a, b) => {
        let cmp = 0;
        if (sortBy === "name") cmp = a.name.localeCompare(b.name);
        else if (sortBy === "expenses") cmp = a.expense_count - b.expense_count;
        else if (sortBy === "rules") cmp = a.rule_count - b.rule_count;
        return sortAsc ? cmp : -cmp;
      })
  );

  onMount(loadCategories);

  async function loadCategories() {
    try {
      categories = await invoke("get_category_stats");
      error = "";
    } catch (err) {
      error = `Failed to load categories: ${err}`;
    }
    loaded = true;
  }

  function toggleSort(col) {
    if (sortBy === col) {
      sortAsc = !sortAsc;
    } else {
      sortBy = col;
      sortAsc = col === "name";
    }
    const colLabel = col === "name" ? "Name" : col === "expenses" ? "Expenses" : "Rules";
    sortAnnouncement = `Sorted by ${colLabel}, ${sortAsc ? "ascending" : "descending"}`;
  }

  function sortIndicator(col) {
    if (sortBy !== col) return "";
    return sortAsc ? " ↑" : " ↓";
  }

  // Create
  async function handleCreate() {
    createError = "";
    if (!newName.trim()) { createError = "Name cannot be empty"; return; }
    try {
      await invoke("create_category", { name: newName.trim() });
      newName = "";
      showCreate = false;
      await loadCategories();
    } catch (err) {
      createError = `${err}`;
    }
  }

  // Inline rename
  function startEdit(index, name) {
    editingIndex = index;
    editingName = name;
  }

  async function saveEdit(oldName) {
    if (editingName.trim() && editingName.trim() !== oldName) {
      editSaving = true;
      try {
        await invoke("rename_category", { oldName, newName: editingName.trim() });
        await loadCategories();
      } catch (err) {
        error = `Rename failed: ${err}`;
      }
      editSaving = false;
    }
    editingIndex = null;
  }

  function cancelEdit() {
    editingIndex = null;
  }


  // Merge
  function toggleSelect(name) {
    const next = new Set(selected);
    if (next.has(name)) next.delete(name);
    else next.add(name);
    selected = next;
  }

  function openMerge() {
    mergeTarget = [...selected][0] || "";
    mergeError = "";
    showMerge = true;
  }

  async function confirmMerge() {
    mergeError = "";
    if (!mergeTarget.trim()) { mergeError = "Enter a target name"; return; }
    try {
      await invoke("merge_categories", { sources: [...selected], target: mergeTarget.trim() });
      showMerge = false;
      selected = new Set();
      await loadCategories();
    } catch (err) {
      mergeError = `${err}`;
    }
  }
</script>

<div>
  <div class="flex items-center justify-between mb-6">
    <h2 class="text-2xl font-bold">Categories</h2>
    <button
      onclick={() => { showCreate = !showCreate; createError = ""; newName = ""; }}
      class="bg-amber-500 hover:bg-amber-400 text-gray-950 px-4 py-2 rounded-lg
             text-sm font-medium transition-colors"
    >
      + New
    </button>
  </div>

  {#if showCreate}
    <div class="bg-gray-900 rounded-xl p-4 border border-gray-800 mb-6">
      <div class="flex gap-3">
        <input
          type="text"
          bind:value={newName}
          placeholder="Category name"
          maxlength="100"
          aria-label="New category name"
          onkeydown={(e) => e.key === "Enter" && handleCreate()}
          class="flex-1 bg-gray-800 border border-gray-700 rounded-lg px-4 py-2
                 text-gray-100 placeholder-gray-600 focus:outline-none focus:border-amber-500 text-sm"
        />
        <button onclick={handleCreate}
          class="bg-amber-500 hover:bg-amber-400 text-gray-950 px-4 py-2 rounded-lg text-sm transition-colors">
          Create
        </button>
        <button onclick={() => showCreate = false}
          class="bg-gray-800 hover:bg-gray-700 text-gray-300 px-4 py-2 rounded-lg text-sm transition-colors">
          Cancel
        </button>
      </div>
      {#if createError}
        <p class="text-sm text-red-400 mt-2">{createError}</p>
      {/if}
    </div>
  {/if}

  {#if error}
    <div class="text-sm px-4 py-2 rounded-lg bg-red-900/50 text-red-400 mb-4">{error}</div>
  {/if}

  <!-- Search + merge bar -->
  <div class="flex items-center gap-3 mb-4">
    <input
      type="text"
      bind:value={search}
      placeholder="Search categories..."
      aria-label="Search categories"
      class="flex-1 bg-gray-900 border border-gray-800 rounded-lg px-4 py-2
             text-gray-100 placeholder-gray-600 focus:outline-none focus:border-amber-500 text-sm"
    />
    {#if selected.size >= 2}
      <button
        onclick={openMerge}
        class="bg-purple-600 hover:bg-purple-500 text-white px-4 py-2 rounded-lg text-sm font-medium transition-colors"
      >
        Merge {selected.size}
      </button>
    {/if}
  </div>

  {#if !loaded}
    <div class="bg-gray-900 rounded-xl border border-gray-800 overflow-hidden p-1">
      <Skeleton variant="row" count={6} />
    </div>
  {:else if categories.length === 0}
    <EmptyState
      title="No categories yet"
      subtitle="Categories are created automatically when you categorize expenses."
      icon="tag"
    />
  {:else}
    <div class="bg-gray-900 rounded-xl border border-gray-800 overflow-hidden">
      <table class="w-full">
        <thead>
          <tr class="border-b border-gray-800 text-sm text-gray-400">
            <th class="w-10 px-4 py-3"></th>
            <th class="text-left px-4 py-3 cursor-pointer select-none hover:text-gray-200"
                tabindex="0"
                aria-sort={sortBy === "name" ? (sortAsc ? "ascending" : "descending") : "none"}
                aria-label="Sort by Name"
                onclick={() => toggleSort("name")}
                onkeydown={(e) => { if (e.key === "Enter" || e.key === " ") { e.preventDefault(); toggleSort("name"); } }}>
              Name{sortIndicator("name")}
            </th>
            <th class="text-right px-4 py-3 cursor-pointer select-none hover:text-gray-200"
                tabindex="0"
                aria-sort={sortBy === "expenses" ? (sortAsc ? "ascending" : "descending") : "none"}
                aria-label="Sort by Expenses"
                onclick={() => toggleSort("expenses")}
                onkeydown={(e) => { if (e.key === "Enter" || e.key === " ") { e.preventDefault(); toggleSort("expenses"); } }}>
              Expenses{sortIndicator("expenses")}
            </th>
            <th class="text-right px-4 py-3 cursor-pointer select-none hover:text-gray-200"
                tabindex="0"
                aria-sort={sortBy === "rules" ? (sortAsc ? "ascending" : "descending") : "none"}
                aria-label="Sort by Rules"
                onclick={() => toggleSort("rules")}
                onkeydown={(e) => { if (e.key === "Enter" || e.key === " ") { e.preventDefault(); toggleSort("rules"); } }}>
              Rules{sortIndicator("rules")}
            </th>
            <th class="w-24 px-4 py-3"></th>
          </tr>
        </thead>
        <tbody>
          {#each filtered as cat, i}
            {#if editingIndex === i}
              <tr class="border-b border-gray-800/50 bg-gray-800/20 border-l-2 border-l-amber-500/40">
                <td class="px-4 py-2"></td>
                <td class="px-4 py-2">
                  <input
                    type="text"
                    bind:value={editingName}
                    maxlength="100"
                    onkeydown={(e) => {
                      if (e.key === "Enter") saveEdit(cat.name);
                      if (e.key === "Escape") cancelEdit();
                    }}
                    class="w-full bg-gray-800 border border-gray-700 rounded px-2 py-1 text-sm
                           text-gray-200 focus:border-amber-500 focus:ring-1 focus:ring-amber-500 focus:outline-none"
                  />
                </td>
                <td class="px-4 py-2"></td>
                <td class="px-4 py-2"></td>
                <td class="px-4 py-2">
                  <div class="flex gap-1 justify-end">
                    <button
                      onclick={() => saveEdit(cat.name)}
                      disabled={editSaving}
                      class="text-amber-400 hover:text-amber-300 disabled:opacity-50 p-1 transition-colors"
                      title="Save"
                      aria-label="Save rename"
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
                </td>
              </tr>
            {:else}
              <tr class="border-b border-gray-800/50 hover:bg-gray-800/30 group border-l-2 border-l-transparent group-hover:border-l-amber-500/40 transition-colors">
                <td class="px-4 py-3">
                  <input
                    type="checkbox"
                    checked={selected.has(cat.name)}
                    onchange={() => toggleSelect(cat.name)}
                    class="rounded bg-gray-800 border-gray-700 text-amber-500 focus:ring-amber-500"
                  />
                </td>
                <td class="px-4 py-3">{cat.name}</td>
                <td class="px-4 py-3 text-right text-gray-400 font-mono text-sm">{cat.expense_count}</td>
                <td class="px-4 py-3 text-right text-gray-400 font-mono text-sm">{cat.rule_count}</td>
                <td class="px-4 py-3">
                  <div class="flex gap-1 opacity-0 group-hover:opacity-100 group-focus-within:opacity-100 transition-opacity justify-end">
                    <button
                      onclick={() => startEdit(i, cat.name)}
                      class="text-gray-500 hover:text-gray-300 p-1 transition-colors"
                      title="Rename"
                      aria-label="Rename category {cat.name}"
                    >
                      <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                          d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z" />
                      </svg>
                    </button>
                    <button
                      onclick={() => { deleteTarget = cat; deleteReplacement = ""; }}
                      class="text-gray-500 hover:text-red-400 p-1 transition-colors"
                      title="Delete"
                      aria-label="Delete category {cat.name}"
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
      <div class="sr-only" role="status" aria-live="polite" aria-atomic="true">
        {sortAnnouncement}
      </div>
    </div>
  {/if}
</div>

<!-- Delete modal -->
{#if deleteTarget}
  <ConfirmModal
    title='Delete "{deleteTarget.name}"'
    onconfirm={async () => {
      if (!deleteReplacement) { throw new Error("Select a replacement category"); }
      await invoke("delete_category", { category: deleteTarget.name, replacement: deleteReplacement });
      deleteTarget = null;
      deleteReplacement = "";
      selected = new Set();
      await loadCategories();
    }}
    onclose={() => { deleteTarget = null; }}
  >
    <p class="text-sm text-gray-400 mb-4">
      Reassign {deleteTarget.expense_count} expenses and {deleteTarget.rule_count} rules to:
    </p>
    <select
      bind:value={deleteReplacement}
      class="w-full bg-gray-800 border border-gray-700 rounded-lg px-4 py-2
             text-gray-100 text-sm focus:outline-none focus:border-amber-500"
    >
      <option value="">Select replacement...</option>
      {#each categories.filter(c => c.name !== deleteTarget.name) as cat}
        <option value={cat.name}>{cat.name}</option>
      {/each}
    </select>
  </ConfirmModal>
{/if}

<!-- Merge modal -->
{#if showMerge}
  <div
    class="fixed inset-0 bg-black/60 flex items-center justify-center z-50"
    role="presentation"
    onclick={(e) => { if (e.target === e.currentTarget) showMerge = false; }}
    onkeydown={(e) => { if (e.key === "Escape") showMerge = false; }}
  >
    <div class="bg-gray-900 rounded-xl p-6 border border-gray-800 w-96"
         role="dialog" aria-modal="true" aria-labelledby="merge-modal-title"
         tabindex="-1"
         use:focusTrap
         onclick={(e) => e.stopPropagation()}>
      <h3 id="merge-modal-title" class="text-lg font-semibold mb-2">Merge {selected.size} categories</h3>
      <p class="text-sm text-gray-400 mb-1">Merging:</p>
      <div class="flex flex-wrap gap-1 mb-4">
        {#each [...selected] as s}
          <span class="px-2 py-0.5 rounded text-xs bg-gray-800 text-gray-300">{s}</span>
        {/each}
      </div>
      <label for="merge-target-name" class="block text-sm text-gray-400 mb-1">Target name</label>
      <input
        id="merge-target-name"
        type="text"
        bind:value={mergeTarget}
        maxlength="100"
        onkeydown={(e) => e.key === "Enter" && confirmMerge()}
        class="w-full bg-gray-800 border border-gray-700 rounded-lg px-4 py-2
               text-gray-100 text-sm focus:outline-none focus:border-amber-500 mb-4"
      />
      {#if mergeError}
        <p class="text-sm text-red-400 mb-3">{mergeError}</p>
      {/if}
      <div class="flex gap-3">
        <button onclick={confirmMerge}
          class="flex-1 bg-purple-600 hover:bg-purple-500 text-white py-2 rounded-lg text-sm font-medium transition-colors">
          Merge
        </button>
        <button onclick={() => showMerge = false}
          class="flex-1 bg-gray-800 hover:bg-gray-700 text-gray-300 py-2 rounded-lg text-sm transition-colors">
          Cancel
        </button>
      </div>
    </div>
  </div>
{/if}
