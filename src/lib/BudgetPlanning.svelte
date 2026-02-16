<script>
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import MonthSelector from "./budget/MonthSelector.svelte";
  import BudgetEditor from "./budget/BudgetEditor.svelte";
  import CalendarEvents from "./budget/CalendarEvents.svelte";
  import BudgetOverview from "./budget/BudgetOverview.svelte";

  let activeTab = $state("budget");
  let loading = $state(true);

  // Current month
  const now = new Date();
  let year = $state(now.getFullYear());
  let month = $state(now.getMonth() + 1);

  // Data
  let summary = $state(null);
  let averages = $state([]);
  let allCategories = $state([]);

  async function loadData() {
    loading = true;
    try {
      const [s, avgs, cats] = await Promise.all([
        invoke("get_budget_summary", { year, month }),
        invoke("get_category_averages"),
        invoke("get_categories"),
      ]);
      summary = s;
      averages = avgs;
      allCategories = cats;
    } catch (err) {
      console.error("Failed to load budget data:", err);
    }
    loading = false;
  }

  onMount(loadData);

  function onMonthChange({ year: y, month: m }) {
    year = y;
    month = m;
    loadData();
  }

  const tabs = [
    { id: "budget", label: "Budget" },
    { id: "calendar", label: "Calendar" },
    { id: "overview", label: "Overview" },
  ];
</script>

<div>
  <div class="flex items-center justify-between mb-6">
    <h2 class="text-2xl font-bold">Budget Planning</h2>
    <MonthSelector {year} {month} onchange={onMonthChange} />
  </div>

  <!-- Tabs -->
  <div class="flex gap-1 mb-6">
    {#each tabs as tab}
      <button
        onclick={() => (activeTab = tab.id)}
        class="px-4 py-2 rounded-lg text-sm font-medium transition-colors
          {activeTab === tab.id
          ? 'bg-gray-800 text-emerald-400'
          : 'text-gray-400 hover:bg-gray-800/50 hover:text-gray-200'}"
      >
        {tab.label}
      </button>
    {/each}
  </div>

  {#if loading}
    <div class="bg-gray-900 rounded-xl p-12 border border-gray-800 text-center text-gray-500">
      Loading...
    </div>
  {:else if summary}
    {#if activeTab === "budget"}
      <BudgetEditor
        {year}
        {month}
        categories={allCategories}
        {averages}
        budgetCategories={summary.budget_categories}
        plannedExpenses={summary.planned_expenses}
        onrefresh={loadData}
      />
    {:else if activeTab === "calendar"}
      <CalendarEvents
        {year}
        {month}
        events={summary.calendar_events}
        onrefresh={loadData}
      />
    {:else if activeTab === "overview"}
      <BudgetOverview
        categories={summary.categories}
        plannedExpenses={summary.planned_expenses}
        totalBudgeted={summary.total_budgeted}
        totalSpent={summary.total_spent}
        totalPlanned={summary.total_planned}
      />
    {/if}
  {/if}
</div>
