<script>
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import SearchFilterBar from "./expense-list/SearchFilterBar.svelte";
  import ExpenseTable from "./expense-list/ExpenseTable.svelte";
  import PaginationBar from "./expense-list/PaginationBar.svelte";
  import AddExpense from "./AddExpense.svelte";
  import BulkUpload from "./BulkUpload.svelte";
  import ConfirmModal from "./ConfirmModal.svelte";
  import EmptyState from "./EmptyState.svelte";
  import { DEFAULT_PAGE_SIZE } from "./constants.js";

  let { onbulkdirtychange = () => {}, subView = $bindable("list") } = $props();
  let bulkUploadDirty = $state(false);
  let showLeaveConfirm = $state(false);

  function handleBulkDirtyChange(dirty) {
    bulkUploadDirty = dirty;
    onbulkdirtychange(dirty);
  }

  function handleBackToExpenses() {
    if (bulkUploadDirty) {
      showLeaveConfirm = true;
    } else {
      subView = "list";
      fetchExpenses();
    }
  }

  function confirmLeave() {
    showLeaveConfirm = false;
    bulkUploadDirty = false;
    onbulkdirtychange(false);
    subView = "list";
    fetchExpenses();
  }

  let expenses = $state([]);
  let totalCount = $state(0);
  let loading = $state(false);

  // Search & filters
  let searchText = $state("");
  let filterCategory = $state("");
  let filterDateFrom = $state("");
  let filterDateTo = $state("");
  let filterAmountMin = $state("");
  let filterAmountMax = $state("");

  // Pagination
  let pageSize = $state(DEFAULT_PAGE_SIZE);
  let currentPage = $state(1);

  let totalPages = $derived(Math.max(1, Math.ceil(totalCount / pageSize)));
  let showingFrom = $derived(totalCount === 0 ? 0 : (currentPage - 1) * pageSize + 1);
  let showingTo = $derived(Math.min(currentPage * pageSize, totalCount));

  let hasActiveFilters = $derived(
    searchText !== "" || filterCategory !== "" || filterDateFrom !== "" ||
    filterDateTo !== "" || filterAmountMin !== "" || filterAmountMax !== ""
  );

  let activeFilterCount = $derived(
    [searchText, filterCategory, filterDateFrom, filterDateTo, filterAmountMin, filterAmountMax]
      .filter(Boolean).length
  );

  // Delete
  let deleteModalExpense = $state(null);

  // Batch select/delete
  let selected = $state(new Set());
  let confirmBatchDelete = $state(false);

  let allSelected = $derived(expenses.length > 0 && selected.size === expenses.length);
  let someSelected = $derived(selected.size > 0);

  // Categories for filters & edit dropdown
  let categories = $state([]);

  onMount(async () => {
    try {
      categories = await invoke("get_categories");
    } catch (err) {
      console.warn("Failed to load categories:", err);
    }
    await fetchExpenses();
  });

  async function fetchExpenses() {
    loading = true;
    try {
      const query = {
        search: searchText.trim() || null,
        category: filterCategory || null,
        date_from: filterDateFrom || null,
        date_to: filterDateTo || null,
        amount_min: filterAmountMin !== "" ? parseFloat(filterAmountMin) : null,
        amount_max: filterAmountMax !== "" ? parseFloat(filterAmountMax) : null,
        limit: pageSize,
        offset: (currentPage - 1) * pageSize,
      };
      const result = await invoke("query_expenses", { query });
      expenses = result.expenses;
      totalCount = result.total_count;
      selected = new Set();
    } catch (err) {
      console.error("Failed to load expenses:", err);
    }
    loading = false;
  }

  function handleSearch(value) {
    searchText = value;
    currentPage = 1;
    fetchExpenses().catch(() => {});
  }

  function handleFilterChange(field, value) {
    if (field === "filterCategory") filterCategory = value;
    else if (field === "filterDateFrom") filterDateFrom = value;
    else if (field === "filterDateTo") filterDateTo = value;
    else if (field === "filterAmountMin") filterAmountMin = value;
    else if (field === "filterAmountMax") filterAmountMax = value;
    currentPage = 1;
    fetchExpenses().catch(() => {});
  }

  function clearFilters() {
    searchText = "";
    filterCategory = "";
    filterDateFrom = "";
    filterDateTo = "";
    filterAmountMin = "";
    filterAmountMax = "";
    currentPage = 1;
    fetchExpenses().catch(() => {});
  }

  function changePageSize(newSize) {
    pageSize = newSize;
    currentPage = 1;
    fetchExpenses().catch(() => {});
  }

  function prevPage() {
    if (currentPage > 1) {
      currentPage--;
      fetchExpenses().catch(() => {});
    }
  }

  function nextPage() {
    if (currentPage < totalPages) {
      currentPage++;
      fetchExpenses().catch(() => {});
    }
  }

  // ── Selection ──

  function toggleSelect(id) {
    if (selected.has(id)) {
      selected.delete(id);
    } else {
      selected.add(id);
    }
    selected = new Set(selected);
  }

  function toggleSelectAll() {
    if (allSelected) {
      selected = new Set();
    } else {
      selected = new Set(expenses.map(e => e.id));
    }
  }
