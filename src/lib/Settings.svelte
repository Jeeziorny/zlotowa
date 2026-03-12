<script>
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import LlmSettings from "./settings/LlmSettings.svelte";
  import UploadHistory from "./settings/UploadHistory.svelte";
  import BackupRestore from "./settings/BackupRestore.svelte";
  import DataExport from "./settings/DataExport.svelte";

  let { onrulesvisibilitychange = () => {} } = $props();

  let activeTab = $state("general");
  let showRulesTab = $state(false);

  const tabs = [
    { id: "general", label: "General" },
    { id: "llm", label: "LLM" },
    { id: "data", label: "Data" },
  ];

  onMount(async () => {
    try {
      const val = await invoke("get_config", { key: "show_rules_tab" });
      showRulesTab = val === "true";
    } catch {}
  });

  async function toggleRulesTab() {
    showRulesTab = !showRulesTab;
    try {
      await invoke("save_config", { key: "show_rules_tab", value: String(showRulesTab) });
    } catch (err) {
      console.warn("Failed to save rules tab config:", err);
    }
    onrulesvisibilitychange(showRulesTab);
  }
</script>

<div>
  <h2 class="text-2xl font-bold mb-6">Settings</h2>

  <div class="max-w-4xl">
    <!-- Tab bar -->
    <div class="flex gap-1 mb-6 border-b border-gray-800">
      {#each tabs as tab}
        <button
          onclick={() => (activeTab = tab.id)}
          class="px-4 py-2 text-sm font-medium transition-colors relative
                 {activeTab === tab.id
                   ? 'text-amber-400'
                   : 'text-gray-400 hover:text-gray-200'}"
        >
          {tab.label}
          {#if activeTab === tab.id}
            <span class="absolute bottom-0 left-0 right-0 h-0.5 bg-amber-400 rounded-t"></span>
          {/if}
        </button>
      {/each}
    </div>

    <!-- General tab -->
    {#if activeTab === "general"}
      <div class="space-y-6">
        <div class="bg-gray-900 rounded-xl p-5 border border-gray-800">
          <h3 class="text-sm font-semibold text-gray-300 mb-4">Preferences</h3>
          <label class="flex items-center gap-3 cursor-pointer">
            <input
              type="checkbox"
              checked={showRulesTab}
              onchange={toggleRulesTab}
              class="rounded bg-gray-800 border-gray-700 text-amber-500 focus:ring-amber-500"
            />
            <div>
              <span class="text-sm text-gray-200">Show Rules tab in sidebar</span>
              <p class="text-xs text-gray-500">View and manage regex classification rules</p>
            </div>
          </label>
        </div>
      </div>
    {/if}

    <!-- LLM tab -->
    {#if activeTab === "llm"}
      <div class="space-y-6">
        <LlmSettings />
      </div>
    {/if}

    <!-- Data tab -->
    {#if activeTab === "data"}
      <div class="space-y-6">
        <DataExport />
        <UploadHistory />
        <BackupRestore />
      </div>
    {/if}
  </div>
</div>
