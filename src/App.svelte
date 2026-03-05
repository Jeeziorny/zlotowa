<script>
  import { onMount, onDestroy } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import Sidebar from "./lib/Sidebar.svelte";
  import Dashboard from "./lib/Dashboard.svelte";
  import ExpenseList from "./lib/ExpenseList.svelte";
  import Settings from "./lib/Settings.svelte";
  import Categories from "./lib/Categories.svelte";
  import BudgetPlanning from "./lib/BudgetPlanning.svelte";
  import Rules from "./lib/Rules.svelte";
  import ConfirmModal from "./lib/ConfirmModal.svelte";
  import KeyboardShortcuts from "./lib/KeyboardShortcuts.svelte";

  let currentPage = $state("dashboard");
  let expenseSubView = $state("list");
  let expensesBulkDirty = $state(false);
  let pendingNav = $state(null);
  let showRules = $state(false);
  let showShortcuts = $state(false);

  onMount(async () => {
    try {
      const val = await invoke("get_config", { key: "show_rules_tab" });
      showRules = val === "true";
    } catch {}
    window.addEventListener("keydown", handleGlobalKeydown);
  });

  onDestroy(() => {
    window.removeEventListener("keydown", handleGlobalKeydown);
  });

  function handleGlobalKeydown(e) {
    const mod = e.metaKey || e.ctrlKey;
    const tag = document.activeElement?.tagName?.toLowerCase();
    const inInput = tag === "input" || tag === "textarea" || tag === "select";
    const hasModal = !!document.querySelector("[role='dialog']");

    // Escape always works
    if (e.key === "Escape") {
      if (showShortcuts) {
        showShortcuts = false;
        e.preventDefault();
        return;
      }
      // Don't handle Escape if a modal is open — modals handle it themselves
      if (hasModal) return;
      // Go back from subview
      if (currentPage === "expenses" && expenseSubView !== "list") {
        if (expensesBulkDirty) {
          pendingNav = "__back";
        } else {
          expenseSubView = "list";
        }
        e.preventDefault();
      }
      return;
    }

    // All other shortcuts require modifier and no input focus
    if (!mod || inInput) return;
    // Don't fire if a modal is open
    if (hasModal) return;

    const key = e.key.toLowerCase();

    if (key === "n") {
      e.preventDefault();
      navigateTo("expenses", "add");
    } else if (key === "u") {
      e.preventDefault();
      navigateTo("expenses", "bulk");
    } else if (key === "k") {
      e.preventDefault();
      if (currentPage !== "expenses") {
        navigateTo("expenses", "list");
      }
      // Focus search after a tick so ExpenseList has mounted
      requestAnimationFrame(() => {
        document.getElementById("expense-search")?.focus();
      });
    } else if (key === "1") {
      e.preventDefault();
      navigateTo("dashboard");
    } else if (key === "2") {
      e.preventDefault();
      navigateTo("expenses", "list");
    } else if (key === "3") {
      e.preventDefault();
      navigateTo("categories");
    } else if (key === "4") {
      e.preventDefault();
      navigateTo("budget");
    }
  }

  function navigateTo(page, subView) {
    if (page === "expenses") {
      if (currentPage === "expenses" && expensesBulkDirty && expenseSubView === "bulk" && subView !== "bulk") {
        pendingNav = page;
        return;
      }
      currentPage = "expenses";
      if (subView) expenseSubView = subView;
    } else {
      handleNavigate(page);
    }
  }

  function handleNavigate(page) {
    if (page === "expenses:add") {
      navigateTo("expenses", "add");
      return;
    }
    if (page === currentPage) return;
    if (currentPage === "expenses" && expensesBulkDirty) {
      pendingNav = page;
    } else {
      currentPage = page;
      if (page === "expenses") expenseSubView = "list";
    }
  }

  function confirmNavigation() {
    expensesBulkDirty = false;
    if (pendingNav === "__back") {
      expenseSubView = "list";
    } else {
      currentPage = pendingNav;
      if (pendingNav === "expenses") expenseSubView = "list";
    }
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
  <Sidebar {currentPage} {showRules} onnavigate={handleNavigate} onshowshortcuts={() => showShortcuts = true} />

  <main class="flex-1 overflow-y-auto p-8">
    {#if currentPage === "dashboard"}
      <Dashboard onnavigate={handleNavigate} />
    {:else if currentPage === "expenses"}
      <ExpenseList bind:subView={expenseSubView} onbulkdirtychange={(dirty) => { expensesBulkDirty = dirty; }} />
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

  {#if showShortcuts}
    <KeyboardShortcuts onclose={() => showShortcuts = false} />
  {/if}
</div>
