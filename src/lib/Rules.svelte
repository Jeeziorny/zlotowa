<script>
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import RulesFilterBar from "./rules/RulesFilterBar.svelte";
  import RulesTable from "./rules/RulesTable.svelte";
  import ConfirmModal from "./ConfirmModal.svelte";
  import PaginationBar from "./expense-list/PaginationBar.svelte";
  import EmptyState from "./EmptyState.svelte";

  let rules = $state([]);
  let totalCount = $state(0);
  let loading = $state(false);
  let fetchError = $state("");

  // Filters
  let searchText = $state("");
  let filterCategory = $state("");

  // Pagination
  let pageSize = $state(50);
  let currentPage = $state(1);

  let totalPages = $derived(Math.max(1, Math.ceil(totalCount / pageSize)));
  let showingFrom = $derived(totalCount === 0 ? 0 : (currentPage - 1) * pageSize + 1);
  let showingTo = $derived(Math.min(currentPage * pageSize, totalCount));

  let hasActiveFilters = $derived(searchText !== "" || filterCategory !== "");

  // Delete modal
  let deleteModalRule = $state(null);

  // Add form
  let showAddForm = $state(false);
  let newPattern = $state("");
  let newCategory = $state("");
  let addError = $state("");
  let adding = $state(false);

  // Categories for dropdowns
  let categories = $state([]);

  onMount(async () => {
    try {
      categories = await invoke("get_categories");
    } catch (err) {
      console.warn("Failed to load categories:", err);
    }
    await fetchRules();
  });

  async function fetchRules() {
    loading = true;
    try {
      const query = {
        search: searchText.trim() || null,
        category: filterCategory || null,
        limit: pageSize,
        offset: (currentPage - 1) * pageSize,
      };
      const result = await invoke("query_rules", { query });
      rules = result.rules;
      totalCount = result.total_count;
      fetchError = "";
    } catch (err) {
      console.error("Failed to load rules:", err);
      fetchError = "Failed to load rules. Please try again.";
    }
    loading = false;
  }

  function handleSearch(value) {
    searchText = value;
    currentPage = 1;
    fetchRules();
  }

  function handleFilterChange(value) {
    filterCategory = value;
    currentPage = 1;
    fetchRules();
  }

  function clearFilters() {
    searchText = "";
    filterCategory = "";
    currentPage = 1;
    fetchRules();
  }

  function changePageSize(newSize) {
    pageSize = newSize;
    currentPage = 1;
    fetchRules();
  }

  function prevPage() {
    if (currentPage > 1) {
      currentPage--;
      fetchRules();
    }
  }

  function nextPage() {
    if (currentPage < totalPages) {
      currentPage++;
      fetchRules();
    }
  }

  async function handleAdd() {
    if (!newPattern.trim()) {
      addError = "Pattern is required";
      return;
    }
    if (!newCategory.trim()) {
      addError = "Category is required";
      return;
    }
    adding = true;
    addError = "";
    try {
      await invoke("add_rule", {
        pattern: newPattern.trim(),
        category: newCategory.trim(),
      });
      newPattern = "";
      newCategory = "";
      showAddForm = false;
      // Refresh categories in case a new one was added
      try { categories = await invoke("get_categories"); } catch {
        fetchError = "Rule saved, but failed to refresh categories.";
      }
      await fetchRules();
    } catch (err) {
      addError = `Failed to add rule: ${err}`;
    }
    adding = false;
  }

  async function handleDeleted() {
    deleteModalRule = null;
    await fetchRules();
  }

  async function handleSaved() {
    // Refresh categories in case category was changed
    try { categories = await invoke("get_categories"); } catch {
      fetchError = "Rule saved, but failed to refresh categories.";
    }
    await fetchRules();
  }
</script>

