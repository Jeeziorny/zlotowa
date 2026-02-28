<script>
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import LlmSettings from "./settings/LlmSettings.svelte";
  import UploadHistory from "./settings/UploadHistory.svelte";
  import BackupRestore from "./settings/BackupRestore.svelte";

  let { onrulesvisibilitychange = () => {} } = $props();

  let showRulesTab = $state(false);

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

  <div class="max-w-lg space-y-6">
    <LlmSettings />
    <UploadHistory />
    <BackupRestore />

    <div class="bg-gray-900 rounded-xl p-5 border border-gray-800">
      <h3 class="text-sm font-semibold text-gray-300 mb-3">Advanced</h3>
      <label class="flex items-center gap-3 cursor-pointer">
        <input
          type="checkbox"
          checked={showRulesTab}
          onchange={toggleRulesTab}
          class="rounded bg-gray-800 border-gray-700 text-emerald-500 focus:ring-emerald-500"
        />
        <div>
          <span class="text-sm text-gray-200">Show Rules tab in sidebar</span>
          <p class="text-xs text-gray-500">View and manage regex classification rules</p>
        </div>
      </label>
    </div>
  </div>
</div>
