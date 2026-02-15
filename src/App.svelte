<script>
  import Sidebar from "./lib/Sidebar.svelte";
  import Dashboard from "./lib/Dashboard.svelte";
  import AddExpense from "./lib/AddExpense.svelte";
  import BulkUpload from "./lib/BulkUpload.svelte";
  import ExpenseList from "./lib/ExpenseList.svelte";

  let currentPage = $state("dashboard");
</script>

<div class="flex h-screen bg-gray-950 text-gray-100">
  <Sidebar {currentPage} onnavigate={(page) => currentPage = page} />

  <main class="flex-1 overflow-y-auto p-8">
    {#if currentPage === "dashboard"}
      <Dashboard />
    {:else if currentPage === "add"}
      <AddExpense />
    {:else if currentPage === "bulk"}
      <BulkUpload />
    {:else if currentPage === "expenses"}
      <ExpenseList />
    {/if}
  </main>
</div>
