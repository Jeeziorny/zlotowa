<script>
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";

  let provider = $state("openai");
  let apiKey = $state("");
  let message = $state("");
  let messageType = $state("");
  let isConfigured = $state(false);
  let showKey = $state(false);
  let saving = $state(false);
  let testing = $state(false);

  onMount(async () => {
    try {
      const config = await invoke("get_llm_config");
      if (config.provider && config.api_key) {
        provider = config.provider;
        apiKey = config.api_key;
        isConfigured = true;
      }
    } catch (err) {
      console.error("Failed to load LLM config:", err);
    }
  });

  async function save() {
    if (!apiKey.trim()) {
      message = "API key is required.";
      messageType = "error";
      return;
    }

    saving = true;
    message = "";
    try {
      await invoke("save_llm_config", {
        config: { provider, api_key: apiKey },
      });
      message = "Configuration saved and validated.";
      messageType = "success";
      isConfigured = true;
    } catch (err) {
      message = `${err}`;
      messageType = "error";
    }
    saving = false;
  }

  async function testConnection() {
    if (!apiKey.trim()) {
      message = "API key is required.";
      messageType = "error";
      return;
    }

    testing = true;
    message = "";
    try {
      await invoke("validate_llm_config", {
        config: { provider, api_key: apiKey },
      });
      message = "Connection successful!";
      messageType = "success";
    } catch (err) {
      message = `Connection failed: ${err}`;
      messageType = "error";
    }
    testing = false;
  }

  async function clear() {
    try {
      await invoke("clear_llm_config");
      provider = "openai";
      apiKey = "";
      isConfigured = false;
      message = "LLM configuration cleared.";
      messageType = "success";
    } catch (err) {
      message = `Error: ${err}`;
      messageType = "error";
    }
  }

  function maskKey(key) {
    if (!key || key.length < 8) return key;
    return key.slice(0, 4) + "..." + key.slice(-4);
  }
</script>

<!-- LLM Configuration -->
<div class="bg-gray-900 rounded-xl p-6 border border-gray-800">
  <h3 class="text-lg font-semibold mb-1">LLM Configuration</h3>
  <p class="text-sm text-gray-400 mb-4">
    Configure an LLM provider to auto-classify expenses that can't be matched by existing rules.
    This is optional — you can always classify manually.
  </p>

  <div class="space-y-4">
    <div>
      <label class="block text-sm text-gray-400 mb-1" for="provider">Provider</label>
      <select
        id="provider"
        bind:value={provider}
        class="w-full bg-gray-800 border border-gray-700 rounded-lg px-4 py-2.5
               text-gray-100 focus:outline-none focus:border-emerald-500"
      >
        <option value="openai">OpenAI</option>
        <option value="anthropic">Anthropic</option>
        <option value="ollama">Ollama (local)</option>
      </select>
    </div>

    <div>
      <label class="block text-sm text-gray-400 mb-1" for="apiKey">
        {provider === "ollama" ? "Endpoint URL" : "API Key"}
      </label>
      <div class="relative">
        <input
          id="apiKey"
          type={showKey ? "text" : "password"}
          bind:value={apiKey}
          placeholder={provider === "ollama"
            ? "http://localhost:11434"
            : "sk-..."}
          class="w-full bg-gray-800 border border-gray-700 rounded-lg px-4 py-2.5 pr-16
                 text-gray-100 placeholder-gray-600 focus:outline-none focus:border-emerald-500"
        />
        <button
          type="button"
          onclick={() => (showKey = !showKey)}
          class="absolute right-3 top-1/2 -translate-y-1/2 text-xs text-gray-400
                 hover:text-gray-200"
        >
          {showKey ? "Hide" : "Show"}
        </button>
      </div>
    </div>

    <div class="flex gap-3">
      <button
        onclick={save}
        disabled={saving || testing}
        class="flex-1 bg-emerald-600 hover:bg-emerald-500 disabled:opacity-50 text-white
               font-medium py-2.5 rounded-lg transition-colors"
      >
        {#if saving}
          <span class="inline-block w-4 h-4 border-2 border-white/30 border-t-white rounded-full animate-spin mr-2 align-middle"></span>
          Validating...
        {:else}
          Save
        {/if}
      </button>
      <button
        onclick={testConnection}
        disabled={saving || testing}
        class="px-4 bg-gray-800 hover:bg-gray-700 disabled:opacity-50 text-gray-300
               font-medium py-2.5 rounded-lg transition-colors"
      >
        {#if testing}
          <span class="inline-block w-4 h-4 border-2 border-gray-500/30 border-t-gray-300 rounded-full animate-spin mr-2 align-middle"></span>
          Testing...
        {:else}
          Test Connection
        {/if}
      </button>
      {#if isConfigured}
        <button
          onclick={clear}
          class="px-4 bg-gray-800 hover:bg-gray-700 text-gray-300 font-medium
                 py-2.5 rounded-lg transition-colors"
        >
          Clear
        </button>
      {/if}
    </div>

    {#if message}
      <div
        class="text-sm px-4 py-2 rounded-lg {messageType === 'success'
          ? 'bg-emerald-900/50 text-emerald-400'
          : 'bg-red-900/50 text-red-400'}"
      >
        {message}
      </div>
    {/if}
  </div>
</div>

{#if isConfigured}
  <div class="bg-gray-900 rounded-xl p-6 border border-gray-800">
    <h3 class="text-lg font-semibold mb-2">Current Configuration</h3>
    <div class="text-sm space-y-1">
      <div class="flex justify-between">
        <span class="text-gray-400">Provider</span>
        <span class="text-gray-200">{provider}</span>
      </div>
      <div class="flex justify-between">
        <span class="text-gray-400">Key</span>
        <span class="text-gray-200 font-mono text-xs">{maskKey(apiKey)}</span>
      </div>
    </div>
  </div>
{/if}
