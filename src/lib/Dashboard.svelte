<script>
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { widgets, defaultWidgetIds } from "./widgets/registry.js";

  let { onnavigate = () => {} } = $props();

  let expenses = $state([]);
  let activeWidgetIds = $state([]);
  let showPicker = $state(false);
  let loaded = $state(false);

  let activeWidgets = $derived(
    activeWidgetIds
      .map((id) => widgets.find((w) => w.id === id))
      .filter(Boolean)
  );

  let inactiveWidgets = $derived(
    widgets.filter((w) => !activeWidgetIds.includes(w.id))
  );

  onMount(async () => {
    try {
      expenses = await invoke("get_expenses");

      const saved = await invoke("get_active_widgets");
      if (saved) {
        activeWidgetIds = saved;
      } else {
        activeWidgetIds = [...defaultWidgetIds];
      }
    } catch (err) {
      console.error("Failed to load dashboard:", err);
      activeWidgetIds = [...defaultWidgetIds];
    }
    loaded = true;
  });

  async function addWidget(id) {
    activeWidgetIds = [...activeWidgetIds, id];
    showPicker = false;
    await persist();
  }

  async function removeWidget(id) {
    activeWidgetIds = activeWidgetIds.filter((wid) => wid !== id);
    await persist();
  }

  async function moveWidget(index, direction) {
    const newIndex = index + direction;
    if (newIndex < 0 || newIndex >= activeWidgetIds.length) return;
    const copy = [...activeWidgetIds];
    [copy[index], copy[newIndex]] = [copy[newIndex], copy[index]];
    activeWidgetIds = copy;
    await persist();
  }

  async function persist() {
    try {
      await invoke("save_active_widgets", { widgetIds: activeWidgetIds });
    } catch (err) {
      console.error("Failed to save widget config:", err);
    }
  }
</script>

<div>
  <div class="flex items-center justify-between mb-6">
    <h2 class="text-2xl font-bold">Dashboard</h2>
    <button
      onclick={() => (showPicker = !showPicker)}
      class="px-4 py-2 text-sm bg-gray-800 hover:bg-gray-700 text-gray-300
             rounded-lg transition-colors"
    >
      {showPicker ? "Close" : "+ Add Widget"}
    </button>
  </div>

  <!-- Widget picker -->
  {#if showPicker}
    <div class="bg-gray-900 rounded-xl p-6 border border-gray-800 mb-6">
      <h3 class="text-lg font-semibold mb-3">Available Widgets</h3>
      {#if inactiveWidgets.length === 0}
        <p class="text-sm text-gray-500">All widgets are already on the dashboard.</p>
      {:else}
        <div class="grid grid-cols-1 md:grid-cols-2 gap-3">
          {#each inactiveWidgets as widget}
            <button
              onclick={() => addWidget(widget.id)}
              class="text-left p-4 bg-gray-800 hover:bg-gray-700 rounded-lg
                     transition-colors border border-gray-700 hover:border-emerald-500/50"
            >
              <div class="font-medium text-gray-200">{widget.name}</div>
              <div class="text-sm text-gray-400 mt-1">{widget.description}</div>
            </button>
          {/each}
        </div>
      {/if}
    </div>
  {/if}

  <!-- Widgets -->
  {#if loaded && expenses.length === 0 && activeWidgets.length > 0}
    <div class="bg-gray-900 rounded-xl p-12 border border-gray-800 text-center text-gray-500">
      <p class="text-lg mb-2">No expenses yet</p>
      <p class="text-sm">Add an expense or do a bulk upload to get started.</p>
    </div>
  {:else if loaded}
    <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
      {#each activeWidgets as widget, i (widget.id)}
        <div class={widget.size === "full" ? "md:col-span-2" : ""}>
          <!-- Widget toolbar -->
          <div class="flex items-center justify-end gap-1 mb-1">
            <button
              onclick={() => moveWidget(i, -1)}
              disabled={i === 0}
              class="text-xs px-1.5 py-0.5 text-gray-500 hover:text-gray-300
                     disabled:opacity-30 disabled:hover:text-gray-500"
              title="Move left"
            >
              ←
            </button>
            <button
              onclick={() => moveWidget(i, 1)}
              disabled={i === activeWidgets.length - 1}
              class="text-xs px-1.5 py-0.5 text-gray-500 hover:text-gray-300
                     disabled:opacity-30 disabled:hover:text-gray-500"
              title="Move right"
            >
              →
            </button>
            <button
              onclick={() => removeWidget(widget.id)}
              class="text-xs px-1.5 py-0.5 text-gray-500 hover:text-red-400"
              title="Remove widget"
            >
              ×
            </button>
          </div>

          <!-- Widget content -->
          <widget.component {expenses} {onnavigate} />
        </div>
      {/each}
    </div>
  {/if}
</div>
