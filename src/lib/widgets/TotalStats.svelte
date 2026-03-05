<script>
  import { getPrefersReducedMotion } from "../stores/reduced-motion.svelte.js";

  let { expenses, onnavigate = () => {} } = $props();

  let totalExpenses = $derived(
    expenses.reduce((sum, e) => sum + Math.abs(e.amount), 0)
  );

  let categoryCount = $derived(
    new Set(expenses.map((e) => e.category).filter(Boolean)).size
  );

  // Count-up animation — only on first load when data arrives
  let hasAnimated = $state(false);
  let displayTotal = $state(0);
  let displayCount = $state(0);
  let displayCategories = $state(0);

  $effect(() => {
    const targetTotal = totalExpenses;
    const targetCount = expenses.length;
    const targetCats = categoryCount;

    // Skip animation if reduced motion, already animated, or no data yet
    if (hasAnimated || getPrefersReducedMotion() || targetCount === 0) {
      displayTotal = targetTotal;
      displayCount = targetCount;
      displayCategories = targetCats;
      return;
    }

    hasAnimated = true;
    const duration = 500;
    let rafId;
    let startTime = 0;

    function tick(now) {
      if (!startTime) startTime = now;
      const elapsed = now - startTime;
      const t = Math.min(elapsed / duration, 1);
      // ease-out cubic
      const ease = 1 - Math.pow(1 - t, 3);

      displayTotal = targetTotal * ease;
      displayCount = Math.round(targetCount * ease);
      displayCategories = Math.round(targetCats * ease);

      if (t < 1) {
        rafId = requestAnimationFrame(tick);
      } else {
        displayTotal = targetTotal;
        displayCount = targetCount;
        displayCategories = targetCats;
      }
    }

    rafId = requestAnimationFrame(tick);

    return () => {
      if (rafId) cancelAnimationFrame(rafId);
    };
  });
</script>

<div class="grid grid-cols-1 md:grid-cols-3 gap-6">
  <button
    onclick={() => onnavigate("expenses")}
    class="bg-gray-900 rounded-xl p-6 border border-gray-800 text-left
           cursor-pointer hover:border-amber-500/50 hover:bg-gray-900/80 transition-all card-hover"
  >
    <div class="text-sm text-gray-400 mb-1">Total Expenses</div>
    <div class="text-3xl font-bold text-amber-400">
      {displayTotal.toFixed(2)}
    </div>
  </button>

  <button
    onclick={() => onnavigate("expenses")}
    class="bg-gray-900 rounded-xl p-6 border border-gray-800 text-left
           cursor-pointer hover:border-amber-500/50 hover:bg-gray-900/80 transition-all card-hover"
  >
    <div class="text-sm text-gray-400 mb-1">Transactions</div>
    <div class="text-3xl font-bold text-amber-400">
      {displayCount}
    </div>
  </button>

  <button
    onclick={() => onnavigate("categories")}
    class="bg-gray-900 rounded-xl p-6 border border-gray-800 text-left
           cursor-pointer hover:border-amber-500/50 hover:bg-gray-900/80 transition-all card-hover"
  >
    <div class="text-sm text-gray-400 mb-1">Categories</div>
    <div class="text-3xl font-bold text-amber-400">
      {displayCategories}
    </div>
  </button>
</div>
