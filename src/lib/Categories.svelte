<script>
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";

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

  // Inline rename
  let editingIndex = $state(null);
  let editingName = $state("");

  // Delete modal
  let deleteTarget = $state(null);
  let deleteReplacement = $state("");
  let deleteError = $state("");

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
      try {
        await invoke("rename_category", { oldName, newName: editingName.trim() });
        await loadCategories();
      } catch (err) {
        error = `Rename failed: ${err}`;
      }
    }
    editingIndex = null;
  }

  function cancelEdit() {
    editingIndex = null;
  }

  // Delete
  async function confirmDelete() {
    deleteError = "";
    if (!deleteReplacement) { deleteError = "Select a replacement category"; return; }
    try {
      await invoke("delete_category", { category: deleteTarget.name, replacement: deleteReplacement });
      deleteTarget = null;
      deleteReplacement = "";
      selected = new Set();
      await loadCategories();
    } catch (err) {
      deleteError = `${err}`;
    }
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
      class="bg-emerald-600 hover:bg-emerald-500 text-white px-4 py-2 rounded-lg
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
          onkeydown={(e) => e.key === "Enter" && handleCreate()}
          class="flex-1 bg-gray-800 border border-gray-700 rounded-lg px-4 py-2
                 text-gray-100 placeholder-gray-600 focus:outline-none focus:border-emerald-500 text-sm"
        />
        <button onclick={handleCreate}
          class="bg-emerald-600 hover:bg-emerald-500 text-white px-4 py-2 rounded-lg text-sm transition-colors">
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
      class="flex-1 bg-gray-900 border border-gray-800 rounded-lg px-4 py-2
             text-gray-100 placeholder-gray-600 focus:outline-none focus:border-emerald-500 text-sm"
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
    <div class="bg-gray-900 rounded-xl p-12 border border-gray-800 text-center text-gray-500">
      <div class="w-8 h-8 border-4 border-gray-700 border-t-emerald-500 rounded-full animate-spin mx-auto mb-3"></div>
      <p class="text-sm">Loading categories...</p>
    </div>
  {:else if categories.length === 0}
    <div class="bg-gray-900 rounded-xl p-12 border border-gray-800 text-center text-gray-500">
      <p class="text-lg mb-2">No categories yet</p>
      <p class="text-sm">Categories are created automatically when you categorize expenses.</p>
    </div>
  {:else}
    <div class="bg-gray-900 rounded-xl border border-gray-800 overflow-hidden">
      <table class="w-full">
        <thead>
          <tr class="border-b border-gray-800 text-sm text-gray-400">
            <th class="w-10 px-4 py-3"></th>
            <th class="text-left px-4 py-3 cursor-pointer select-none hover:text-gray-200"
                tabindex="0"
                aria-sort={sortBy === "name" ? (sortAsc ? "ascending" : "descending") : "none"}
                onclick={() => toggleSort("name")}
                onkeydown={(e) => { if (e.key === "Enter" || e.key === " ") { e.preventDefault(); toggleSort("name"); } }}>
              Name{sortIndicator("name")}
            </th>
            <th class="text-right px-4 py-3 cursor-pointer select-none hover:text-gray-200"
                tabindex="0"
                aria-sort={sortBy === "expenses" ? (sortAsc ? "ascending" : "descending") : "none"}
                onclick={() => toggleSort("expenses")}
                onkeydown={(e) => { if (e.key === "Enter" || e.key === " ") { e.preventDefault(); toggleSort("expenses"); } }}>
              Expenses{sortIndicator("expenses")}
            </th>
            <th class="text-right px-4 py-3 cursor-pointer select-none hover:text-gray-200"
                tabindex="0"
                aria-sort={sortBy === "rules" ? (sortAsc ? "ascending" : "descending") : "none"}
                onclick={() => toggleSort("rules")}
                onkeydown={(e) => { if (e.key === "Enter" || e.key === " ") { e.preventDefault(); toggleSort("rules"); } }}>
              Rules{sortIndicator("rules")}
            </th>
            <th class="w-16 px-4 py-3"></th>
          </tr>
        </thead>
        <tbody>
          {#each filtered as cat, i}
            <tr class="border-b border-gray-800/50 hover:bg-gray-800/30">
              <td class="px-4 py-3">
                <input
                  type="checkbox"
                  checked={selected.has(cat.name)}
                  onchange={() => toggleSelect(cat.name)}
                  class="rounded bg-gray-800 border-gray-700 text-emerald-500 focus:ring-emerald-500"
                />
              </td>
              <td class="px-4 py-3">
                {#if editingIndex === i}
                  <input
                    type="text"
                    bind:value={editingName}
                    onkeydown={(e) => {
                      if (e.key === "Enter") saveEdit(cat.name);
                      if (e.key === "Escape") cancelEdit();
                    }}
                    onblur={() => saveEdit(cat.name)}
                    class="bg-gray-800 border border-emerald-500 rounded px-2 py-1
                           text-gray-100 focus:outline-none w-full max-w-64"
                  />
                {:else}
                  <button
                    onclick={() => startEdit(i, cat.name)}
                    class="text-left hover:text-emerald-400 transition-colors"
                    title="Click to rename"
                  >
                    {cat.name}
                  </button>
                {/if}
              </td>
              <td class="px-4 py-3 text-right text-gray-400 font-mono text-sm">{cat.expense_count}</td>
              <td class="px-4 py-3 text-right text-gray-400 font-mono text-sm">{cat.rule_count}</td>
              <td class="px-4 py-3 text-right">
                <button
                  onclick={() => { deleteTarget = cat; deleteReplacement = ""; deleteError = ""; }}
                  class="text-gray-600 hover:text-red-400 transition-colors text-sm"
                  title="Delete category"
                  aria-label="Delete category {cat.name}"
                >
                  &#x2715;
                </button>
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
  {/if}
</div>

<!-- Delete modal -->
{#if deleteTarget}
  <div
    class="fixed inset-0 bg-black/60 flex items-center justify-center z-50"
    role="presentation"
    onclick={(e) => { if (e.target === e.currentTarget) deleteTarget = null; }}
    onkeydown={(e) => { if (e.key === "Escape") deleteTarget = null; }}
  >
    <div class="bg-gray-900 rounded-xl p-6 border border-gray-800 w-96" role="dialog" aria-modal="true" aria-labelledby="delete-category-modal-title">
      <h3 id="delete-category-modal-title" class="text-lg font-semibold mb-2">Delete "{deleteTarget.name}"</h3>
      <p class="text-sm text-gray-400 mb-4">
        Reassign {deleteTarget.expense_count} expenses and {deleteTarget.rule_count} rules to:
      </p>
      <select
        bind:value={deleteReplacement}
        class="w-full bg-gray-800 border border-gray-700 rounded-lg px-4 py-2
               text-gray-100 text-sm focus:outline-none focus:border-emerald-500 mb-4"
      >
        <option value="">Select replacement...</option>
        {#each categories.filter(c => c.name !== deleteTarget.name) as cat}
          <option value={cat.name}>{cat.name}</option>
        {/each}
      </select>
      {#if deleteError}
        <p class="text-sm text-red-400 mb-3">{deleteError}</p>
      {/if}
      <div class="flex gap-3">
        <button onclick={confirmDelete}
          class="flex-1 bg-red-600 hover:bg-red-500 text-white py-2 rounded-lg text-sm font-medium transition-colors">
          Delete
        </button>
        <button onclick={() => deleteTarget = null}
          class="flex-1 bg-gray-800 hover:bg-gray-700 text-gray-300 py-2 rounded-lg text-sm transition-colors">
          Cancel
        </button>
      </div>
    </div>
  </div>
{/if}

<!-- Merge modal -->
{#if showMerge}
  <div
    class="fixed inset-0 bg-black/60 flex items-center justify-center z-50"
    role="presentation"
    onclick={(e) => { if (e.target === e.currentTarget) showMerge = false; }}
    onkeydown={(e) => { if (e.key === "Escape") showMerge = false; }}
  >
    <div class="bg-gray-900 rounded-xl p-6 border border-gray-800 w-96" role="dialog" aria-modal="true" aria-labelledby="merge-modal-title">
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
        onkeydown={(e) => e.key === "Enter" && confirmMerge()}
        class="w-full bg-gray-800 border border-gray-700 rounded-lg px-4 py-2
               text-gray-100 text-sm focus:outline-none focus:border-emerald-500 mb-4"
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
