<script>
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { widgets, defaultWidgetInstances } from "./widgets/registry.js";
  import { focusTrap } from "./actions/focusTrap.js";

  let { onnavigate = () => {} } = $props();

  let expenses = $state([]);
  let activeInstances = $state([]);
  let showPicker = $state(false);
  let editing = $state(false);
  let loaded = $state(false);
  let configDialog = $state(null); // { instanceId?, widgetId, chips[], input }

  let activeWidgets = $derived(
    activeInstances
      .map((inst) => {
        const def = widgets.find((w) => w.id === inst.widgetId);
        if (!def) return null;
        return { ...def, instanceId: inst.instanceId, config: inst.config };
      })
      .filter(Boolean)
  );

  let pickerWidgets = $derived.by(() => {
    return widgets.filter((w) => {
      if (w.multiInstance) return true;
      return !activeInstances.some((inst) => inst.widgetId === w.id);
    });
  });

  onMount(async () => {
    try {
      expenses = await invoke("get_expenses");

      const saved = await invoke("get_active_widgets");
      if (saved && Array.isArray(saved) && saved.length > 0) {
        // Migrate: old format is string[], new format is object[]
        if (typeof saved[0] === "string") {
          activeInstances = saved.map((id) => ({ widgetId: id, instanceId: id }));
          await persist();
        } else {
          activeInstances = saved;
        }
      } else {
        activeInstances = [...defaultWidgetInstances];
      }
    } catch (err) {
      console.error("Failed to load dashboard:", err);
      activeInstances = [...defaultWidgetInstances];
    }
    loaded = true;
  });

  function addWidget(widgetDef) {
    if (widgetDef.configurable) {
      configDialog = { widgetId: widgetDef.id, chips: [], input: "" };
    } else {
      activeInstances = [...activeInstances, { widgetId: widgetDef.id, instanceId: widgetDef.id }];
      if (!widgetDef.multiInstance) {
        // Hide picker if no more single-instance widgets available
        const remaining = widgets.filter(
          (w) => w.multiInstance || !activeInstances.some((inst) => inst.widgetId === w.id)
        );
        if (remaining.length === 0) showPicker = false;
      }
      persist();
    }
  }

  function addChip() {
    if (!configDialog) return;
    const val = configDialog.input.trim();
    if (val && !configDialog.chips.includes(val)) {
      configDialog.chips = [...configDialog.chips, val];
    }
    configDialog.input = "";
  }

  function removeChip(chip) {
    if (!configDialog) return;
    configDialog.chips = configDialog.chips.filter((c) => c !== chip);
  }

  function confirmConfig() {
    if (!configDialog) return;
    // Flush any remaining input text as a chip
    const trailing = configDialog.input.trim();
    const allChips = trailing && !configDialog.chips.includes(trailing)
      ? [...configDialog.chips, trailing]
      : [...configDialog.chips];
    if (allChips.length === 0) return;
    const keyword = allChips.join(", ");

    if (configDialog.instanceId) {
      // Editing existing instance
      activeInstances = activeInstances.map((inst) =>
        inst.instanceId === configDialog.instanceId
          ? { ...inst, config: { keyword } }
          : inst
      );
    } else {
      // Adding new instance
      const instanceId = `${configDialog.widgetId}-${Date.now()}`;
      activeInstances = [
        ...activeInstances,
        { widgetId: configDialog.widgetId, instanceId, config: { keyword } },
      ];
    }
    configDialog = null;
    persist();
  }

  function editWidgetConfig(instanceId) {
    const inst = activeInstances.find((i) => i.instanceId === instanceId);
    if (!inst) return;
    const existing = inst.config?.keyword || "";
    configDialog = {
      instanceId,
      widgetId: inst.widgetId,
      chips: existing ? existing.split(",").map((k) => k.trim()).filter(Boolean) : [],
      input: "",
    };
  }

  async function removeWidget(instanceId) {
    activeInstances = activeInstances.filter((inst) => inst.instanceId !== instanceId);
    await persist();
  }

  async function moveWidget(index, direction) {
    const newIndex = index + direction;
    if (newIndex < 0 || newIndex >= activeInstances.length) return;
    const copy = [...activeInstances];
    [copy[index], copy[newIndex]] = [copy[newIndex], copy[index]];
    activeInstances = copy;
    await persist();
  }

  async function updateWidgetConfig(instanceId, newConfig) {
    activeInstances = activeInstances.map((inst) =>
      inst.instanceId === instanceId ? { ...inst, config: newConfig } : inst
    );
    await persist();
  }

  async function persist() {
    try {
      await invoke("save_active_widgets", { widgets: activeInstances });
    } catch (err) {
      console.error("Failed to save widget config:", err);
    }
  }

  function handleConfigKeydown(e) {
    if (e.key === "Enter") {
      e.preventDefault();
      if (configDialog.input.trim()) {
        addChip();
      } else {
        confirmConfig();
      }
    }
    if (e.key === "Backspace" && !configDialog.input && configDialog.chips.length > 0) {
      configDialog.chips = configDialog.chips.slice(0, -1);
    }
    if (e.key === "Escape") configDialog = null;
  }
