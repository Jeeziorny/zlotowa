<script>
  import { invoke } from "@tauri-apps/api/core";

  // Steps: input -> column-mapping -> review -> done
  let step = $state("input");

  // Step 1: input
  let inputText = $state("");
  let file = $state(null);
  let inputError = $state("");

  // Step 2: column mapping
  let previewRows = $state([]);
  let parserName = $state("");
  let titleCol = $state(0);
  let amountCol = $state(1);
  let dateCol = $state(2);
  let dateFormat = $state("%Y-%m-%d");
  let mappingError = $state("");

  // Step 3: review
  let classifiedRows = $state([]);
  let reviewError = $state("");
  let classifying = $state(false);

  // Step 4: done
  let savedCount = $state(0);

  // Common date formats for the dropdown
  const dateFormats = [
    { value: "%Y-%m-%d", label: "2024-01-15 (YYYY-MM-DD)" },
    { value: "%d-%m-%Y", label: "15-01-2024 (DD-MM-YYYY)" },
    { value: "%m-%d-%Y", label: "01-15-2024 (MM-DD-YYYY)" },
    { value: "%d/%m/%Y", label: "15/01/2024 (DD/MM/YYYY)" },
    { value: "%m/%d/%Y", label: "01/15/2024 (MM/DD/YYYY)" },
    { value: "%Y/%m/%d", label: "2024/01/15 (YYYY/MM/DD)" },
    { value: "%d.%m.%Y", label: "15.01.2024 (DD.MM.YYYY)" },
  ];

  let headerRow = $derived(previewRows.length > 0 ? previewRows[0] : []);
  let dataRows = $derived(previewRows.length > 1 ? previewRows.slice(1) : []);
  let nonDuplicateRows = $derived(classifiedRows.filter((r) => !r.is_duplicate));
  let duplicateRows = $derived(classifiedRows.filter((r) => r.is_duplicate));

  // Grouped rows by source
  let dbClassified = $derived(nonDuplicateRows.filter(r => r.source === "Database"));
  let llmClassified = $derived(nonDuplicateRows.filter(r => r.source === "Llm"));
  let unclassified = $derived(nonDuplicateRows.filter(r => !r.source || r.source === "Manual"));

  function handleFileDrop(event) {
    event.preventDefault();
    const files = event.dataTransfer?.files;
    if (files?.length > 0) {
      file = files[0];
      readFile(files[0]);
    }
  }

  function handleFileSelect(event) {
    const f = event.target.files?.[0];
    if (f) {
      file = f;
      readFile(f);
    }
  }

  function handleDragOver(event) {
    event.preventDefault();
  }

  function readFile(f) {
    const reader = new FileReader();
    reader.onload = (e) => {
      inputText = e.target.result;
    };
    reader.readAsText(f);
  }

  async function goToMapping() {
    if (!inputText.trim()) {
      inputError = "Please paste data or upload a file.";
      return;
    }
    inputError = "";

    try {
      const result = await invoke("preview_csv", { input: inputText });
      previewRows = result.rows;
      parserName = result.parser_name;

      // Auto-detect columns from header names
      if (headerRow.length > 0) {
        for (let i = 0; i < headerRow.length; i++) {
          const h = headerRow[i].toLowerCase();
          if (
            h.includes("title") ||
            h.includes("description") ||
            h.includes("name") ||
            h.includes("merchant") ||
            h.includes("opis") ||
            h.includes("tytuł")
          )
            titleCol = i;
          if (
            h.includes("amount") ||
            h.includes("value") ||
            h.includes("sum") ||
            h.includes("kwota")
          )
            amountCol = i;
          if (h.includes("date") || h.includes("data")) dateCol = i;
        }
      }

      step = "column-mapping";
    } catch (err) {
      inputError = `${err}`;
    }
  }

  async function goToReview() {
    mappingError = "";
    classifying = true;
    try {
      const rows = await invoke("parse_and_classify", {
        input: inputText,
        mapping: {
          title_index: titleCol,
          amount_index: amountCol,
          date_index: dateCol,
          date_format: dateFormat,
        },
      });
      classifiedRows = rows.map((r) => ({ ...r, _editing: false }));
      step = "review";
    } catch (err) {
      mappingError = `${err}`;
    }
    classifying = false;
  }

  function editCategory(index, newCategory) {
    classifiedRows[index].category = newCategory;
    classifiedRows[index].source = "Manual";
  }

  async function saveApproved() {
    reviewError = "";
    const toSave = nonDuplicateRows.map((r) => ({
      title: r.title,
      amount: r.amount,
      date: r.date,
      category: r.category,
      source: r.source,
    }));

    try {
      savedCount = await invoke("bulk_save_expenses", { expenses: toSave });
      step = "done";
    } catch (err) {
      reviewError = `${err}`;
    }
  }

  function reset() {
    step = "input";
    inputText = "";
    file = null;
    inputError = "";
    previewRows = [];
    classifiedRows = [];
    dateFormat = "%Y-%m-%d";
    titleCol = 0;
    amountCol = 1;
    dateCol = 2;
  }
