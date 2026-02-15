<script>
  let { expenses } = $props();

  let topTitles = $derived.by(() => {
    const counts = {};
    for (const e of expenses) {
      counts[e.title] = (counts[e.title] || 0) + 1;
    }
    return Object.entries(counts)
      .sort((a, b) => b[1] - a[1])
      .slice(0, 5);
  });
</script>

<div class="bg-gray-900 rounded-xl p-6 border border-gray-800">
  <h3 class="text-lg font-semibold mb-4">Most Frequent</h3>

  {#if topTitles.length > 0}
    <div class="space-y-3">
      {#each topTitles as [title, count]}
        <div class="flex justify-between items-center">
          <span class="text-gray-300 truncate mr-4">{title}</span>
          <span class="text-sm text-gray-400 whitespace-nowrap">{count}x</span>
        </div>
      {/each}
    </div>
  {:else}
    <p class="text-sm text-gray-500">No data yet.</p>
  {/if}
</div>
