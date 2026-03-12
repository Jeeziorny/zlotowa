<script>
  let { steps, currentStep } = $props();

  let currentIdx = $derived(steps.findIndex(s => s.id === currentStep));
</script>

<div class="flex items-center gap-2 mb-6 text-sm">
  {#each steps as s, i}
    {#if i > 0}
      <div class="h-px flex-1 max-w-8 {i <= currentIdx ? 'bg-amber-500' : 'bg-gray-700'}"></div>
    {/if}
    <span
      class="px-3 py-1 rounded-full text-xs font-medium
        {currentStep === s.id
        ? 'bg-amber-500 text-gray-950'
        : i < currentIdx
          ? 'bg-amber-900/50 text-amber-400'
          : 'bg-gray-800 text-gray-500'}"
    >
      {s.label}
    </span>
  {/each}
</div>
