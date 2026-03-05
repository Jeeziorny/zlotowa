<script>
  let { variant = "card", count = 1 } = $props();

  let items = $derived(Array.from({ length: count }, (_, i) => i));
</script>

{#each items as i}
  {#if variant === "card"}
    <div class="bg-gray-900 rounded-xl p-6 border border-gray-800">
      <div class="animate-skeleton h-4 w-24 bg-gray-800 rounded mb-3"></div>
      <div class="animate-skeleton h-8 w-32 bg-gray-800 rounded"></div>
    </div>
  {:else if variant === "row"}
    <div class="flex items-center gap-4 px-4 py-3 border-b border-gray-800/50">
      <div class="animate-skeleton h-4 w-4 bg-gray-800 rounded"></div>
      <div class="animate-skeleton h-4 flex-1 bg-gray-800 rounded"></div>
      <div class="animate-skeleton h-4 w-16 bg-gray-800 rounded"></div>
      <div class="animate-skeleton h-4 w-12 bg-gray-800 rounded"></div>
    </div>
  {:else if variant === "text"}
    <div class="animate-skeleton h-4 bg-gray-800 rounded {i === count - 1 ? 'w-2/3' : 'w-full'} mb-2"></div>
  {/if}
{/each}
