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
  let columnRoles = $state({});
  let activePopover = $state(null);
  let dateFormat = $state("%Y-%m-%d");
  let mappingError = $state("");
  let llmWarning = $state("");
  let llmWarningDismissed = $state(false);

  // Step 3: review
  let classifiedRows = $state([]);
  let reviewError = $state("");
  let classifying = $state(false);
  let allCategories = $state([]);
  let activeCategoryDropdown = $state(null);
  let categoryInputText = $state({});

  // Step 4: done
  let savedCount = $state(0);

  const dateFormats = [
    { value: "%Y-%m-%d", label: "YYYY-MM-DD" },
    { value: "%d-%m-%Y", label: "DD-MM-YYYY" },
    { value: "%m-%d-%Y", label: "MM-DD-YYYY" },
    { value: "%d/%m/%Y", label: "DD/MM/YYYY" },
    { value: "%m/%d/%Y", label: "MM/DD/YYYY" },
    { value: "%Y/%m/%d", label: "YYYY/MM/DD" },
    { value: "%d.%m.%Y", label: "DD.MM.YYYY" },
  ];

  let headerRow = $derived(previewRows.length > 0 ? previewRows[0] : []);
  let dataRows = $derived(previewRows.length > 1 ? previewRows.slice(1) : []);
  let nonDuplicateRows = $derived(classifiedRows.filter((r) => !r.is_duplicate && r.amount < 0));
  let incomeRows = $derived(classifiedRows.filter((r) => !r.is_duplicate && r.amount >= 0));
  let duplicateRows = $derived(classifiedRows.filter((r) => r.is_duplicate));

  let dbClassified = $derived(nonDuplicateRows.filter(r => r._originalSource === "Database"));
  let llmClassified = $derived(nonDuplicateRows.filter(r => r._originalSource === "Llm"));
  let unclassified = $derived(nonDuplicateRows.filter(r => !r._originalSource));

  // Derived column indices from roles
  let titleCol = $derived(findColByRole("title"));
  let amountCol = $derived(findColByRole("amount"));
  let dateCol = $derived(findColByRole("date"));
  let mappingComplete = $derived(titleCol != null && amountCol != null && dateCol != null);

  function findColByRole(role) {
    const entry = Object.entries(columnRoles).find(([_, r]) => r === role);
    return entry ? Number(entry[0]) : null;
  }

  function assignRole(colIndex, role) {
    const newRoles = { ...columnRoles };
    for (const [key, val] of Object.entries(newRoles)) {
      if (val === role) delete newRoles[key];
    }
    newRoles[colIndex] = role;
    columnRoles = newRoles;
    activePopover = null;
    if (role === "date") autoDetectDateFormat(colIndex);
  }

  function unassignRole(colIndex) {
    const newRoles = { ...columnRoles };
    delete newRoles[colIndex];
    columnRoles = newRoles;
    activePopover = null;
  }

  function autoDetectDateFormat(colIndex) {
    const values = dataRows
      .slice(0, 3)
      .map(row => row[colIndex]?.trim())
      .filter(Boolean);
    if (values.length === 0) return;

    for (const fmt of dateFormats) {
      const allMatch = values.every(v => testDateFormat(v, fmt.value));
      if (allMatch) {
        dateFormat = fmt.value;
        return;
      }
    }
  }

  function testDateFormat(value, format) {
    if (!value) return false;
    const v = value.trim();
    const patterns = {
      "%Y-%m-%d": /^\d{4}-\d{1,2}-\d{1,2}$/,
      "%d-%m-%Y": /^\d{1,2}-\d{1,2}-\d{4}$/,
      "%m-%d-%Y": /^\d{1,2}-\d{1,2}-\d{4}$/,
      "%d/%m/%Y": /^\d{1,2}\/\d{1,2}\/\d{4}$/,
      "%m/%d/%Y": /^\d{1,2}\/\d{1,2}\/\d{4}$/,
      "%Y/%m/%d": /^\d{4}\/\d{1,2}\/\d{1,2}$/,
      "%d.%m.%Y": /^\d{1,2}\.\d{1,2}\.\d{4}$/,
    };
    return patterns[format]?.test(v) ?? false;
  }

  function tryParseAmount(value) {
    if (!value) return null;
    let v = value.trim().replace(/[^\d.,\-+]/g, "");
    if (!v) return null;
    // European: 1.234,56 -> 1234.56
    if (/^\d{1,3}(\.\d{3})*,\d{1,2}$/.test(v)) {
      v = v.replace(/\./g, "").replace(",", ".");
    } else {
      v = v.replace(",", ".");
    }
    const n = parseFloat(v);
    return isFinite(n) ? n : null;
  }

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
      const newRoles = {};
      if (headerRow.length > 0) {
        for (let i = 0; i < headerRow.length; i++) {
          const h = headerRow[i].toLowerCase();
          if (h.includes("title") || h.includes("description") || h.includes("name") ||
              h.includes("merchant") || h.includes("opis") || h.includes("tytuł"))
            newRoles[i] = "title";
          if (h.includes("amount") || h.includes("value") || h.includes("sum") || h.includes("kwota"))
            newRoles[i] = "amount";
          if (h.includes("date") || h.includes("data"))
            newRoles[i] = "date";
        }
      }
      columnRoles = newRoles;

      // Auto-detect date format if date column found
      const dc = findColByRole("date");
      if (dc != null) autoDetectDateFormat(dc);

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
    } catch (err) {
      inputError = `${err}`;
    }
  }

  async function goToReview() {
    if (!mappingComplete) {
      mappingError = "Assign all three columns (Title, Amount, Date) to continue.";
      return;
    }
    mappingError = "";
    classifying = true;
    await new Promise(r => requestAnimationFrame(() => requestAnimationFrame(r)));
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
      classifiedRows = rows.map((r) => ({ ...r, _editing: false, rule_pattern: r.title, _autoApplied: 0, _originalSource: r.source }));
      await loadCategories();
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

  async function loadCategories() {
    try {
      allCategories = await invoke("get_categories");
    } catch (err) {
      console.warn("Failed to load categories:", err);
      allCategories = [];
    }
  }

  function selectCategory(index, cat) {
    editCategory(index, cat);
    activeCategoryDropdown = null;
    categoryInputText = { ...categoryInputText, [index]: "" };
  }

  function removeCategory(index) {
    editCategory(index, "");
    activeCategoryDropdown = null;
  }

  function getCategoryFilteredSuggestions(index) {
    const text = (categoryInputText[index] || "").toLowerCase();
    if (!text) return allCategories;
    return allCategories.filter((c) => c.toLowerCase().includes(text));
  }

  function onCategoryKeydown(index, e) {
    if (e.key === "Enter") {
      const text = (categoryInputText[index] || "").trim();
      if (text) {
        selectCategory(index, text);
      }
      e.preventDefault();
    } else if (e.key === "Escape") {
      activeCategoryDropdown = null;
    }
  }

  function onCategoryInput(index, e) {
    categoryInputText = { ...categoryInputText, [index]: e.target.value };
    activeCategoryDropdown = index;
  }

  function onCategoryFocus(index) {
    activeCategoryDropdown = index;
  }

  function onCategoryBlur(index) {
    setTimeout(() => {
      if (activeCategoryDropdown === index) activeCategoryDropdown = null;
    }, 150);
  }

  let batchFilename = $derived(file ? file.name : "Pasted data");

  async function saveApproved() {
    reviewError = "";
    const toSave = nonDuplicateRows.map((r) => ({
      title: r.title,
      amount: r.amount,
      date: r.date,
      category: r.category,
      source: r.source,
      rule_pattern: r.rule_pattern !== r.title ? r.rule_pattern : null,
    }));

    try {
      savedCount = await invoke("bulk_save_expenses", {
        expenses: toSave,
        filename: batchFilename,
      });
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
    columnRoles = {};
    llmWarning = "";
    llmWarningDismissed = false;
    allCategories = [];
    activeCategoryDropdown = null;
    categoryInputText = {};
  }
</script>

{#snippet expenseCards(rows, showSource)}
  <div class="space-y-3">
    {#each rows as row}
      {@const origIndex = classifiedRows.indexOf(row)}
      <div class="bg-gray-800/40 rounded-lg p-4 border border-gray-800/50">
        <!-- Top row: Date | Title | Amount + confidence -->
        <div class="flex items-center gap-3 mb-3">
          <span class="text-sm text-gray-400 shrink-0">{row.date}</span>
          <span class="text-sm truncate flex-1">{row.title}</span>
          <span class="text-sm font-mono text-gray-100 shrink-0">{row.amount.toFixed(2)}</span>
          {#if showSource && row.confidence != null}
            {#if row.confidence >= 0.8}
              <span class="px-2 py-0.5 rounded text-xs bg-emerald-900/50 text-emerald-400 shrink-0">High</span>
            {:else if row.confidence >= 0.5}
              <span class="px-2 py-0.5 rounded text-xs bg-yellow-900/50 text-yellow-400 shrink-0">Medium</span>
            {:else}
              <span class="px-2 py-0.5 rounded text-xs bg-red-900/50 text-red-400 shrink-0">Low</span>
            {/if}
          {/if}
        </div>
        <!-- Bottom row: Category chip input -->
        <div class="relative">
          <span class="text-[10px] text-gray-500 uppercase tracking-wider mb-1 block">Category</span>
          {#if row.category}
            <span class="inline-flex items-center gap-1.5 px-3 py-1 rounded-full bg-emerald-900/40 text-emerald-400 border border-emerald-800/50 text-sm">
              {row.category}
              <button
                onclick={() => removeCategory(origIndex)}
                class="text-emerald-400 hover:text-emerald-300 text-base leading-none"
              >&times;</button>
            </span>
          {:else}
            <input
              type="text"
              value={categoryInputText[origIndex] || ""}
              oninput={(e) => onCategoryInput(origIndex, e)}
              onfocus={() => onCategoryFocus(origIndex)}
              onblur={() => onCategoryBlur(origIndex)}
              onkeydown={(e) => onCategoryKeydown(origIndex, e)}
              placeholder="Type category..."
              class="bg-gray-800 border border-gray-700 rounded-lg px-3 py-1.5
                     text-gray-100 placeholder-gray-600 focus:outline-none
                     focus:border-emerald-500 w-full text-sm"
            />
            {#if activeCategoryDropdown === origIndex}
              {@const suggestions = getCategoryFilteredSuggestions(origIndex)}
              {#if suggestions.length > 0}
                <div class="absolute z-30 mt-1 left-0 right-0 bg-gray-800 border border-gray-700 rounded-lg shadow-lg max-h-40 overflow-y-auto">
                  {#each suggestions as cat}
                    <button
                      onmousedown={() => selectCategory(origIndex, cat)}
                      class="w-full text-left px-3 py-1.5 text-sm text-gray-200 hover:bg-gray-700 transition-colors"
                    >{cat}</button>
                  {/each}
                </div>
              {/if}
            {/if}
          {/if}
        </div>
      </div>
    {/each}
  </div>
{/snippet}

<div>
  {#if classifying}
    <div class="fixed inset-0 bg-black/60 z-50 flex items-center justify-center">
      <div class="bg-gray-900 border border-gray-800 rounded-2xl p-10 flex flex-col items-center gap-4 shadow-2xl max-w-sm w-full mx-4">
        <div class="w-10 h-10 border-4 border-emerald-500/30 border-t-emerald-500 rounded-full animate-spin"></div>
        <p class="text-lg font-semibold text-gray-100">Classifying expenses...</p>
        <p class="text-sm text-gray-400">{dataRows.length} expenses — matching rules, then calling AI for the rest</p>
        <div class="w-full bg-gray-800 rounded-full h-1.5 overflow-hidden">
          <div class="h-full bg-emerald-500 rounded-full animate-progress"></div>
        </div>
      </div>
    </div>
  {/if}

  <h2 class="text-2xl font-bold mb-2">Expense Bulk Upload</h2>

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
        <div
          ondrop={handleFileDrop}
          ondragover={handleDragOver}
          onkeydown={(e) => {
            if (e.key === "Enter" || e.key === " ") {
              e.preventDefault();
              e.currentTarget.querySelector("input[type=file]")?.click();
            }
          }}
          role="button"
          tabindex="0"
          aria-label="Upload CSV file"
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
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div class="space-y-6" onclick={(e) => { if (!e.target.closest('[data-popover]')) activePopover = null; }}>
      <div class="bg-gray-900 rounded-xl p-6 border border-gray-800">
        <h3 class="text-lg font-semibold mb-1">
          Detected format: {parserName}
        </h3>
        <p class="text-sm text-gray-400 mb-4">
          Click on column name to assign the data type.
        </p>

        {#if llmWarning && !llmWarningDismissed}
          <div class="text-sm px-4 py-2 rounded-lg bg-amber-900/30 text-amber-400 border border-amber-800/50 mb-4 flex items-center justify-between gap-3">
            <span>{llmWarning}</span>
            <button
              onclick={() => (llmWarningDismissed = true)}
              class="text-amber-400 hover:text-amber-300 shrink-0 text-lg leading-none"
            >&times;</button>
          </div>
        {/if}

        <!-- Preview table with click-to-assign headers -->
        <div class="overflow-x-auto">
          <table class="w-full text-sm">
            <thead>
              <tr class="border-b border-gray-700">
                {#each headerRow as _col, i}
                  <th class="text-left px-3 py-2 relative" data-popover>
                    <button
                      onclick={(e) => { e.stopPropagation(); activePopover = activePopover === i ? null : i; }}
                      class="flex items-center gap-1.5 hover:text-gray-200 transition-colors
                        {columnRoles[i] === 'title' ? 'text-emerald-400' :
                         columnRoles[i] === 'amount' ? 'text-blue-400' :
                         columnRoles[i] === 'date' ? 'text-purple-400' : 'text-gray-400'}"
                    >
                      Column {i + 1}
                      {#if columnRoles[i] === "title"}
                        <span class="px-1.5 py-0.5 rounded text-[10px] bg-emerald-900/50 text-emerald-400">Title</span>
                      {:else if columnRoles[i] === "amount"}
                        <span class="px-1.5 py-0.5 rounded text-[10px] bg-blue-900/50 text-blue-400">Amount</span>
                      {:else if columnRoles[i] === "date"}
                        <span class="px-1.5 py-0.5 rounded text-[10px] bg-purple-900/50 text-purple-400">Date</span>
                      {/if}
                    </button>
                    {#if activePopover === i}
                      <div class="absolute z-20 mt-1 left-0 bg-gray-800 border border-gray-700 rounded-lg shadow-lg p-1 flex gap-1" data-popover>
                        <button onclick={(e) => { e.stopPropagation(); assignRole(i, "title"); }}
                          class="px-2 py-1 rounded text-xs hover:bg-emerald-900/50 text-emerald-400 whitespace-nowrap">
                          Title
                        </button>
                        <button onclick={(e) => { e.stopPropagation(); assignRole(i, "amount"); }}
                          class="px-2 py-1 rounded text-xs hover:bg-blue-900/50 text-blue-400 whitespace-nowrap">
                          Amount
                        </button>
                        <button onclick={(e) => { e.stopPropagation(); assignRole(i, "date"); }}
                          class="px-2 py-1 rounded text-xs hover:bg-purple-900/50 text-purple-400 whitespace-nowrap">
                          Date
                        </button>
                        {#if columnRoles[i]}
                          <button onclick={(e) => { e.stopPropagation(); unassignRole(i); }}
                            class="px-2 py-1 rounded text-xs hover:bg-gray-700 text-gray-400 whitespace-nowrap">
                            Clear
                          </button>
                        {/if}
                      </div>
                    {/if}
                  </th>
                {/each}
              </tr>
            </thead>
            <tbody>
              <tr class="border-b border-gray-800/50">
                {#each headerRow as cell, i}
                  <td class="px-3 py-2 text-gray-500 italic text-xs">{cell}</td>
                {/each}
              </tr>
              {#each dataRows.slice(0, 1) as row}
                <tr class="border-b border-gray-800/50">
                  {#each row as cell, i}
                    <td class="px-3 py-2">
                      {#if columnRoles[i] === "title"}
                        <span class="text-emerald-300">{cell}</span>
                      {:else if columnRoles[i] === "amount"}
                        {@const parsed = tryParseAmount(cell)}
                        <span class="text-blue-300">{cell}</span>
                        {#if parsed !== null}
                          <span class="text-xs text-blue-500 ml-1">({parsed.toFixed(2)})</span>
                        {:else}
                          <span class="text-xs text-red-400 ml-1">?</span>
                        {/if}
                      {:else if columnRoles[i] === "date"}
                        {@const valid = testDateFormat(cell, dateFormat)}
                        <span class="text-purple-300">{cell}</span>
                        {#if !valid}
                          <span class="text-xs text-red-400 ml-1">?</span>
                        {/if}
                      {:else}
                        <span class="text-gray-400">{cell}</span>
                      {/if}
                    </td>
                  {/each}
                </tr>
              {/each}
            </tbody>
          </table>
          {#if dataRows.length > 1}
            <p class="text-xs text-gray-500 mt-2 px-3">
              ...and {dataRows.length - 1} more rows
            </p>
          {/if}
        </div>
      </div>

      <!-- Date format selector (only when date column assigned) -->
      {#if dateCol != null}
        <div class="bg-gray-900 rounded-xl p-4 border border-gray-800 flex items-center gap-3">
          <span class="text-sm text-gray-400">Date format:</span>
          <select
            bind:value={dateFormat}
            class="bg-gray-800 border border-gray-700 rounded-lg px-3 py-1.5
                   text-gray-100 text-sm focus:outline-none focus:border-emerald-500"
          >
            {#each dateFormats as fmt}
              <option value={fmt.value}>{fmt.label}</option>
            {/each}
          </select>
          <span class="text-xs text-gray-500">(auto-detected)</span>
        </div>
      {/if}

      {#if !mappingComplete}
        <div class="text-sm px-4 py-2 rounded-lg bg-gray-800 text-gray-400">
          Assign all three columns (Title, Amount, Date) to continue.
        </div>
      {/if}

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
          disabled={classifying || !mappingComplete}
          class="flex-1 bg-emerald-600 hover:bg-emerald-500 disabled:opacity-50 text-white font-medium
                 py-3 rounded-xl transition-colors"
        >
          {#if classifying}
            Classifying...
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

      {#if dbClassified.length > 0}
        <div class="bg-gray-900 rounded-xl p-6 border border-gray-800">
          <h4 class="font-semibold mb-3 flex items-center gap-2">
            <span class="w-2 h-2 rounded-full bg-blue-400"></span>
            Classified by rules ({dbClassified.length})
          </h4>
          {@render expenseCards(dbClassified, false)}
        </div>
      {/if}

      {#if llmClassified.length > 0}
        <div class="bg-gray-900 rounded-xl p-6 border border-gray-800">
          <h4 class="font-semibold mb-3 flex items-center gap-2">
            <span class="w-2 h-2 rounded-full bg-purple-400"></span>
            Classified by AI ({llmClassified.length})
          </h4>
          {@render expenseCards(llmClassified, true)}
        </div>
      {/if}

      {#if unclassified.length > 0}
        <div class="bg-gray-900 rounded-xl p-6 border border-gray-800">
          <h4 class="font-semibold mb-3 flex items-center gap-2">
            <span class="w-2 h-2 rounded-full bg-yellow-400"></span>
            Needs your input ({unclassified.length})
          </h4>
          {@render expenseCards(unclassified, false)}
        </div>
      {/if}

      {#if incomeRows.length > 0}
        <div class="bg-gray-900 rounded-xl p-6 border border-cyan-900/50">
          <h4 class="font-semibold mb-1 text-cyan-400">
            Income ({incomeRows.length})
          </h4>
          <p class="text-sm text-gray-400 mb-4">
            These rows have no negative sign in the amount column — they are incomes, not expenses, and will not be saved.
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
                {#each incomeRows as row}
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
