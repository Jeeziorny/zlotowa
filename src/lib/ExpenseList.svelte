<script>
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { save } from "@tauri-apps/plugin-dialog";

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
  let exportDate = $state(true);
  let exportTitle = $state(true);
  let exportAmount = $state(true);
  let exportCategory = $state(true);
  let exportSource = $state(false);
  let exportError = $state("");
  let exportSuccess = $state("");
  let exporting = $state(false);

  // Inline edit
  let editingId = $state(null);
  let editTitle = $state("");
  let editAmount = $state("");
  let editDate = $state("");
  let editCategory = $state("");
  let editError = $state("");
  let saving = $state(false);

  // Delete
  let deleting = $state(false);

  // Batch select/delete
  let selected = $state(new Set());
  let confirmBatchDelete = $state(false);
  let batchDeleting = $state(false);

  let allSelected = $derived(expenses.length > 0 && selected.size === expenses.length);
  let someSelected = $derived(selected.size > 0);

  // Categories for filters & edit dropdown
  let categories = $state([]);

  // Debounce timer
  let debounceTimer = null;

  onMount(async () => {
    try {
      categories = await invoke("get_categories");
    } catch (_) {}
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
      // Clear selections when page changes
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

  // Delete modal
  let deleteModalExpense = $state(null);

  // ── Export ──

  async function doExport() {
    exporting = true;
    exportError = "";
    exportSuccess = "";
    try {
      const path = await save({
        defaultPath: `4ccountant-export-${new Date().toISOString().split("T")[0]}.csv`,
        filters: [{ name: "CSV", extensions: ["csv"] }],
      });
      if (!path) { exporting = false; return; }
      await invoke("export_expenses", {
        columns: {
          date: exportDate,
          title: exportTitle,
          amount: exportAmount,
          category: exportCategory,
          classification_source: exportSource,
        },
        path,
      });
      const filename = path.split("/").pop() || path.split("\\").pop() || path;
      exportSuccess = `Exported to ${filename}`;
      setTimeout(() => { showExportModal = false; exportSuccess = ""; }, 2000);
    } catch (err) {
      exportError = `Export failed: ${err}`;
    }
    exporting = false;
  }

  // ── Inline Edit ──

  function startEdit(expense) {
    editingId = expense.id;
    editTitle = expense.title;
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
    if (!editTitle.trim()) {
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
      await invoke("update_expense", {
        id: editingId,
        input: {
          title: editTitle.trim(),
          amount,
          date: editDate,
          category: editCategory.trim() || null,
          rule_pattern: null,
        },
      });
      // Re-fetch to reflect changes (category/title might affect filters)
      editingId = null;
      await fetchExpenses();
    } catch (err) {
      editError = `Save failed: ${err}`;
    }
    saving = false;
  }

  // ── Single Delete ──

  async function doDelete(id) {
    deleting = true;
    try {
      await invoke("delete_expense", { id });
      deleteModalExpense = null;
      await fetchExpenses();
    } catch (err) {
      console.error("Delete failed:", err);
    }
    deleting = false;
  }

  // ── Batch Delete ──

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

  async function doBatchDelete() {
    batchDeleting = true;
    try {
      const ids = [...selected];
      await invoke("delete_expenses", { ids });
      confirmBatchDelete = false;
      await fetchExpenses();
    } catch (err) {
      console.error("Batch delete failed:", err);
    }
    batchDeleting = false;
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
        onclick={() => { showExportModal = !showExportModal; exportError = ""; exportSuccess = ""; }}
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


  {#if showExportModal}
    <div class="bg-gray-900 rounded-xl p-6 border border-gray-800 mb-6">
      <h3 class="text-lg font-semibold mb-3">Export Settings</h3>
      <div class="space-y-2 mb-4">
        <label class="flex items-center gap-2 text-sm text-gray-300 cursor-pointer">
          <input type="checkbox" bind:checked={exportDate}
                 class="rounded bg-gray-800 border-gray-700 text-emerald-500 focus:ring-emerald-500" />
          Date
        </label>
        <label class="flex items-center gap-2 text-sm text-gray-300 cursor-pointer">
          <input type="checkbox" bind:checked={exportTitle}
                 class="rounded bg-gray-800 border-gray-700 text-emerald-500 focus:ring-emerald-500" />
          Title
        </label>
        <label class="flex items-center gap-2 text-sm text-gray-300 cursor-pointer">
          <input type="checkbox" bind:checked={exportAmount}
                 class="rounded bg-gray-800 border-gray-700 text-emerald-500 focus:ring-emerald-500" />
          Amount
        </label>
        <label class="flex items-center gap-2 text-sm text-gray-300 cursor-pointer">
          <input type="checkbox" bind:checked={exportCategory}
                 class="rounded bg-gray-800 border-gray-700 text-emerald-500 focus:ring-emerald-500" />
          Category
        </label>
        <label class="flex items-center gap-2 text-sm text-gray-300 cursor-pointer">
          <input type="checkbox" bind:checked={exportSource}
                 class="rounded bg-gray-800 border-gray-700 text-emerald-500 focus:ring-emerald-500" />
          Classification Source
        </label>
      </div>
      <div class="flex gap-3">
        <button
          onclick={doExport}
          disabled={exporting}
          class="bg-emerald-600 hover:bg-emerald-500 disabled:opacity-50 text-white
                 px-4 py-2 rounded-lg text-sm font-medium transition-colors"
        >
          {exporting ? "Exporting..." : "Download CSV"}
        </button>
        <button
          onclick={() => showExportModal = false}
          class="bg-gray-800 hover:bg-gray-700 text-gray-300 px-4 py-2 rounded-lg
                 text-sm transition-colors"
        >
          Cancel
        </button>
      </div>
      {#if exportError}
        <div class="mt-3 text-sm bg-red-900/50 text-red-400 px-4 py-2 rounded-lg">
          {exportError}
        </div>
      {/if}
      {#if exportSuccess}
        <div class="mt-3 text-sm bg-emerald-900/50 text-emerald-400 px-4 py-2 rounded-lg">
          {exportSuccess}
        </div>
      {/if}
    </div>
  {/if}

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
                    bind:value={editTitle}
                    class="w-full bg-gray-800 border border-gray-700 rounded px-2 py-1 text-sm
                           text-gray-200 focus:border-emerald-500 focus:ring-1 focus:ring-emerald-500 focus:outline-none"
                  />
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
                <td class="px-4 py-3">{expense.title}</td>
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

  <!-- Single delete confirmation modal -->
  {#if deleteModalExpense}
    <div class="fixed inset-0 bg-black/60 flex items-center justify-center z-50"
         onclick={() => { if (!deleting) deleteModalExpense = null; }}>
      <div class="bg-gray-900 border border-gray-800 rounded-xl p-6 max-w-sm w-full mx-4 shadow-xl"
           onclick={(e) => e.stopPropagation()}>
        <h3 class="text-lg font-semibold text-gray-100 mb-2">Delete expense?</h3>
        <p class="text-sm text-gray-400 mb-1">This cannot be undone.</p>
        <p class="text-sm text-gray-300 mb-5 break-words">
          "{deleteModalExpense.title}" &mdash; {deleteModalExpense.amount.toFixed(2)}
        </p>
        <div class="flex gap-3 justify-end">
          <button
            onclick={() => deleteModalExpense = null}
            disabled={deleting}
            class="bg-gray-800 hover:bg-gray-700 text-gray-300 px-4 py-2 rounded-lg
                   text-sm transition-colors disabled:opacity-50"
          >
            Cancel
          </button>
          <button
            onclick={() => doDelete(deleteModalExpense.id)}
            disabled={deleting}
            class="bg-red-600 hover:bg-red-500 disabled:opacity-50 text-white px-4 py-2
                   rounded-lg text-sm font-medium transition-colors"
          >
            {deleting ? "Deleting..." : "Delete"}
          </button>
        </div>
      </div>
    </div>
  {/if}

  <!-- Batch delete confirmation modal -->
  {#if confirmBatchDelete}
    <div class="fixed inset-0 bg-black/60 flex items-center justify-center z-50"
         onclick={() => { if (!batchDeleting) confirmBatchDelete = false; }}>
      <div class="bg-gray-900 border border-gray-800 rounded-xl p-6 max-w-sm w-full mx-4 shadow-xl"
           onclick={(e) => e.stopPropagation()}>
        <h3 class="text-lg font-semibold text-gray-100 mb-2">Delete {selected.size} expense{selected.size > 1 ? "s" : ""}?</h3>
        <p class="text-sm text-gray-400 mb-5">This cannot be undone.</p>
        <div class="flex gap-3 justify-end">
          <button
            onclick={() => confirmBatchDelete = false}
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
  {/if}
</div>
