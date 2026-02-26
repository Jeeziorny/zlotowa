<script>
  let { savedCount, onreset } = $props();
</script>

<div class="max-w-lg bg-gray-900 rounded-xl p-12 border border-gray-800 text-center">
  <div class="text-4xl mb-4 text-emerald-400">
    {savedCount}
  </div>
  <p class="text-lg font-semibold mb-2">Expenses saved</p>
  <p class="text-sm text-gray-400 mb-6">
    Classification rules have been updated for future imports.
  </p>
  <button
    onclick={onreset}
    class="px-6 bg-emerald-600 hover:bg-emerald-500 text-white font-medium
           py-2.5 rounded-lg transition-colors"
  >
    Upload More
  </button>
</div>
