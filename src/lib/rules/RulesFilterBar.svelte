<script>
  let {
    searchText,
    filterCategory,
    hasActiveFilters,
    categories,
    onsearch,
    onfilterchange,
    onclear,
  } = $props();

  let debounceTimer = $state(null);

  function handleSearchInput(e) {
    const value = e.target.value;
    clearTimeout(debounceTimer);
    debounceTimer = setTimeout(() => onsearch(value), 250);
  }
</script>

<div class="flex flex-wrap items-center gap-3 mb-4">
  <div class="relative flex-1 min-w-48">
    <input
      type="text"
      placeholder="Search patterns..."
      value={searchText}
      oninput={handleSearchInput}
      class="w-full bg-gray-900 border border-gray-800 rounded-lg px-4 py-2 pl-9 text-sm
             text-gray-200 placeholder-gray-500 focus:border-emerald-500 focus:ring-1
             focus:ring-emerald-500 focus:outline-none"
    />
    <svg class="w-4 h-4 absolute left-3 top-2.5 text-gray-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
    </svg>
  </div>

  <select
    value={filterCategory}
    onchange={(e) => onfilterchange(e.target.value)}
    class="bg-gray-900 border border-gray-800 rounded-lg px-3 py-2 text-sm text-gray-200
           focus:border-emerald-500 focus:ring-1 focus:ring-emerald-500 focus:outline-none"
  >
    <option value="">All categories</option>
    {#each categories as cat}
      <option value={cat}>{cat}</option>
    {/each}
  </select>

  {#if hasActiveFilters}
    <button
      onclick={onclear}
      class="text-sm text-gray-400 hover:text-emerald-400 transition-colors"
    >
      Clear filters
    </button>
  {/if}
</div>
