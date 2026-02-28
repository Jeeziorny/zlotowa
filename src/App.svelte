<script>
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import Sidebar from "./lib/Sidebar.svelte";
  import Dashboard from "./lib/Dashboard.svelte";
  import ExpenseList from "./lib/ExpenseList.svelte";
  import Settings from "./lib/Settings.svelte";
  import Categories from "./lib/Categories.svelte";
  import BudgetPlanning from "./lib/BudgetPlanning.svelte";
  import Rules from "./lib/Rules.svelte";
  import ConfirmModal from "./lib/ConfirmModal.svelte";

  let currentPage = $state("dashboard");
  let expensesBulkDirty = $state(false);
  let pendingNav = $state(null);
  let showRules = $state(false);

  onMount(async () => {
    try {
      const val = await invoke("get_config", { key: "show_rules_tab" });
      showRules = val === "true";
    } catch {}
  });

  function handleNavigate(page) {
    if (page === currentPage) return;
    if (currentPage === "expenses" && expensesBulkDirty) {
      pendingNav = page;
    } else {
      currentPage = page;
    }
  }

  function confirmNavigation() {
    expensesBulkDirty = false;
    currentPage = pendingNav;
    pendingNav = null;
  }

  function handleRulesVisibilityChange(visible) {
    showRules = visible;
    if (!visible && currentPage === "rules") {
      currentPage = "dashboard";
    }
  }
</script>

<div class="flex h-screen bg-gray-950 text-gray-100">
  <Sidebar {currentPage} {showRules} onnavigate={handleNavigate} />

  <main class="flex-1 overflow-y-auto p-8">
    {#if currentPage === "dashboard"}
      <Dashboard onnavigate={handleNavigate} />
    {:else if currentPage === "expenses"}
      <ExpenseList onbulkdirtychange={(dirty) => { expensesBulkDirty = dirty; }} />
    {:else if currentPage === "categories"}
      <Categories />
    {:else if currentPage === "budget"}
      <BudgetPlanning />
    {:else if currentPage === "rules"}
      <Rules />
    {:else if currentPage === "settings"}
      <Settings onrulesvisibilitychange={handleRulesVisibilityChange} />
    {/if}
  </main>

  {#if pendingNav}
    <ConfirmModal
      title="Leave bulk upload?"
      confirmLabel="Leave"
      onconfirm={async () => { confirmNavigation(); }}
      onclose={() => { pendingNav = null; }}
    >
      <p class="text-sm text-gray-400">You'll lose your upload progress.</p>
    </ConfirmModal>
  {/if}
</div>
