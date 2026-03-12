<script>
  import { invoke } from "@tauri-apps/api/core";
  import FileInput from "./bulk-upload/FileInput.svelte";
  import ColumnMapping from "./bulk-upload/ColumnMapping.svelte";
  import TitleCleanupStep from "./bulk-upload/TitleCleanupStep.svelte";
  import ReviewClassified from "./bulk-upload/ReviewClassified.svelte";
  import ReviewRules from "./bulk-upload/ReviewRules.svelte";
  import BulkDone from "./bulk-upload/BulkDone.svelte";
  import StepIndicator from "./StepIndicator.svelte";
  import { QUIP_INTERVAL_MS } from "./constants.js";

  let { ondirtychange = () => {} } = $props();

  // Steps: input -> column-mapping -> cleanup -> review -> rules -> done
  let step = $state("input");
  let isDirty = $derived(step !== "input" && step !== "done");

  $effect(() => {
    ondirtychange(isDirty, step === "rules" ? pendingRules : null);
  });
  let error = $state("");

  // Shared state across steps
  let inputText = $state("");
  let batchFilename = $state("Pasted data");
  let previewRows = $state([]);
  let parserName = $state("");
  let llmWarning = $state("");
  let classifying = $state(false);
  let classifyQuip = $state("");
  let classifyQuipInterval = $state(null);
  const quips = [
    "Shaking the coin jar...",
    "Counting every penny...",
    "Asking the AI nicely...",
    "Bribing the classifier...",
    "Sorting receipts by vibe...",
    "Teaching AI about your spending habits...",
    "Pretending to understand your finances...",
    "Consulting the oracle of expenses...",
    "Making cents of it all...",
    "Running the numbers... literally...",
  ];
  function startQuips() {
    classifyQuip = quips[Math.floor(Math.random() * quips.length)];
    classifyQuipInterval = setInterval(() => {
      classifyQuip = quips[Math.floor(Math.random() * quips.length)];
    }, QUIP_INTERVAL_MS);
  }
  function stopQuips() {
    if (classifyQuipInterval) clearInterval(classifyQuipInterval);
    classifyQuipInterval = null;
  }
  let parsedRows = $state([]);
  let classifiedRows = $state([]);
  let allCategories = $state([]);
  let savedCount = $state(0);
  let pendingRules = $state([]);
  let selectedDelimiter = $state(null);

  async function handleFileInput({ text, filename, delimiter }) {
    error = "";
    inputText = text;
    batchFilename = filename;
    selectedDelimiter = delimiter;

    try {
      const result = await invoke("preview_csv", { input: text, delimiter });
      previewRows = result.rows;
      parserName = result.parser_name;
    } catch (err) {
      error = `Failed to parse file: ${err}`;
      return;
    }

    // Check LLM config
    try {
      const config = await invoke("get_llm_config");
      if (!config.provider || !config.api_key) {
        llmWarning = "No LLM API key configured. Expenses not matched by rules will need manual categorization.";
      } else {
        llmWarning = "";
      }
    } catch (err) {
      console.warn("Failed to check LLM config:", err);
      llmWarning = "Could not check LLM configuration. Expenses not matched by rules may need manual categorization.";
    }

    step = "column-mapping";
  }

  function extractFilenamePattern(name) {
    const base = name.replace(/\.[^.]+$/, "");
    return base.replace(/[-_]?(\d{4}|\d{1,2}|jan|feb|mar|apr|may|jun|jul|aug|sep|oct|nov|dec)[-_\d]*$/i, "") || base;
  }

  async function saveColumnMapping(mapping) {
    try {
      const headers = previewRows[0] ?? [];
      const pattern = extractFilenamePattern(batchFilename);
      const existing = await invoke("get_config", { key: "column_mappings" });
      let mappings = [];
      if (existing) {
        try { mappings = JSON.parse(existing); } catch {}
      }
      // Upsert: remove existing entry with same headers
      const headersKey = JSON.stringify(headers);
      mappings = mappings.filter(m => JSON.stringify(m.headers) !== headersKey);
      mappings.push({
        pattern,
        headers,
        mapping: { title: mapping.title_index, amount: mapping.amount_index, date: mapping.date_index },
        dateFormat: mapping.date_format,
        hasHeader: mapping.has_header,
        savedAt: new Date().toISOString(),
      });
      // Keep 10 most recent
      mappings.sort((a, b) => new Date(b.savedAt) - new Date(a.savedAt));
      mappings = mappings.slice(0, 10);
      await invoke("save_config", { key: "column_mappings", value: JSON.stringify(mappings) });
    } catch (err) {
      console.warn("Failed to save column mapping:", err);
    }
  }

  async function handleMapping(mapping) {
    error = "";
    try {
      const rows = await invoke("parse_csv_data", {
        input: inputText,
        mapping,
        delimiter: selectedDelimiter,
      });
      parsedRows = rows;
      await saveColumnMapping(mapping);
      step = "cleanup";
    } catch (err) {
      error = `Parsing failed: ${err}`;
    }
  }

  async function handleCleanupDone() {
    error = "";
    classifying = true;
    startQuips();
    await new Promise(r => requestAnimationFrame(() => requestAnimationFrame(r)));
    try {
      const rows = await invoke("classify_expenses", {
        rows: parsedRows,
      });
      classifiedRows = rows.map((r) => ({
        ...r,
        _editing: false,
        _autoApplied: 0,
        _originalSource: r.source,
      }));
      await loadCategories();
      step = "review";
    } catch (err) {
      error = `Classification failed: ${err}`;
    } finally {
      classifying = false;
      stopQuips();
    }
  }

  async function loadCategories() {
    try {
      allCategories = await invoke("get_categories");
    } catch (err) {
      console.warn("Failed to load categories:", err);
      allCategories = [];
      error = "Failed to load categories for autocomplete.";
    }
  }

  async function handleSave(nonDuplicateRows) {
    const toSave = nonDuplicateRows.map((r) => ({
      title: r.title,
      amount: Math.abs(r.amount),
      date: r.date,
      category: r.category,
      source: r.source,
    }));

    console.log("[BulkUpload] saving expenses, sources:", toSave.map(e => `${e.title}: ${e.source}`));
    const result = await invoke("bulk_save_expenses", {
      expenses: toSave,
      filename: batchFilename,
    });
    console.log("[BulkUpload] result:", JSON.stringify(result));
    savedCount = result.saved_count;
    pendingRules = result.pending_rules ?? [];
    step = "rules";
  }

  async function handleSaveRules(rules) {
    await invoke("bulk_save_rules", { rules });
    step = "done";
  }

  function handleSkipRules() {
    step = "done";
  }

  function reset() {
    step = "input";
    error = "";
    inputText = "";
    batchFilename = "Pasted data";
    previewRows = [];
    parserName = "";
    llmWarning = "";
    parsedRows = [];
    classifiedRows = [];
    allCategories = [];
    savedCount = 0;
    pendingRules = [];
    selectedDelimiter = null;
  }