</script>

<div>
  <div class="flex items-center justify-between mb-6">
    <h2 class="text-2xl font-bold">Dashboard</h2>
    <div class="flex items-center gap-3">
      {#if editing}
        <button
          onclick={() => (showPicker = !showPicker)}
          class="px-4 py-2 text-sm bg-gray-800 hover:bg-gray-700 text-gray-300
                 rounded-lg transition-colors"
        >
          {showPicker ? "Close picker" : "+ Add Widget"}
        </button>
      {/if}
      <button
        onclick={() => { editing = !editing; if (!editing) showPicker = false; }}
        class="px-4 py-2 text-sm rounded-lg transition-colors {editing
          ? 'bg-amber-500 hover:bg-amber-400 text-gray-950'
          : 'bg-gray-800 hover:bg-gray-700 text-gray-300'}"
      >
        {editing ? "Done" : "Edit dashboard"}
      </button>
    </div>
  </div>

  <!-- Widget picker -->
  {#if showPicker}
    <div class="bg-gray-900 rounded-xl p-6 border border-gray-800 mb-6">
      <h3 class="text-lg font-semibold mb-3">Available Widgets</h3>
      {#if pickerWidgets.length === 0}
        <p class="text-sm text-gray-500">All widgets are already on the dashboard.</p>
      {:else}
        <div class="grid grid-cols-1 md:grid-cols-2 gap-3">
          {#each pickerWidgets as widget}
            <button
              onclick={() => addWidget(widget)}
              class="text-left p-4 bg-gray-800 hover:bg-gray-700 rounded-lg
                     transition-colors border border-gray-700 hover:border-amber-500/50"
            >
              <div class="font-medium text-gray-200">
                {widget.name}
                {#if widget.multiInstance && activeInstances.some((i) => i.widgetId === widget.id)}
                  <span class="text-xs text-gray-500 ml-1">(+ add another)</span>
                {/if}
              </div>
              <div class="text-sm text-gray-400 mt-1">{widget.description}</div>
            </button>
          {/each}
        </div>
      {/if}
    </div>
  {/if}

  <!-- Config dialog -->
  {#if configDialog}
    <div class="fixed inset-0 bg-black/60 flex items-center justify-center z-50"
         role="presentation"
         onclick={(e) => { if (e.target === e.currentTarget) configDialog = null; }}
         onkeydown={(e) => { if (e.key === "Escape") configDialog = null; }}>
      <div class="bg-gray-900 border border-gray-700 rounded-xl p-6 w-full max-w-sm mx-4"
           role="dialog" aria-modal="true" aria-labelledby="config-dialog-title"
           tabindex="-1"
           use:focusTrap
           onclick={(e) => e.stopPropagation()}>
        <h3 id="config-dialog-title" class="text-lg font-semibold mb-4">
          {configDialog.instanceId ? "Edit" : "Add"} Keyword Tracker
        </h3>
        <label for="keyword-config-input" class="block text-sm text-gray-400 mb-2">Keywords</label>
        <!-- svelte-ignore a11y_click_events_have_key_events -->
        <!-- svelte-ignore a11y_no_static_element_interactions -->
        <div
          class="flex flex-wrap items-center gap-1.5 w-full px-2 py-1.5 bg-gray-800 border border-gray-700
                 rounded-lg focus-within:border-amber-500 min-h-[38px] cursor-text"
          onclick={() => document.getElementById("keyword-config-input")?.focus()}
        >
          {#each configDialog.chips as chip}
            <span class="flex items-center gap-1 text-sm text-amber-400 bg-amber-400/10 px-2 py-0.5 rounded">
              {chip}
              <button
                onclick={(e) => { e.stopPropagation(); removeChip(chip); }}
                class="text-amber-400/60 hover:text-amber-300 text-xs leading-none"
                aria-label="Remove {chip}"
              >&times;</button>
            </span>
          {/each}
          <!-- svelte-ignore a11y_autofocus -->
          <input
            id="keyword-config-input"
            type="text"
            bind:value={configDialog.input}
            onkeydown={handleConfigKeydown}
            placeholder={configDialog.chips.length === 0 ? "Type a keyword and press Enter" : "Add another..."}
            class="flex-1 min-w-[80px] bg-transparent text-gray-200 placeholder-gray-500
                   focus:outline-none text-sm py-0.5"
            autofocus
          />
        </div>
        <p class="text-xs text-gray-500 mt-1.5">Press Enter to add each keyword</p>
        <div class="flex justify-end gap-3 mt-4">
          <button
            onclick={() => (configDialog = null)}
            class="px-4 py-2 text-sm text-gray-400 hover:text-gray-200 transition-colors"
          >
            Cancel
          </button>
          <button
            onclick={confirmConfig}
            disabled={configDialog.chips.length === 0 && !configDialog.input.trim()}
            class="px-4 py-2 text-sm bg-amber-500 hover:bg-amber-400 text-gray-950
                   rounded-lg transition-colors disabled:opacity-40 disabled:hover:bg-amber-500"
          >
            {configDialog.instanceId ? "Save" : "Add"}
          </button>
        </div>
      </div>
    </div>
  {/if}

  <!-- Widgets -->
  {#if !loaded}
    <div class="bg-gray-900 rounded-xl p-12 border border-gray-800 text-center text-gray-500">
      <div class="w-8 h-8 border-4 border-gray-700 border-t-amber-500 rounded-full animate-spin mx-auto mb-3"></div>
      <p class="text-sm">Loading dashboard...</p>
    </div>
  {:else if loaded && expenses.length === 0 && activeWidgets.length > 0}
    <div class="bg-gray-900 rounded-xl p-12 border border-gray-800 text-center text-gray-500">
      <p class="text-lg mb-2">No expenses yet</p>
      <p class="text-sm">Add an expense or do a bulk upload to get started.</p>
    </div>
  {:else if loaded}
    <div class="grid grid-cols-1 md:grid-cols-2 gap-6 items-start">
      {#each activeWidgets as widget, i (widget.instanceId)}
        <div class={widget.size === "full" ? "md:col-span-2" : ""}>
          {#if editing}
            <!-- Widget toolbar -->
            <div class="flex items-center justify-end gap-1 mb-1">
              {#if widget.configurable}
                <button
                  onclick={() => editWidgetConfig(widget.instanceId)}
                  class="text-xs px-1.5 py-0.5 text-gray-500 hover:text-amber-400"
                  title="Edit config"
                  aria-label="Edit {widget.name} config"
                >
                  &#x270E;
                </button>
              {/if}
              <button
                onclick={() => moveWidget(i, -1)}
                disabled={i === 0}
                class="text-xs px-1.5 py-0.5 text-gray-500 hover:text-gray-300
                       disabled:opacity-30 disabled:hover:text-gray-500"
                title="Move left"
                aria-label="Move {widget.name} left"
              >
                ←
              </button>
              <button
                onclick={() => moveWidget(i, 1)}
                disabled={i === activeWidgets.length - 1}
                class="text-xs px-1.5 py-0.5 text-gray-500 hover:text-gray-300
                       disabled:opacity-30 disabled:hover:text-gray-500"
                title="Move right"
                aria-label="Move {widget.name} right"
              >
                →
              </button>
              <button
                onclick={() => removeWidget(widget.instanceId)}
                class="text-xs px-1.5 py-0.5 text-gray-500 hover:text-red-400"
                title="Remove widget"
                aria-label="Remove {widget.name}"
              >
                ×
              </button>
            </div>
          {/if}

          <!-- Widget content -->
          <widget.component
            {expenses}
            {onnavigate}
            config={widget.config}
            onconfigchange={(cfg) => updateWidgetConfig(widget.instanceId, cfg)}
          />
        </div>
      {/each}
    </div>
  {/if}
</div>
