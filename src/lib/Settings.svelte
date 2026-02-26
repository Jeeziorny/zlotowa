<script>
  import LlmSettings from "./settings/LlmSettings.svelte";
  import UploadHistory from "./settings/UploadHistory.svelte";
</script>

<div>
  <h2 class="text-2xl font-bold mb-6">Settings</h2>

  <div class="max-w-lg space-y-6">
    <LlmSettings />
    <UploadHistory />
  </div>
</div>
