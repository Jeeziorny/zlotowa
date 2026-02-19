<script>
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";

  let { expenses } = $props();

  let loading = $state(true);
  let summary = $state(null);

  let remaining = $derived(
    summary ? summary.total_budgeted - summary.total_spent : 0,
  );
  let ratio = $derived(
    summary && summary.total_budgeted > 0
      ? summary.total_spent / summary.total_budgeted
      : 0,
  );
  let barWidth = $derived(Math.min(ratio * 100, 100));
  let barColor = $derived(
    ratio > 1 ? "bg-red-500" : ratio >= 0.8 ? "bg-amber-500" : "bg-emerald-500",
  );
  let overCount = $derived(
    summary ? summary.categories.filter((c) => c.status === "over").length : 0,
  );
  let hasBudget = $derived(summary && summary.total_budgeted > 0);

  onMount(async () => {
    try {
      summary = await invoke("get_active_budget_summary");
    } catch (err) {
      console.error("Failed to load budget status:", err);
    }
    loading = false;
  });
</script>

<div class="bg-gray-900 rounded-xl p-6 border border-gray-800">
  <h3 class="text-lg font-semibold mb-4">
    Budget Status
    {#if summary}
      <span class="text-sm font-normal text-gray-500 ml-2">
        {summary.start_date} — {summary.end_date}
      </span>
    {/if}
  </h3>

  {#if loading}
    <p class="text-sm text-gray-500">Loading...</p>
  {:else if !hasBudget}
    <p class="text-sm text-gray-500">No active budget.</p>
    <p class="text-xs text-gray-600 mt-1">
      Visit the Budget page to create one.
    </p>
  {:else}
    <div class="grid grid-cols-3 gap-3 text-center mb-4">
      <div>
        <div class="text-xs text-gray-500">Budgeted</div>
        <div class="font-bold font-mono">
          {summary.total_budgeted.toFixed(2)}
        </div>
      </div>
      <div>
        <div class="text-xs text-gray-500">Spent</div>
        <div class="font-bold font-mono">{summary.total_spent.toFixed(2)}</div>
      </div>
      <div>
        <div class="text-xs text-gray-500">
          {remaining >= 0 ? "Left" : "Over"}
        </div>
        <div
          class="font-bold font-mono {remaining >= 0
            ? 'text-emerald-400'
            : 'text-red-400'}"
        >
          {Math.abs(remaining).toFixed(2)}
        </div>
      </div>
    </div>

    <!-- Progress bar -->
    <div class="w-full bg-gray-800 rounded-full h-3">
      <div
        class="{barColor} h-3 rounded-full transition-all"
        style="width: {barWidth}%"
      ></div>
    </div>

    {#if overCount > 0}
      <p class="text-xs text-red-400 mt-2">
        {overCount} categor{overCount === 1 ? "y" : "ies"} over budget
      </p>
    {/if}
  {/if}
</div>
