<script>
  let {
    currentPage,
    totalPages,
    pageSize,
    showingFrom,
    showingTo,
    totalCount,
    label = "expense",
    onpagesize,
    onprev,
    onnext,
  } = $props();
</script>

<div class="flex items-center justify-between mt-4 text-sm text-gray-400">
  <span>
    Showing {showingFrom}-{showingTo} of {totalCount} {label}{totalCount !== 1 ? "s" : ""}
  </span>
  <div class="flex items-center gap-3">
    <div class="flex items-center gap-1.5">
      <span class="text-gray-500">Rows:</span>
      {#each [25, 50, 100] as size}
        <button
          onclick={() => onpagesize(size)}
          class="px-2 py-0.5 rounded text-sm transition-colors
                 {pageSize === size ? 'bg-emerald-900/40 text-emerald-400' : 'text-gray-400 hover:text-gray-200'}"
        >
          {size}
        </button>
      {/each}
    </div>
    <div class="flex items-center gap-1">
      <button
        onclick={onprev}
        disabled={currentPage <= 1}
        title="Previous page"
        aria-label="Previous page"
        class="px-2 py-1 rounded text-gray-400 hover:text-gray-200 disabled:opacity-30
               disabled:cursor-not-allowed transition-colors"
      >
        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7" />
        </svg>
      </button>
      <span class="px-2 text-gray-300">{currentPage} / {totalPages}</span>
      <button
        onclick={onnext}
        disabled={currentPage >= totalPages}
        title="Next page"
        aria-label="Next page"
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
