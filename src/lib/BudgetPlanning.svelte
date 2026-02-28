<script>
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import BudgetCreator from "./budget/BudgetCreator.svelte";
  import BudgetOverview from "./budget/BudgetOverview.svelte";

  let activeTab = $state("overview");
  let loading = $state(true);

  // Data
  let allBudgets = $state([]);
  let currentIndex = $state(-1);
  let currentSummary = $state(null);
  let averages = $state([]);
  let allCategories = $state([]);

  async function loadData() {
    loading = true;
    try {
      const [budgets, avgs, cats] = await Promise.all([
        invoke("list_budgets"),
        invoke("get_category_averages"),
        invoke("get_categories"),
      ]);
      allBudgets = budgets;
      averages = avgs;
      allCategories = cats;

      if (allBudgets.length > 0) {
        // Find active budget (date range contains today)
        const today = new Date().toISOString().slice(0, 10);
        const activeIdx = allBudgets.findIndex(
          (b) => b.start_date <= today && b.end_date >= today,
        );
        currentIndex = activeIdx >= 0 ? activeIdx : allBudgets.length - 1;
        await loadSummary();
      } else {
        currentIndex = -1;
        currentSummary = null;
      }
    } catch (err) {
      console.error("Failed to load budget data:", err);
    }
    loading = false;
  }

  async function loadSummary() {
    if (currentIndex < 0 || currentIndex >= allBudgets.length) {
      currentSummary = null;
      return;
    }
    try {
      const budgetId = allBudgets[currentIndex].id;
      currentSummary = await invoke("get_budget_summary", { budgetId });
    } catch (err) {
      console.error("Failed to load budget summary:", err);
      currentSummary = null;
    }
  }

  async function goPrev() {
    if (currentIndex > 0) {
      currentIndex--;
      await loadSummary();
    }
  }

  async function goNext() {
    if (currentIndex < allBudgets.length - 1) {
      currentIndex++;
      await loadSummary();
    }
  }

  onMount(loadData);

  function onBudgetCreated() {
    activeTab = "overview";
    loadData();
  }

  async function onRefresh() {
    await loadData();
  }

  let tabs = $derived([
    { id: "overview", label: "Overview" },
    { id: "create", label: "Create +" },
  ]);
</script>

<div>
  <div class="flex items-center justify-between mb-6">
    <h2 class="text-2xl font-bold">Budget Planning</h2>
  </div>

  <!-- Tabs -->
  <div class="flex gap-1 mb-6">
    {#each tabs as tab}
      <button
        onclick={() => (activeTab = tab.id)}
        class="px-4 py-2 rounded-lg text-sm font-medium transition-colors
          {activeTab === tab.id
            ? 'bg-gray-800 text-amber-400'
            : 'text-gray-400 hover:bg-gray-800/50 hover:text-gray-200'}"
      >
        {tab.label}
      </button>
    {/each}
  </div>

  {#if loading}
    <div
      class="bg-gray-900 rounded-xl p-12 border border-gray-800 text-center text-gray-500"
    >
      Loading...
    </div>
  {:else if activeTab === "overview"}
    {#if currentSummary}
      <!-- Budget navigation -->
      {#if allBudgets.length > 1}
        <div class="flex items-center justify-center gap-4 mb-4">
          <button
            onclick={goPrev}
            disabled={currentIndex <= 0}
            class="px-3 py-1.5 rounded-lg text-sm font-medium transition-colors
              {currentIndex <= 0
              ? 'text-gray-600 cursor-not-allowed'
              : 'text-gray-400 hover:bg-gray-800 hover:text-gray-200'}"
          >
            &larr; Prev
          </button>
          <span class="text-sm text-gray-400 font-mono">
            {currentIndex + 1} / {allBudgets.length}
          </span>
          <button
            onclick={goNext}
            disabled={currentIndex >= allBudgets.length - 1}
            class="px-3 py-1.5 rounded-lg text-sm font-medium transition-colors
              {currentIndex >= allBudgets.length - 1
              ? 'text-gray-600 cursor-not-allowed'
              : 'text-gray-400 hover:bg-gray-800 hover:text-gray-200'}"
          >
            Next &rarr;
          </button>
        </div>
      {/if}

      <BudgetOverview
        budgetId={currentSummary.budget_id}
        startDate={currentSummary.start_date}
        endDate={currentSummary.end_date}
        categories={currentSummary.categories}
        budgetCategories={currentSummary.budget_categories}
        totalBudgeted={currentSummary.total_budgeted}
        totalSpent={currentSummary.total_spent}
        {allCategories}
        onrefresh={onRefresh}
      />
    {:else}
      <div
        class="bg-gray-900 rounded-xl p-12 border border-gray-800 text-center"
      >
        <p class="text-gray-400 mb-2">No budgets yet.</p>
        <p class="text-sm text-gray-600">
          Go to the
          <button
            onclick={() => (activeTab = "create")}
            class="text-amber-400 hover:text-amber-300 underline"
            >Create +</button
          >
          tab to set up a budget.
        </p>
      </div>
    {/if}
  {:else if activeTab === "create"}
    <BudgetCreator
      {allCategories}
      {averages}
      oncreated={onBudgetCreated}
    />
  {/if}
</div>
