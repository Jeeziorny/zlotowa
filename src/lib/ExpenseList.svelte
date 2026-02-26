<script>
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import ExportPanel from "./expense-list/ExportPanel.svelte";
  import DeleteConfirmModal from "./expense-list/DeleteConfirmModal.svelte";
  import BatchDeleteModal from "./expense-list/BatchDeleteModal.svelte";

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

  // Export
  let showExportModal = $state(false);

  // Inline edit
  let editingId = $state(null);
  let editDisplayTitle = $state("");
  let editAmount = $state("");
  let editDate = $state("");
  let editCategory = $state("");
  let editError = $state("");
  let saving = $state(false);

  // Delete
  let deleteModalExpense = $state(null);

  // Batch select/delete
  let selected = $state(new Set());
  let confirmBatchDelete = $state(false);

  let allSelected = $derived(expenses.length > 0 && selected.size === expenses.length);
  let someSelected = $derived(selected.size > 0);

  // Categories for filters & edit dropdown
  let categories = $state([]);

  // Debounce timer
  let debounceTimer = null;

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

  function onSearchInput(e) {
    searchText = e.target.value;
    clearTimeout(debounceTimer);
    debounceTimer = setTimeout(() => {
      currentPage = 1;
      fetchExpenses();
    }, 300);
  }

  function onFilterChange() {
    currentPage = 1;
    fetchExpenses();
  }

  function clearFilters() {
    searchText = "";
    filterCategory = "";
    filterDateFrom = "";
    filterDateTo = "";
    filterAmountMin = "";
    filterAmountMax = "";
    currentPage = 1;
    fetchExpenses();
  }

  function changePageSize(newSize) {
    pageSize = newSize;
    currentPage = 1;
    fetchExpenses();
  }

  function prevPage() {
    if (currentPage > 1) {
      currentPage--;
      fetchExpenses();
    }
  }

  function nextPage() {
    if (currentPage < totalPages) {
      currentPage++;
      fetchExpenses();
    }
  }

  // ── Inline Edit ──

  function startEdit(expense) {
    editingId = expense.id;
    editDisplayTitle = expense.display_title || expense.title;
    editAmount = String(expense.amount);
    editDate = expense.date;
    editCategory = expense.category || "";
    editError = "";
  }

  function cancelEdit() {
    editingId = null;
    editError = "";
  }

  async function saveEdit() {
    const amount = parseFloat(editAmount);
    if (isNaN(amount)) {
      editError = "Amount must be a valid number";
      return;
    }
    if (!editDisplayTitle.trim()) {
      editError = "Title cannot be empty";
      return;
    }
    if (!editDate) {
      editError = "Date is required";
      return;
    }
    saving = true;
    editError = "";
    try {
      const editedExpense = expenses.find(e => e.id === editingId);
      await invoke("update_expense", {
        id: editingId,
        input: {
          title: editedExpense.title,
          display_title: editDisplayTitle.trim(),
          amount,
          date: editDate,
          category: editCategory.trim() || null,
          rule_pattern: null,
        },
      });
      editingId = null;
      await fetchExpenses();
    } catch (err) {
      editError = `Save failed: ${err}`;
    }
    saving = false;
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
        onclick={() => { showExportModal = !showExportModal; }}
        class="bg-gray-800 hover:bg-gray-700 text-gray-200 px-4 py-2 rounded-lg
               text-sm font-medium transition-colors border border-gray-700"
      >
        Export CSV
      </button>
    </div>
  </div>

  <!-- Search bar -->
  <div class="relative mb-3">
    <svg class="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-gray-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
    </svg>
    <input
      type="text"
      value={searchText}
      oninput={onSearchInput}
      placeholder="Search by title..."
      class="w-full bg-gray-900 border border-gray-800 rounded-lg pl-10 pr-4 py-2.5 text-sm
             text-gray-200 placeholder-gray-500 focus:border-emerald-500 focus:ring-1
             focus:ring-emerald-500 focus:outline-none"
    />
  </div>

  <!-- Filter bar -->
  <div class="flex flex-wrap gap-3 mb-4 items-end">
    <div class="flex flex-col gap-1">
      <label for="filter-category" class="text-xs text-gray-500">Category</label>
      <select
        id="filter-category"
        bind:value={filterCategory}
        onchange={onFilterChange}
        class="bg-gray-900 border border-gray-800 rounded-lg px-3 py-2 text-sm text-gray-200
               focus:border-emerald-500 focus:ring-1 focus:ring-emerald-500 focus:outline-none"
      >
        <option value="">All categories</option>
        {#each categories as cat}
          <option value={cat}>{cat}</option>
        {/each}
        <option value="uncategorized">Uncategorized</option>
      </select>
    </div>
    <div class="flex flex-col gap-1">
      <label for="filter-date-from" class="text-xs text-gray-500">From</label>
      <input
        id="filter-date-from"
        type="date"
        bind:value={filterDateFrom}
        onchange={onFilterChange}
        class="bg-gray-900 border border-gray-800 rounded-lg px-3 py-2 text-sm text-gray-200
               focus:border-emerald-500 focus:ring-1 focus:ring-emerald-500 focus:outline-none"
      />
    </div>
    <div class="flex flex-col gap-1">
      <label for="filter-date-to" class="text-xs text-gray-500">To</label>
      <input
        id="filter-date-to"
        type="date"
        bind:value={filterDateTo}
        onchange={onFilterChange}
        class="bg-gray-900 border border-gray-800 rounded-lg px-3 py-2 text-sm text-gray-200
               focus:border-emerald-500 focus:ring-1 focus:ring-emerald-500 focus:outline-none"
      />
    </div>
    <div class="flex flex-col gap-1">
      <label for="filter-amount-min" class="text-xs text-gray-500">Min amount</label>
      <input
        id="filter-amount-min"
        type="number"
        step="0.01"
        bind:value={filterAmountMin}
        onchange={onFilterChange}
        placeholder="0.00"
        class="w-28 bg-gray-900 border border-gray-800 rounded-lg px-3 py-2 text-sm text-gray-200
               focus:border-emerald-500 focus:ring-1 focus:ring-emerald-500 focus:outline-none"
      />
    </div>
    <div class="flex flex-col gap-1">
      <label for="filter-amount-max" class="text-xs text-gray-500">Max amount</label>
      <input
        id="filter-amount-max"
        type="number"
        step="0.01"
        bind:value={filterAmountMax}
        onchange={onFilterChange}
        placeholder="0.00"
        class="w-28 bg-gray-900 border border-gray-800 rounded-lg px-3 py-2 text-sm text-gray-200
               focus:border-emerald-500 focus:ring-1 focus:ring-emerald-500 focus:outline-none"
      />
    </div>
    {#if hasActiveFilters}
      <button
        onclick={clearFilters}
        class="text-gray-400 hover:text-gray-200 text-sm px-3 py-2 transition-colors"
      >
        Clear filters
      </button>
    {/if}
  </div>

  <ExportPanel bind:show={showExportModal} onclose={() => showExportModal = false} />

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
    <div class="bg-gray-900 rounded-xl border border-gray-800 overflow-hidden {loading ? 'opacity-60' : ''}">
      <table class="w-full">
        <thead>
          <tr class="border-b border-gray-800 text-sm text-gray-400">
            <th class="px-4 py-3 w-10">
              <input
                type="checkbox"
                checked={allSelected}
                onchange={toggleSelectAll}
                class="rounded bg-gray-800 border-gray-700 text-emerald-500 focus:ring-emerald-500"
              />
            </th>
            <th class="text-left px-4 py-3">Date</th>
            <th class="text-left px-4 py-3">Title</th>
            <th class="text-right px-4 py-3">Amount</th>
            <th class="text-left px-4 py-3">Category</th>
            <th class="px-4 py-3 w-24"></th>
          </tr>
        </thead>
        <tbody>
          {#each expenses as expense (expense.id)}
            {#if editingId === expense.id}
              <!-- Edit mode row -->
              <tr class="border-b border-gray-800/50 bg-gray-800/20">
                <td class="px-4 py-2"></td>
                <td class="px-4 py-2">
                  <input
                    type="date"
                    bind:value={editDate}
                    class="w-full bg-gray-800 border border-gray-700 rounded px-2 py-1 text-sm
                           text-gray-200 focus:border-emerald-500 focus:ring-1 focus:ring-emerald-500 focus:outline-none"
                  />
                </td>
                <td class="px-4 py-2">
                  <input
                    type="text"
                    bind:value={editDisplayTitle}
                    class="w-full bg-gray-800 border border-gray-700 rounded px-2 py-1 text-sm
                           text-gray-200 focus:border-emerald-500 focus:ring-1 focus:ring-emerald-500 focus:outline-none"
                  />
                  {#if expense.display_title}
                    <div class="text-xs text-gray-600 mt-0.5 truncate" title={expense.title}>
                      Raw: {expense.title}
                    </div>
                  {/if}
                </td>
                <td class="px-4 py-2">
                  <input
                    type="number"
                    step="0.01"
                    bind:value={editAmount}
                    class="w-full bg-gray-800 border border-gray-700 rounded px-2 py-1 text-sm text-right
                           font-mono text-gray-200 focus:border-emerald-500 focus:ring-1 focus:ring-emerald-500 focus:outline-none"
                  />
                </td>
                <td class="px-4 py-2">
                  <input
                    type="text"
                    bind:value={editCategory}
                    list="edit-categories"
                    class="w-full bg-gray-800 border border-gray-700 rounded px-2 py-1 text-sm
                           text-gray-200 focus:border-emerald-500 focus:ring-1 focus:ring-emerald-500 focus:outline-none"
                  />
                  <datalist id="edit-categories">
                    {#each categories as cat}
                      <option value={cat}></option>
                    {/each}
                  </datalist>
                </td>
                <td class="px-4 py-2">
                  <div class="flex gap-1">
                    <button
                      onclick={saveEdit}
                      disabled={saving}
                      class="text-emerald-400 hover:text-emerald-300 disabled:opacity-50 p-1 transition-colors"
                      title="Save"
                    >
                      <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
                      </svg>
                    </button>
                    <button
                      onclick={cancelEdit}
                      class="text-gray-400 hover:text-gray-300 p-1 transition-colors"
                      title="Cancel"
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
              <!-- Normal row -->
              <tr class="border-b border-gray-800/50 hover:bg-gray-800/30 group">
                <td class="px-4 py-3">
                  <input
                    type="checkbox"
                    checked={selected.has(expense.id)}
                    onchange={() => toggleSelect(expense.id)}
                    class="rounded bg-gray-800 border-gray-700 text-emerald-500 focus:ring-emerald-500"
                  />
                </td>
                <td class="px-4 py-3 text-sm text-gray-400">{expense.date}</td>
                <td class="px-4 py-3" title={expense.display_title ? expense.title : ''}>{expense.display_title || expense.title}</td>
                <td class="px-4 py-3 text-right font-mono">{expense.amount.toFixed(2)}</td>
                <td class="px-4 py-3">
                  {#if expense.category}
                    <span class="bg-emerald-900/30 text-emerald-400 px-2 py-0.5 rounded text-sm">
                      {expense.category}
                    </span>
                  {:else}
                    <span class="text-gray-600 text-sm">-</span>
                  {/if}
                </td>
                <td class="px-4 py-3">
                  <div class="flex gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
                    <button
                      onclick={() => startEdit(expense)}
                      class="text-gray-400 hover:text-emerald-400 p-1 transition-colors"
                      title="Edit"
                    >
                      <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                          d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z" />
                      </svg>
                    </button>
                    <button
                      onclick={() => deleteModalExpense = expense}
                      class="text-gray-400 hover:text-red-400 p-1 transition-colors"
                      title="Delete"
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

    <!-- Pagination controls -->
    <div class="flex items-center justify-between mt-4 text-sm text-gray-400">
      <span>
        Showing {showingFrom}-{showingTo} of {totalCount} expense{totalCount !== 1 ? "s" : ""}
      </span>
      <div class="flex items-center gap-3">
        <div class="flex items-center gap-1.5">
          <span class="text-gray-500">Rows:</span>
          {#each [25, 50, 100] as size}
            <button
              onclick={() => changePageSize(size)}
              class="px-2 py-0.5 rounded text-sm transition-colors
                     {pageSize === size ? 'bg-emerald-900/40 text-emerald-400' : 'text-gray-400 hover:text-gray-200'}"
            >
              {size}
            </button>
          {/each}
        </div>
        <div class="flex items-center gap-1">
          <button
            onclick={prevPage}
            disabled={currentPage <= 1}
            title="Previous page"
            class="px-2 py-1 rounded text-gray-400 hover:text-gray-200 disabled:opacity-30
                   disabled:cursor-not-allowed transition-colors"
          >
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7" />
            </svg>
          </button>
          <span class="px-2 text-gray-300">{currentPage} / {totalPages}</span>
          <button
            onclick={nextPage}
            disabled={currentPage >= totalPages}
            title="Next page"
            class="px-2 py-1 rounded text-gray-400 hover:text-gray-200 disabled:opacity-30
                   disabled:cursor-not-allowed transition-colors"
          >
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7" />
            </svg>
          </button>
        </div>
      </div>
    </div>
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
</div>
