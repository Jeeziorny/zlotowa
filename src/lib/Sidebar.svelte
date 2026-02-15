<script>
  let { currentPage, onnavigate } = $props();

  const items = [
    { id: "dashboard", label: "Dashboard", icon: "⊞" },
    { id: "add", label: "Add Expense", icon: "+" },
    { id: "bulk", label: "Bulk Upload", icon: "↑" },
    { id: "expenses", label: "Expenses", icon: "☰" },
  ];
</script>

<aside class="w-56 bg-gray-900 border-r border-gray-800 flex flex-col">
  <div class="p-6">
    <h1 class="text-xl font-bold text-emerald-400">4ccountant</h1>
  </div>

  <nav class="flex-1 px-3">
    {#each items as item}
      <button
        class="w-full text-left px-4 py-3 rounded-lg mb-1 flex items-center gap-3 transition-colors
          {currentPage === item.id
          ? 'bg-gray-800 text-emerald-400'
          : 'text-gray-400 hover:bg-gray-800/50 hover:text-gray-200'}"
        onclick={() => onnavigate(item.id)}
      >
        <span class="text-lg">{item.icon}</span>
        <span>{item.label}</span>
      </button>
    {/each}
  </nav>

  <div class="p-4 border-t border-gray-800 text-xs text-gray-600">
    v0.1.0
  </div>
</aside>
