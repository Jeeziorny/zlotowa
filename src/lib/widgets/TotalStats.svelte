<script>
  let { expenses } = $props();

  let totalExpenses = $derived(
    expenses.reduce((sum, e) => sum + Math.abs(e.amount), 0)
  );

  let categoryCount = $derived(
    new Set(expenses.map((e) => e.category).filter(Boolean)).size
  );
</script>

<div class="grid grid-cols-1 md:grid-cols-3 gap-6">
  <div class="bg-gray-900 rounded-xl p-6 border border-gray-800">
    <div class="text-sm text-gray-400 mb-1">Total Expenses</div>
    <div class="text-3xl font-bold text-emerald-400">
      {totalExpenses.toFixed(2)}
    </div>
  </div>

  <div class="bg-gray-900 rounded-xl p-6 border border-gray-800">
    <div class="text-sm text-gray-400 mb-1">Transactions</div>
    <div class="text-3xl font-bold text-emerald-400">
      {expenses.length}
    </div>
  </div>

  <div class="bg-gray-900 rounded-xl p-6 border border-gray-800">
    <div class="text-sm text-gray-400 mb-1">Categories</div>
    <div class="text-3xl font-bold text-emerald-400">
      {categoryCount}
    </div>
  </div>
</div>
