<script>
  import { invoke } from "@tauri-apps/api/core";
  import FileInput from "./bulk-upload/FileInput.svelte";
  import ColumnMapping from "./bulk-upload/ColumnMapping.svelte";
  import TitleCleanupStep from "./bulk-upload/TitleCleanupStep.svelte";
  import ReviewClassified from "./bulk-upload/ReviewClassified.svelte";
  import BulkDone from "./bulk-upload/BulkDone.svelte";

  // Steps: input -> column-mapping -> cleanup -> review -> done
  let step = $state("input");
  let error = $state("");

  // Shared state across steps
  let inputText = $state("");
  let batchFilename = $state("Pasted data");
  let previewRows = $state([]);
  let parserName = $state("");
  let llmWarning = $state("");
  let classifying = $state(false);
  let parsedRows = $state([]);
  let classifiedRows = $state([]);
  let allCategories = $state([]);
  let savedCount = $state(0);

  async function handleFileInput({ text, filename }) {
    error = "";
    inputText = text;
    batchFilename = filename;

    try {
      const result = await invoke("preview_csv", { input: text });
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
    } catch (err) { console.warn("Failed to check LLM config:", err); llmWarning = ""; }

    step = "column-mapping";
  }

  async function handleMapping(mapping) {
    error = "";
    try {
      const rows = await invoke("parse_csv_data", {
        input: inputText,
        mapping,
      });
      parsedRows = rows;
      step = "cleanup";
    } catch (err) {
      error = `Parsing failed: ${err}`;
    }
  }

  async function handleCleanupDone() {
    error = "";
    classifying = true;
    await new Promise(r => requestAnimationFrame(() => requestAnimationFrame(r)));
    try {
      const rows = await invoke("classify_expenses", {
        rows: parsedRows,
      });
      classifiedRows = rows.map((r, i) => ({
        ...r,
        _editing: false,
        rule_pattern: r.title,
        _autoApplied: 0,
        _originalSource: r.source,
        _originalTitle: parsedRows[i]?._originalTitle || null,
      }));
      await loadCategories();
      step = "review";
    } catch (err) {
      error = `Classification failed: ${err}`;
    } finally {
      classifying = false;
    }
  }

  async function loadCategories() {
    try {
      allCategories = await invoke("get_categories");
    } catch (err) {
      console.warn("Failed to load categories:", err);
      allCategories = [];
    }
  }

  async function handleSave(nonDuplicateRows) {
    const toSave = nonDuplicateRows.map((r) => ({
      title: r._originalTitle || r.title,
      amount: r.amount,
      date: r.date,
      category: r.category,
      source: r.source,
      rule_pattern: r.rule_pattern !== r.title ? r.rule_pattern : null,
      display_title: r._originalTitle ? r.title : null,
    }));

    const count = await invoke("bulk_save_expenses", {
      expenses: toSave,
      filename: batchFilename,
    });
    savedCount = count;
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
  }
</script>

{#if classifying}
  <div class="fixed inset-0 bg-black/60 z-50 flex items-center justify-center">
    <div class="bg-gray-900 border border-gray-800 rounded-2xl p-10 flex flex-col items-center gap-4 shadow-2xl max-w-sm w-full mx-4">
      <div class="w-10 h-10 border-4 border-emerald-500/30 border-t-emerald-500 rounded-full animate-spin"></div>
      <p class="text-lg font-semibold text-gray-100">Classifying expenses...</p>
      <p class="text-sm text-gray-400">{parsedRows.length} expenses — matching rules, then calling AI for the rest</p>
      <div class="w-full bg-gray-800 rounded-full h-1.5 overflow-hidden">
        <div class="h-full bg-emerald-500 rounded-full animate-progress"></div>
      </div>
    </div>
  </div>
{/if}

<div>
  <h2 class="text-2xl font-bold mb-2">Expense Bulk Upload</h2>

  <!-- Progress bar -->
  <div class="flex items-center gap-2 mb-6 text-sm">
    {#each [
      { id: "input", label: "1. Input" },
      { id: "column-mapping", label: "2. Columns" },
      { id: "cleanup", label: "3. Cleanup" },
      { id: "review", label: "4. Review" },
      { id: "done", label: "5. Done" },
    ] as s, i}
      {@const stepOrder = ["input", "column-mapping", "cleanup", "review", "done"]}
      {@const currentIdx = stepOrder.indexOf(step)}
      {@const thisIdx = stepOrder.indexOf(s.id)}
      {#if i > 0}
        <div class="h-px flex-1 max-w-8 {thisIdx <= currentIdx ? 'bg-emerald-500' : 'bg-gray-700'}"></div>
      {/if}
      <span
        class="px-3 py-1 rounded-full text-xs font-medium
          {step === s.id
          ? 'bg-emerald-600 text-white'
          : thisIdx < currentIdx
            ? 'bg-emerald-900/50 text-emerald-400'
            : 'bg-gray-800 text-gray-500'}"
      >
        {s.label}
      </span>
    {/each}
  </div>

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
  {:else if step === "done"}
    <BulkDone {savedCount} onreset={reset} />
  {/if}
</div>
