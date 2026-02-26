<script>
  let { previewRows, parserName, llmWarning, classifying, onback, onnext } = $props();

  let columnRoles = $state({});
  let activePopover = $state(null);
  let dateFormat = $state("%Y-%m-%d");
  let mappingError = $state("");
  let llmWarningDismissed = $state(false);

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

  let titleCol = $derived(findColByRole("title"));
  let amountCol = $derived(findColByRole("amount"));
  let dateCol = $derived(findColByRole("date"));
  let mappingComplete = $derived(titleCol != null && amountCol != null && dateCol != null);

  // Auto-detect columns from header names on first render
  $effect(() => {
    if (headerRow.length > 0 && Object.keys(columnRoles).length === 0) {
      const newRoles = {};
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
      columnRoles = newRoles;

      const dc = findColByRole("date");
      if (dc != null) autoDetectDateFormat(dc);
    }
  });

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
    if (/^\d{1,3}(\.\d{3})*,\d{1,2}$/.test(v)) {
      v = v.replace(/\./g, "").replace(",", ".");
    } else {
      v = v.replace(",", ".");
    }
    const n = parseFloat(v);
    return isFinite(n) ? n : null;
  }

  async function submit() {
    if (!mappingComplete) {
      mappingError = "Assign all three columns (Title, Amount, Date) to continue.";
      return;
    }
    mappingError = "";
    try {
      await onnext({
        title_index: titleCol,
        amount_index: amountCol,
        date_index: dateCol,
        date_format: dateFormat,
      });
    } catch (err) {
      mappingError = `${err}`;
    }
  }
</script>

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
          aria-label="Dismiss warning"
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
      onclick={onback}
      class="px-6 bg-gray-800 hover:bg-gray-700 text-gray-300 font-medium
             py-3 rounded-xl transition-colors"
    >
      Back
    </button>
    <button
      onclick={submit}
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
