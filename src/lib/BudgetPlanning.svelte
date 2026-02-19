<script>
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import BudgetCreator from "./budget/BudgetCreator.svelte";
  import CalendarEvents from "./budget/CalendarEvents.svelte";
  import BudgetOverview from "./budget/BudgetOverview.svelte";

  let activeTab = $state("overview");
  let loading = $state(true);

  // Data
  let activeBudget = $state(null);
  let averages = $state([]);
  let allCategories = $state([]);

  async function loadData() {
    loading = true;
    try {
      const [s, avgs, cats] = await Promise.all([
        invoke("get_active_budget_summary"),
        invoke("get_category_averages"),
        invoke("get_categories"),
      ]);
      activeBudget = s;
      averages = avgs;
      allCategories = cats;
    } catch (err) {
      console.error("Failed to load budget data:", err);
    }
    loading = false;
  }

  onMount(loadData);

  function onBudgetCreated() {
    activeTab = "overview";
    loadData();
  }

  let tabs = $derived([
    { id: "overview", label: "Overview" },
    {
      id: "create",
      label: "Create +",
      disabled: !!activeBudget,
    },
    { id: "calendar", label: "Calendar", disabled: !activeBudget },
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
        onclick={() => {
          if (!tab.disabled) activeTab = tab.id;
        }}
        disabled={tab.disabled}
        class="px-4 py-2 rounded-lg text-sm font-medium transition-colors
          {tab.disabled
          ? 'text-gray-600 cursor-not-allowed'
          : activeTab === tab.id
            ? 'bg-gray-800 text-emerald-400'
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
    {#if activeBudget}
      <BudgetOverview
        budgetId={activeBudget.budget_id}
        startDate={activeBudget.start_date}
        endDate={activeBudget.end_date}
        categories={activeBudget.categories}
        budgetCategories={activeBudget.budget_categories}
        plannedExpenses={activeBudget.planned_expenses}
        calendarEvents={activeBudget.calendar_events}
        totalBudgeted={activeBudget.total_budgeted}
        totalSpent={activeBudget.total_spent}
        totalPlanned={activeBudget.total_planned}
        totalCalendar={activeBudget.total_calendar}
        {allCategories}
        onrefresh={loadData}
      />
    {:else}
      <div
        class="bg-gray-900 rounded-xl p-12 border border-gray-800 text-center"
      >
        <p class="text-gray-400 mb-2">No active budget.</p>
        <p class="text-sm text-gray-600">
          Go to the
          <button
            onclick={() => (activeTab = "create")}
            class="text-emerald-400 hover:text-emerald-300 underline"
            >Create +</button
          >
          tab to set up a budget for the current period.
        </p>
      </div>
    {/if}
  {:else if activeTab === "create"}
    {#if activeBudget}
      <div
        class="bg-gray-900 rounded-xl p-12 border border-gray-800 text-center"
      >
        <p class="text-gray-400">
          An active budget already exists ({activeBudget.start_date} —
          {activeBudget.end_date}).
        </p>
        <p class="text-sm text-gray-600 mt-2">
          Delete the current budget from the Overview tab before creating a new
          one.
        </p>
      </div>
    {:else}
      <BudgetCreator
        {allCategories}
        {averages}
        oncreated={onBudgetCreated}
      />
    {/if}
  {:else if activeTab === "calendar"}
    {#if activeBudget}
      <CalendarEvents
        budgetId={activeBudget.budget_id}
        startDate={activeBudget.start_date}
        endDate={activeBudget.end_date}
        events={activeBudget.calendar_events}
        onrefresh={loadData}
      />
    {:else}
      <div
        class="bg-gray-900 rounded-xl p-12 border border-gray-800 text-center"
      >
        <p class="text-gray-400">Create a budget first to import calendar events.</p>
      </div>
    {/if}
  {/if}
</div>