</script>

<div>
  {#if subView !== "list"}
    <button
      onclick={handleBackToExpenses}
      class="text-gray-400 hover:text-amber-400 text-sm mb-4 inline-flex items-center gap-1 transition-colors"
    >
      <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7" />
      </svg>
      Back to Expenses
    </button>
    {#if subView === "add"}
      <AddExpense />
    {:else if subView === "bulk"}
      <BulkUpload ondirtychange={handleBulkDirtyChange} />
    {/if}

    {#if showLeaveConfirm}
      <ConfirmModal
        title="Leave bulk upload?"
        confirmLabel="Leave"
        onconfirm={async () => { confirmLeave(); }}
        onclose={() => { showLeaveConfirm = false; }}
      >
        <p class="text-sm text-gray-400">You'll lose your upload progress.</p>
      </ConfirmModal>
    {/if}
  {:else}
  <div class="flex items-center justify-between mb-4">
    <h2 class="text-2xl font-bold">Expenses</h2>
    <div class="flex gap-2">
      {#if someSelected}
        <button
          onclick={() => confirmBatchDelete = true}
          class="bg-red-900/50 hover:bg-red-900/80 text-red-400 px-4 py-2 rounded-lg
                 text-sm font-medium transition-colors border border-red-800"
        >
          Delete {selected.size} selected
        </button>
      {/if}
      <button
        onclick={() => subView = "add"}
        class="bg-gray-800 hover:bg-gray-700 text-gray-300 px-4 py-2 rounded-lg
               text-sm font-medium transition-colors border border-gray-700"
      >
        + Add manually
      </button>
      <button
        onclick={() => subView = "bulk"}
        class="bg-amber-500 hover:bg-amber-400 text-gray-950 px-4 py-2 rounded-lg
               text-sm font-medium transition-colors"
      >
        + Import CSV
      </button>
    </div>
  </div>

  <SearchFilterBar
    {searchText}
    {filterCategory}
    {filterDateFrom}
    {filterDateTo}
    {filterAmountMin}
    {filterAmountMax}
    {hasActiveFilters}
    {categories}
    onsearch={handleSearch}
    onfilterchange={handleFilterChange}
    onclear={clearFilters}
  />

  {#if hasActiveFilters}
    <div class="flex flex-wrap gap-2 mb-3">
      {#if searchText}
        <span class="inline-flex items-center gap-1.5 bg-gray-800 text-gray-300 border border-gray-700 rounded-full px-3 py-1 text-xs">
          <span class="text-gray-500">Search:</span>
          <span>"{searchText}"</span>
          <button onclick={() => handleSearch("")} class="text-gray-500 hover:text-red-400 transition-colors" aria-label="Remove search filter">×</button>
        </span>
      {/if}
      {#if filterCategory}
        <span class="inline-flex items-center gap-1.5 bg-gray-800 text-gray-300 border border-gray-700 rounded-full px-3 py-1 text-xs">
          <span class="text-gray-500">Category:</span>
          <span>{filterCategory}</span>
          <button onclick={() => handleFilterChange("filterCategory", "")} class="text-gray-500 hover:text-red-400 transition-colors" aria-label="Remove category filter">×</button>
        </span>
      {/if}
      {#if filterDateFrom}
        <span class="inline-flex items-center gap-1.5 bg-gray-800 text-gray-300 border border-gray-700 rounded-full px-3 py-1 text-xs">
          <span class="text-gray-500">From:</span>
          <span>{filterDateFrom}</span>
          <button onclick={() => handleFilterChange("filterDateFrom", "")} class="text-gray-500 hover:text-red-400 transition-colors" aria-label="Remove date from filter">×</button>
        </span>
      {/if}
      {#if filterDateTo}
        <span class="inline-flex items-center gap-1.5 bg-gray-800 text-gray-300 border border-gray-700 rounded-full px-3 py-1 text-xs">
          <span class="text-gray-500">To:</span>
          <span>{filterDateTo}</span>
          <button onclick={() => handleFilterChange("filterDateTo", "")} class="text-gray-500 hover:text-red-400 transition-colors" aria-label="Remove date to filter">×</button>
        </span>
      {/if}
      {#if filterAmountMin}
        <span class="inline-flex items-center gap-1.5 bg-gray-800 text-gray-300 border border-gray-700 rounded-full px-3 py-1 text-xs">
          <span class="text-gray-500">Min amount:</span>
          <span>{filterAmountMin}</span>
          <button onclick={() => handleFilterChange("filterAmountMin", "")} class="text-gray-500 hover:text-red-400 transition-colors" aria-label="Remove min amount filter">×</button>
        </span>
      {/if}
      {#if filterAmountMax}
        <span class="inline-flex items-center gap-1.5 bg-gray-800 text-gray-300 border border-gray-700 rounded-full px-3 py-1 text-xs">
          <span class="text-gray-500">Max amount:</span>
          <span>{filterAmountMax}</span>
          <button onclick={() => handleFilterChange("filterAmountMax", "")} class="text-gray-500 hover:text-red-400 transition-colors" aria-label="Remove max amount filter">×</button>
        </span>
      {/if}
      {#if activeFilterCount >= 2}
        <button onclick={clearFilters} class="inline-flex items-center bg-gray-800 text-gray-500 hover:text-gray-300 border border-gray-700 rounded-full px-3 py-1 text-xs transition-colors">
          Clear all
        </button>
      {/if}
    </div>
  {/if}

  {#if loading && expenses.length === 0}
    <div class="bg-gray-900 rounded-xl p-12 border border-gray-800 text-center text-gray-500">
      <p class="text-lg">Loading expenses...</p>
    </div>
  {:else if expenses.length === 0 && !hasActiveFilters}
    <EmptyState
      title="No expenses yet"
      subtitle="Add an expense or do a bulk upload to get started."
      icon="inbox"
    />
  {:else if expenses.length === 0 && hasActiveFilters}
    <EmptyState
      title="No matching expenses"
      subtitle="Try adjusting your search or filters."
      icon="search"
    />
  {:else}
    <ExpenseTable
      {expenses}
      {categories}
      {loading}
      {selected}
      {allSelected}
      onselect={toggleSelect}
      onselectall={toggleSelectAll}
      ondelete={(expense) => deleteModalExpense = expense}
      onsaved={fetchExpenses}
    />

    <PaginationBar
      {currentPage}
      {totalPages}
      {pageSize}
      {showingFrom}
      {showingTo}
      {totalCount}
      onpagesize={changePageSize}
      onprev={prevPage}
      onnext={nextPage}
    />
  {/if}

  {#if deleteModalExpense}
    <ConfirmModal
      title="Delete expense?"
      onconfirm={async () => {
        await invoke("delete_expense", { id: deleteModalExpense.id });
        deleteModalExpense = null;
        fetchExpenses();
      }}
      onclose={() => { deleteModalExpense = null; }}
    >
      <p class="text-sm text-gray-400 mb-1">This cannot be undone.</p>
      <p class="text-sm text-gray-300 break-words">
        "{deleteModalExpense.title}" &mdash; {deleteModalExpense.amount.toFixed(2)}
      </p>
    </ConfirmModal>
  {/if}

  {#if confirmBatchDelete}
    <ConfirmModal
      title="Delete {selected.size} expense{selected.size > 1 ? 's' : ''}?"
      confirmLabel="Delete All"
      onconfirm={async () => {
        const ids = [...selected];
        await invoke("delete_expenses", { ids });
        confirmBatchDelete = false;
        fetchExpenses();
      }}
      onclose={() => { confirmBatchDelete = false; }}
    >
      <p class="text-sm text-gray-400">This cannot be undone.</p>
    </ConfirmModal>
  {/if}
  {/if}
</div>
