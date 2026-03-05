<script>
  import { onDestroy } from "svelte";

  let {
    searchText,
    filterCategory,
    filterDateFrom,
    filterDateTo,
    filterAmountMin,
    filterAmountMax,
    hasActiveFilters,
    categories,
    onsearch,
    onfilterchange,
    onclear,
  } = $props();

  let debounceTimer = null;

  function onSearchInput(e) {
    clearTimeout(debounceTimer);
    const value = e.target.value;
    debounceTimer = setTimeout(() => {
      onsearch(value);
    }, 300);
  }

  onDestroy(() => clearTimeout(debounceTimer));
</script>

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
    aria-label="Search expenses by title"
    class="w-full bg-gray-900 border border-gray-800 rounded-lg pl-10 pr-4 py-2.5 text-sm
           text-gray-200 placeholder-gray-500 focus:border-amber-500 focus:ring-1
           focus:ring-amber-500 focus:outline-none"
  />
</div>

<!-- Filter bar -->
<div class="flex flex-wrap gap-3 mb-4 items-end">
  <div class="flex flex-col gap-1">
    <label for="filter-category" class="text-xs text-gray-500">Category</label>
    <select
      id="filter-category"
      value={filterCategory}
      onchange={(e) => onfilterchange("filterCategory", e.target.value)}
      class="bg-gray-900 border border-gray-800 rounded-lg px-3 py-2 text-sm text-gray-200
             focus:border-amber-500 focus:ring-1 focus:ring-amber-500 focus:outline-none"
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
      value={filterDateFrom}
      onchange={(e) => onfilterchange("filterDateFrom", e.target.value)}
      class="bg-gray-900 border border-gray-800 rounded-lg px-3 py-2 text-sm text-gray-200
             focus:border-amber-500 focus:ring-1 focus:ring-amber-500 focus:outline-none"
    />
  </div>
  <div class="flex flex-col gap-1">
    <label for="filter-date-to" class="text-xs text-gray-500">To</label>
    <input
      id="filter-date-to"
      type="date"
      value={filterDateTo}
      onchange={(e) => onfilterchange("filterDateTo", e.target.value)}
      class="bg-gray-900 border border-gray-800 rounded-lg px-3 py-2 text-sm text-gray-200
             focus:border-amber-500 focus:ring-1 focus:ring-amber-500 focus:outline-none"
    />
  </div>
  <div class="flex flex-col gap-1">
    <label for="filter-amount-min" class="text-xs text-gray-500">Min amount</label>
    <input
      id="filter-amount-min"
      type="number"
      step="0.01"
      value={filterAmountMin}
      onchange={(e) => onfilterchange("filterAmountMin", e.target.value)}
      placeholder="0.00"
      class="w-28 bg-gray-900 border border-gray-800 rounded-lg px-3 py-2 text-sm text-gray-200
             focus:border-amber-500 focus:ring-1 focus:ring-amber-500 focus:outline-none"
    />
  </div>
  <div class="flex flex-col gap-1">
    <label for="filter-amount-max" class="text-xs text-gray-500">Max amount</label>
    <input
      id="filter-amount-max"
      type="number"
      step="0.01"
      value={filterAmountMax}
      onchange={(e) => onfilterchange("filterAmountMax", e.target.value)}
      placeholder="0.00"
      class="w-28 bg-gray-900 border border-gray-800 rounded-lg px-3 py-2 text-sm text-gray-200
             focus:border-amber-500 focus:ring-1 focus:ring-amber-500 focus:outline-none"
    />
  </div>
  {#if hasActiveFilters}
    <button
      onclick={onclear}
      class="text-gray-400 hover:text-gray-200 text-sm px-3 py-2 transition-colors"
      aria-label="Clear all filters"
    >
      Clear filters
    </button>
  {/if}
</div>