</script>

{#snippet expenseTable(rows, showSource)}
  <div class="overflow-x-auto">
    <table class="w-full text-sm">
      <thead>
        <tr class="border-b border-gray-700 text-gray-400">
          <th class="text-left px-4 py-2">Date</th>
          <th class="text-left px-4 py-2">Title</th>
          <th class="text-right px-4 py-2">Amount</th>
          <th class="text-left px-4 py-2">Category</th>
          {#if showSource}
            <th class="text-left px-4 py-2">Source</th>
          {/if}
        </tr>
      </thead>
      <tbody>
        {#each rows as row}
          {@const origIndex = classifiedRows.indexOf(row)}
          <tr class="border-b border-gray-800/50 hover:bg-gray-800/30">
            <td class="px-4 py-2 text-gray-400">{row.date}</td>
            <td class="px-4 py-2">{row.title}</td>
            <td class="px-4 py-2 text-right font-mono">{row.amount.toFixed(2)}</td>
            <td class="px-4 py-2">
              <input
                type="text"
                value={row.category || ""}
                onchange={(e) => editCategory(origIndex, e.target.value)}
                placeholder="Enter category"
                class="bg-gray-800 border border-gray-700 rounded px-2 py-1
                       text-gray-100 placeholder-gray-600 focus:outline-none
                       focus:border-emerald-500 w-full max-w-48"
              />
            </td>
            {#if showSource}
              <td class="px-4 py-2">
                {#if row.source === "Database"}
                  <span class="px-2 py-0.5 rounded text-xs bg-blue-900/50 text-blue-400">DB Rule</span>
                {:else if row.source === "Llm"}
                  <span class="px-2 py-0.5 rounded text-xs bg-purple-900/50 text-purple-400">LLM</span>
                {:else if row.source === "Manual"}
                  <span class="px-2 py-0.5 rounded text-xs bg-gray-800 text-gray-400">Manual</span>
                {:else}
                  <span class="px-2 py-0.5 rounded text-xs bg-yellow-900/50 text-yellow-400">Unclassified</span>
                {/if}
              </td>
            {/if}
          </tr>
        {/each}
      </tbody>
    </table>
  </div>
{/snippet}

<div>
  <h2 class="text-2xl font-bold mb-2">Bulk Upload</h2>

  <!-- Progress bar -->
  <div class="flex items-center gap-2 mb-6 text-sm">
    {#each [
      { id: "input", label: "1. Input" },
      { id: "column-mapping", label: "2. Columns" },
      { id: "review", label: "3. Review" },
      { id: "done", label: "4. Done" },
    ] as s, i}
      {#if i > 0}
        <div class="h-px flex-1 max-w-8 {step === s.id || ['column-mapping', 'review', 'done'].indexOf(step) > ['column-mapping', 'review', 'done'].indexOf(s.id) ? 'bg-emerald-500' : 'bg-gray-700'}"></div>
      {/if}
      <span
        class="px-3 py-1 rounded-full text-xs font-medium
          {step === s.id
          ? 'bg-emerald-600 text-white'
          : 'bg-gray-800 text-gray-500'}"
      >
        {s.label}
      </span>
    {/each}
  </div>

  <!-- Step 1: Input -->
  {#if step === "input"}
    <div class="max-w-2xl space-y-6">
      <div class="bg-gray-900 rounded-xl p-6 border border-gray-800">
        <h3 class="text-lg font-semibold mb-3">Paste expense data</h3>
        <p class="text-sm text-gray-400 mb-3">
          Paste CSV content from your bank's export.
        </p>
        <textarea
          bind:value={inputText}
          rows="10"
          placeholder="date,title,amount&#10;2024-01-15,Grocery Store,45.99&#10;2024-01-16,Gas Station,62.30"
          class="w-full bg-gray-800 border border-gray-700 rounded-lg px-4 py-3
                 text-gray-100 placeholder-gray-600 focus:outline-none focus:border-emerald-500
                 font-mono text-sm resize-y"
        ></textarea>
      </div>

      <div class="bg-gray-900 rounded-xl p-6 border border-gray-800">
        <h3 class="text-lg font-semibold mb-3">Or upload a file</h3>
        <!-- svelte-ignore a11y_no_static_element_interactions -->
        <div
          ondrop={handleFileDrop}
          ondragover={handleDragOver}
          role="button"
          tabindex="0"
          class="border-2 border-dashed border-gray-700 rounded-lg p-8 text-center
                 hover:border-emerald-500/50 transition-colors cursor-pointer"
        >
          {#if file}
            <p class="text-emerald-400">{file.name}</p>
            <p class="text-xs text-gray-500 mt-1"
              >{(file.size / 1024).toFixed(1)} KB</p
            >
          {:else}
            <p class="text-gray-400 mb-2">
              Drag & drop a .csv or .txt file here
            </p>
            <p class="text-sm text-gray-600">or</p>
          {/if}
          <label
            class="inline-block mt-3 px-4 py-2 bg-gray-800 rounded-lg text-sm
                   text-gray-300 hover:bg-gray-700 cursor-pointer transition-colors"
          >
            Browse files
            <input
              type="file"
              accept=".csv,.txt"
              onchange={handleFileSelect}
              class="hidden"
            />
          </label>
        </div>
      </div>

      {#if inputError}
        <div class="text-sm px-4 py-2 rounded-lg bg-red-900/50 text-red-400">
          {inputError}
        </div>
      {/if}

      <button
        onclick={goToMapping}
        disabled={!inputText.trim()}
        class="w-full bg-emerald-600 hover:bg-emerald-500 disabled:bg-gray-700
               disabled:text-gray-500 text-white font-medium py-3 rounded-xl
               transition-colors"
      >
        Next: Map Columns
      </button>
    </div>

  <!-- Step 2: Column Mapping -->
  {:else if step === "column-mapping"}
    <div class="space-y-6">
      <div class="bg-gray-900 rounded-xl p-6 border border-gray-800">
        <h3 class="text-lg font-semibold mb-1">
          Detected format: {parserName}
        </h3>
        <p class="text-sm text-gray-400 mb-4">
          Select which column is which. Preview shows first few rows.
        </p>

        <div class="grid grid-cols-2 md:grid-cols-4 gap-4 mb-6">
          <div>
            <label class="block text-sm text-gray-400 mb-1">Title column</label
            >
            <select
              bind:value={titleCol}
              class="w-full bg-gray-800 border border-gray-700 rounded-lg px-3 py-2
                     text-gray-100 focus:outline-none focus:border-emerald-500"
            >
              {#each headerRow as col, i}
                <option value={i}>{col}</option>
              {/each}
            </select>
          </div>

          <div>
            <label class="block text-sm text-gray-400 mb-1"
              >Amount column</label
            >
            <select
              bind:value={amountCol}
              class="w-full bg-gray-800 border border-gray-700 rounded-lg px-3 py-2
                     text-gray-100 focus:outline-none focus:border-emerald-500"
            >
              {#each headerRow as col, i}
                <option value={i}>{col}</option>
              {/each}
            </select>
          </div>

          <div>
            <label class="block text-sm text-gray-400 mb-1">Date column</label>
            <select
              bind:value={dateCol}
              class="w-full bg-gray-800 border border-gray-700 rounded-lg px-3 py-2
                     text-gray-100 focus:outline-none focus:border-emerald-500"
            >
              {#each headerRow as col, i}
                <option value={i}>{col}</option>
              {/each}
            </select>
          </div>

          <div>
            <label class="block text-sm text-gray-400 mb-1">Date format</label>
            <select
              bind:value={dateFormat}
              class="w-full bg-gray-800 border border-gray-700 rounded-lg px-3 py-2
                     text-gray-100 focus:outline-none focus:border-emerald-500"
            >
              {#each dateFormats as fmt}
                <option value={fmt.value}>{fmt.label}</option>
              {/each}
            </select>
          </div>
        </div>

        <!-- Preview table -->
        <div class="overflow-x-auto">
          <table class="w-full text-sm">
            <thead>
              <tr class="border-b border-gray-700">
                {#each headerRow as col, i}
                  <th
                    class="text-left px-3 py-2 {i === titleCol
                      ? 'text-emerald-400'
                      : i === amountCol
                        ? 'text-blue-400'
                        : i === dateCol
                          ? 'text-purple-400'
                          : 'text-gray-400'}"
                  >
                    {col}
                    {#if i === titleCol}<span class="text-xs ml-1"
                        >(title)</span
                      >{/if}
                    {#if i === amountCol}<span class="text-xs ml-1"
                        >(amount)</span
                      >{/if}
                    {#if i === dateCol}<span class="text-xs ml-1"
                        >(date)</span
                      >{/if}
                  </th>
                {/each}
              </tr>
            </thead>
            <tbody>
              {#each dataRows.slice(0, 5) as row}
                <tr class="border-b border-gray-800/50">
                  {#each row as cell, i}
                    <td
                      class="px-3 py-2 {i === titleCol
                        ? 'text-emerald-300'
                        : i === amountCol
                          ? 'text-blue-300'
                          : i === dateCol
                            ? 'text-purple-300'
                            : 'text-gray-400'}"
                    >
                      {cell}
                    </td>
                  {/each}
                </tr>
              {/each}
            </tbody>
          </table>
          {#if dataRows.length > 5}
            <p class="text-xs text-gray-500 mt-2 px-3">
              ...and {dataRows.length - 5} more rows
            </p>
          {/if}
        </div>
      </div>

      {#if mappingError}
        <div class="text-sm px-4 py-2 rounded-lg bg-red-900/50 text-red-400">
          {mappingError}
        </div>
      {/if}

      <div class="flex gap-3">
        <button
          onclick={() => (step = "input")}
          class="px-6 bg-gray-800 hover:bg-gray-700 text-gray-300 font-medium
                 py-3 rounded-xl transition-colors"
        >
          Back
        </button>
        <button
          onclick={goToReview}
          disabled={classifying}
          class="flex-1 bg-emerald-600 hover:bg-emerald-500 disabled:opacity-50 text-white font-medium
                 py-3 rounded-xl transition-colors"
        >
          {#if classifying}
            <span class="inline-block w-4 h-4 border-2 border-white/30 border-t-white rounded-full animate-spin mr-2 align-middle"></span>
            Classifying expenses...
          {:else}
            Next: Classify & Review
          {/if}
        </button>
      </div>
    </div>

  <!-- Step 3: Review -->
  {:else if step === "review"}
    <div class="space-y-6">
      <div class="bg-gray-900 rounded-xl p-6 border border-gray-800">
        <h3 class="text-lg font-semibold mb-1">Review Classifications</h3>
        <p class="text-sm text-gray-400 mb-4">
          Edit categories as needed. Click a category to change it.
        </p>
      </div>

      <!-- DB Rule classified -->
      {#if dbClassified.length > 0}
        <div class="bg-gray-900 rounded-xl p-6 border border-gray-800">
          <h4 class="font-semibold mb-3 flex items-center gap-2">
            <span class="w-2 h-2 rounded-full bg-blue-400"></span>
            Classified by rules ({dbClassified.length})
          </h4>
          {@render expenseTable(dbClassified, false)}
        </div>
      {/if}

      <!-- LLM classified -->
      {#if llmClassified.length > 0}
        <div class="bg-gray-900 rounded-xl p-6 border border-gray-800">
          <h4 class="font-semibold mb-3 flex items-center gap-2">
            <span class="w-2 h-2 rounded-full bg-purple-400"></span>
            Classified by AI ({llmClassified.length})
          </h4>
          {@render expenseTable(llmClassified, false)}
        </div>
      {/if}

      <!-- Unclassified -->
      {#if unclassified.length > 0}
        <div class="bg-gray-900 rounded-xl p-6 border border-gray-800">
          <h4 class="font-semibold mb-3 flex items-center gap-2">
            <span class="w-2 h-2 rounded-full bg-yellow-400"></span>
            Needs your input ({unclassified.length})
          </h4>
          {@render expenseTable(unclassified, false)}
        </div>
      {/if}

      {#if duplicateRows.length > 0}
        <div class="bg-gray-900 rounded-xl p-6 border border-yellow-900/50">
          <h3 class="text-lg font-semibold mb-1 text-yellow-400">
            Duplicates ({duplicateRows.length})
          </h3>
          <p class="text-sm text-gray-400 mb-4">
            These expenses already exist in the database and will be skipped.
          </p>
          <div class="overflow-x-auto">
            <table class="w-full text-sm opacity-60">
              <thead>
                <tr class="border-b border-gray-700 text-gray-400">
                  <th class="text-left px-4 py-2">Date</th>
                  <th class="text-left px-4 py-2">Title</th>
                  <th class="text-right px-4 py-2">Amount</th>
                </tr>
              </thead>
              <tbody>
                {#each duplicateRows as row}
                  <tr class="border-b border-gray-800/50">
                    <td class="px-4 py-2 text-gray-500">{row.date}</td>
                    <td class="px-4 py-2 text-gray-500">{row.title}</td>
                    <td class="px-4 py-2 text-right font-mono text-gray-500"
                      >{row.amount.toFixed(2)}</td
                    >
                  </tr>
                {/each}
              </tbody>
            </table>
          </div>
        </div>
      {/if}

      {#if reviewError}
        <div class="text-sm px-4 py-2 rounded-lg bg-red-900/50 text-red-400">
          {reviewError}
        </div>
      {/if}

      <div class="flex gap-3">
        <button
          onclick={() => (step = "column-mapping")}
          class="px-6 bg-gray-800 hover:bg-gray-700 text-gray-300 font-medium
                 py-3 rounded-xl transition-colors"
        >
          Back
        </button>
        <button
          onclick={saveApproved}
          disabled={nonDuplicateRows.length === 0}
          class="flex-1 bg-emerald-600 hover:bg-emerald-500 disabled:bg-gray-700
                 disabled:text-gray-500 text-white font-medium py-3 rounded-xl
                 transition-colors"
        >
          Save {nonDuplicateRows.length} Expenses
        </button>
      </div>
    </div>

  <!-- Step 4: Done -->
  {:else if step === "done"}
    <div
      class="max-w-lg bg-gray-900 rounded-xl p-12 border border-gray-800 text-center"
    >
      <div class="text-4xl mb-4 text-emerald-400">
        {savedCount}
      </div>
      <p class="text-lg font-semibold mb-2">Expenses saved</p>
      <p class="text-sm text-gray-400 mb-6">
        Classification rules have been updated for future imports.
      </p>
      <button
        onclick={reset}
        class="px-6 bg-emerald-600 hover:bg-emerald-500 text-white font-medium
               py-2.5 rounded-lg transition-colors"
      >
        Upload More
      </button>
    </div>
  {/if}
</div>
