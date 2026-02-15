<script>
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";

  let expenses = $state([]);

  onMount(async () => {
    try {
      expenses = await invoke("get_expenses");
    } catch (err) {
      console.error("Failed to load expenses:", err);
    }
  });

  function sourceLabel(source) {
    if (!source) return "";
    switch (source) {
      case "Database": return "DB";
      case "Llm": return "LLM";
      case "Manual": return "Manual";
      default: return source;
    }
  }

  function sourceBadgeClass(source) {
    switch (source) {
      case "Database": return "bg-blue-900/50 text-blue-400";
      case "Llm": return "bg-purple-900/50 text-purple-400";
      case "Manual": return "bg-gray-800 text-gray-400";
      default: return "bg-gray-800 text-gray-400";
    }
  }
</script>

<div>
  <h2 class="text-2xl font-bold mb-6">Expenses</h2>

  {#if expenses.length === 0}
    <div class="bg-gray-900 rounded-xl p-12 border border-gray-800 text-center text-gray-500">
      <p class="text-lg mb-2">No expenses yet</p>
      <p class="text-sm">Add an expense or do a bulk upload to get started.</p>
    </div>
  {:else}
    <div class="bg-gray-900 rounded-xl border border-gray-800 overflow-hidden">
      <table class="w-full">
        <thead>
          <tr class="border-b border-gray-800 text-sm text-gray-400">
            <th class="text-left px-6 py-3">Date</th>
            <th class="text-left px-6 py-3">Title</th>
            <th class="text-right px-6 py-3">Amount</th>
            <th class="text-left px-6 py-3">Category</th>
            <th class="text-left px-6 py-3">Source</th>
          </tr>
        </thead>
        <tbody>
          {#each expenses as expense}
            <tr class="border-b border-gray-800/50 hover:bg-gray-800/30">
              <td class="px-6 py-3 text-sm text-gray-400">{expense.date}</td>
              <td class="px-6 py-3">{expense.title}</td>
              <td class="px-6 py-3 text-right font-mono">{expense.amount.toFixed(2)}</td>
              <td class="px-6 py-3">
                {#if expense.category}
                  <span class="bg-emerald-900/30 text-emerald-400 px-2 py-0.5 rounded text-sm">
                    {expense.category}
                  </span>
                {:else}
                  <span class="text-gray-600 text-sm">-</span>
                {/if}
              </td>
              <td class="px-6 py-3">
                {#if expense.classification_source}
                  <span class="px-2 py-0.5 rounded text-xs {sourceBadgeClass(expense.classification_source)}">
                    {sourceLabel(expense.classification_source)}
                  </span>
                {/if}
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
  {/if}
</div>
