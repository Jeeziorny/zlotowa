<script>
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import SearchFilterBar from "./expense-list/SearchFilterBar.svelte";
  import ExpenseTable from "./expense-list/ExpenseTable.svelte";
  import PaginationBar from "./expense-list/PaginationBar.svelte";
  import DeleteConfirmModal from "./expense-list/DeleteConfirmModal.svelte";
  import BatchDeleteModal from "./expense-list/BatchDeleteModal.svelte";
  import AddExpense from "./AddExpense.svelte";
  import BulkUpload from "./BulkUpload.svelte";
  import TitleCleanup from "./TitleCleanup.svelte";

  // Sub-view: "list" | "add" | "bulk" | "cleanup"
  let subView = $state("list");

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
  let pageSize = $state(50);
  let currentPage = $state(1);

  let totalPages = $derived(Math.max(1, Math.ceil(totalCount / pageSize)));
  let showingFrom = $derived(totalCount === 0 ? 0 : (currentPage - 1) * pageSize + 1);
  let showingTo = $derived(Math.min(currentPage * pageSize, totalCount));

  let hasActiveFilters = $derived(
    searchText !== "" || filterCategory !== "" || filterDateFrom !== "" ||
    filterDateTo !== "" || filterAmountMin !== "" || filterAmountMax !== ""
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
      onclick={() => { subView = "list"; fetchExpenses(); }}
      class="text-gray-400 hover:text-emerald-400 text-sm mb-4 inline-flex items-center gap-1 transition-colors"
    >
      <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7" />
      </svg>
      Back to Expenses
    </button>
    {#if subView === "add"}
      <AddExpense />
    {:else if subView === "bulk"}
      <BulkUpload />
    {:else if subView === "cleanup"}
      <TitleCleanup />
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
        class="bg-emerald-600 hover:bg-emerald-500 text-white px-4 py-2 rounded-lg
               text-sm font-medium transition-colors"
      >
        + Add
      </button>
      <button
        onclick={() => subView = "bulk"}
        class="bg-gray-800 hover:bg-gray-700 text-gray-200 px-4 py-2 rounded-lg
               text-sm font-medium transition-colors border border-gray-700"
      >
        Upload CSV
      </button>
      <button
        onclick={() => subView = "cleanup"}
        class="bg-gray-800 hover:bg-gray-700 text-gray-200 px-4 py-2 rounded-lg
               text-sm font-medium transition-colors border border-gray-700"
      >
        Clean Titles
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

  {#if loading && expenses.length === 0}
    <div class="bg-gray-900 rounded-xl p-12 border border-gray-800 text-center text-gray-500">
      <p class="text-lg">Loading expenses...</p>
    </div>
  {:else if expenses.length === 0 && !hasActiveFilters}
    <div class="bg-gray-900 rounded-xl p-12 border border-gray-800 text-center text-gray-500">
      <p class="text-lg mb-2">No expenses yet</p>
      <p class="text-sm">Add an expense or do a bulk upload to get started.</p>
    </div>
  {:else if expenses.length === 0 && hasActiveFilters}
    <div class="bg-gray-900 rounded-xl p-12 border border-gray-800 text-center text-gray-500">
      <p class="text-lg mb-2">No matching expenses</p>
      <p class="text-sm">Try adjusting your search or filters.</p>
    </div>
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
    <DeleteConfirmModal
      expense={deleteModalExpense}
      ondelete={() => { deleteModalExpense = null; fetchExpenses(); }}
      onclose={() => { deleteModalExpense = null; }}
    />
  {/if}

  {#if confirmBatchDelete}
    <BatchDeleteModal
      selectedIds={selected}
      ondelete={() => { confirmBatchDelete = false; fetchExpenses(); }}
      onclose={() => { confirmBatchDelete = false; }}
    />
  {/if}
  {/if}
</div>