</script>

{#if classifying}
  <div class="fixed inset-0 bg-black/60 z-50 flex items-center justify-center">
    <div class="bg-gray-900 border border-gray-800 rounded-2xl p-10 flex flex-col items-center gap-5 shadow-2xl max-w-sm w-full mx-4">
      <div class="coin-bounce">
        <svg class="w-16 h-16" version="1.2" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 32 32" width="64" height="64">
          <defs>
            <image width="26" height="26" id="coin-img1" href="data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAABoAAAAaCAMAAACelLz8AAAAAXNSR0IB2cksfwAAADlQTFRF8Y8G8Y8GAAAAAAAAAAAAAAAA/+yO8Y8GAAAA/+yO8Y8G8ZAH8Y8G/+yN8ZEK+cNS8Y8G8Y8G8Y8GaKutPwAAABN0Uk5TAP9w10DThHFm/9v/mf//WzpoLCJ55BIAAAB+SURBVHicrdLZCoAgEAXQri0QEdX/f6QEFUHLZFZommLRPLjMUQdkEJ2BaxGRmXEQtITa+QlaWjGFE1udFC863Z9LZvWkQenkoAwYH0gUpRyDnK1boJz1YoRFKLoSaFG1JsmzqDkabtFeA/Rcy/wqH/3w8y/oW2+oxBEBfbgBN0c9JXwDi5AAAAAASUVORK5CYII="/>
            <image width="22" height="22" id="coin-img2" href="data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAABYAAAAWBAMAAAA2mnEIAAAAAXNSR0IB2cksfwAAABJQTFRF/+yO/+yO8pUP8ZIK8ZAH8ZILecShyQAAAAZ0Uk5TAP//////enng/gAAAEtJREFUeJxjZGBgEARihvcMDIwobDALzCOKDWcC9eJhv2dk5H/PKAAR/8Bkd4ABwv7P+F7wA5T9QcD+IIz9npGfEa4em5mkuhOHfwFD6RjBvhO95AAAAABJRU5ErkJggg=="/>
          </defs>
          <use href="#coin-img1" x="3" y="3"/>
          <use href="#coin-img2" x="5" y="5"/>
        </svg>
      </div>
      <div class="coin-shadow"></div>
      <p class="text-lg font-semibold text-gray-100">Classifying {parsedRows.length} expenses</p>
      <p class="text-sm text-gray-400 h-5 transition-opacity duration-300">{classifyQuip}</p>
      <div class="flex gap-1.5">
        <div class="w-2 h-2 rounded-full bg-amber-500 coin-dot" style="animation-delay: 0s"></div>
        <div class="w-2 h-2 rounded-full bg-amber-500 coin-dot" style="animation-delay: 0.2s"></div>
        <div class="w-2 h-2 rounded-full bg-amber-500 coin-dot" style="animation-delay: 0.4s"></div>
      </div>
    </div>
  </div>
{/if}

<div>
  <h2 class="text-2xl font-bold mb-2">Expense Bulk Upload</h2>

  <StepIndicator
    steps={[
      { id: "input", label: "1. Input" },
      { id: "column-mapping", label: "2. Columns" },
      { id: "cleanup", label: "3. Cleanup" },
      { id: "review", label: "4. Review" },
      { id: "rules", label: "5. Rules" },
      { id: "done", label: "6. Done" },
    ]}
    currentStep={step}
  />

  {#if error}
    <div class="mb-4 px-4 py-3 rounded-xl bg-red-900/50 border border-red-800/50 text-red-400 text-sm">
      {error}
    </div>
  {/if}

  {#if step === "input"}
    <FileInput onnext={handleFileInput} />
  {:else if step === "column-mapping"}
    <ColumnMapping
      {previewRows}
      {parserName}
      {llmWarning}
      filename={batchFilename}
      onback={() => step = "input"}
      onnext={handleMapping}
    />
  {:else if step === "cleanup"}
    <TitleCleanupStep
      bind:parsedRows
      onback={() => step = "column-mapping"}
      onnext={handleCleanupDone}
    />
  {:else if step === "review"}
    <ReviewClassified
      bind:classifiedRows
      {allCategories}
      onback={() => step = "cleanup"}
      onsave={handleSave}
    />
  {:else if step === "rules"}
    <ReviewRules
      {pendingRules}
      onback={() => step = "review"}
      onsave={handleSaveRules}
      onskip={handleSkipRules}
    />
  {:else if step === "done"}
    <BulkDone {savedCount} onreset={reset} />
  {/if}
</div>