<div>
  <div class="flex items-center justify-between mb-4">
    <h2 class="text-2xl font-bold">Classification Rules</h2>
    <button
      onclick={() => { showAddForm = !showAddForm; addError = ""; }}
      class="bg-amber-500 hover:bg-amber-400 text-gray-950 px-4 py-2 rounded-lg
             text-sm font-medium transition-colors"
    >
      {showAddForm ? "Cancel" : "+ Add Rule"}
    </button>
  </div>

  {#if showAddForm}
    <div class="bg-gray-900 rounded-xl border border-gray-800 p-4 mb-4">
      <div class="flex flex-wrap items-end gap-3">
        <div class="flex-1 min-w-48">
          <label for="new-pattern" class="block text-xs text-gray-400 mb-1">Pattern (regex)</label>
          <input
            id="new-pattern"
            type="text"
            bind:value={newPattern}
            placeholder="(?i)coffee"
            class="w-full bg-gray-800 border border-gray-700 rounded px-3 py-2 text-sm font-mono
                   text-gray-200 placeholder-gray-500 focus:border-amber-500 focus:ring-1
                   focus:ring-amber-500 focus:outline-none"
          />
        </div>
        <div class="min-w-40">
          <label for="new-category" class="block text-xs text-gray-400 mb-1">Category</label>
          <input
            id="new-category"
            type="text"
            bind:value={newCategory}
            list="add-rule-categories"
            placeholder="Food"
            class="w-full bg-gray-800 border border-gray-700 rounded px-3 py-2 text-sm
                   text-gray-200 placeholder-gray-500 focus:border-amber-500 focus:ring-1
                   focus:ring-amber-500 focus:outline-none"
          />
          <datalist id="add-rule-categories">
            {#each categories as cat}
              <option value={cat}></option>
            {/each}
          </datalist>
        </div>
        <button
          onclick={handleAdd}
          disabled={adding}
          class="bg-amber-500 hover:bg-amber-400 disabled:opacity-50 text-gray-950 px-4 py-2
                 rounded-lg text-sm font-medium transition-colors"
        >
          {adding ? "Adding..." : "Add"}
        </button>
      </div>
      {#if addError}
        <div class="text-sm text-red-400 mt-2">{addError}</div>
      {/if}
    </div>
  {/if}

  <RulesFilterBar
    {searchText}
    {filterCategory}
    {hasActiveFilters}
    {categories}
    onsearch={handleSearch}
    onfilterchange={handleFilterChange}
    onclear={clearFilters}
  />

  {#if fetchError}
    <div class="flex items-center justify-between bg-red-900/30 border border-red-800 text-red-300 rounded-lg px-4 py-2.5 mb-3 text-sm">
      <span>{fetchError}</span>
      <button onclick={() => fetchError = ""} class="text-red-400 hover:text-red-200 ml-4" aria-label="Dismiss error">×</button>
    </div>
  {/if}

  {#if loading && rules.length === 0}
    <div class="bg-gray-900 rounded-xl p-12 border border-gray-800 text-center text-gray-500">
      <p class="text-lg">Loading rules...</p>
    </div>
  {:else if rules.length === 0 && !hasActiveFilters}
    <EmptyState
      title="No classification rules yet"
      subtitle="Rules are auto-created when you manually categorize expenses, or add one above."
      icon="list"
    />
  {:else if rules.length === 0 && hasActiveFilters}
    <EmptyState
      title="No matching rules"
      subtitle="Try adjusting your search or filters."
      icon="search"
    />
  {:else}
    <RulesTable
      {rules}
      {categories}
      {loading}
      ondelete={(rule) => deleteModalRule = rule}
      onsaved={handleSaved}
    />

    <PaginationBar
      {currentPage}
      {totalPages}
      {pageSize}
      {showingFrom}
      {showingTo}
      {totalCount}
      label="rule"
      onpagesize={changePageSize}
      onprev={prevPage}
      onnext={nextPage}
    />
  {/if}

  {#if deleteModalRule}
    <ConfirmModal
      title="Delete rule?"
      onconfirm={async () => {
        await invoke("delete_rule", { id: deleteModalRule.id });
        handleDeleted();
      }}
      onclose={() => { deleteModalRule = null; }}
    >
      <p class="text-sm text-gray-400 mb-1">This cannot be undone.</p>
      <div class="text-sm text-gray-300 space-y-1">
        <p class="break-all"><span class="text-gray-500">Pattern:</span> <code class="font-mono">{deleteModalRule.pattern}</code></p>
        <p><span class="text-gray-500">Category:</span> {deleteModalRule.category}</p>
      </div>
    </ConfirmModal>
  {/if}
</div>
