<script>
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";
  import { MAX_RECENT_CLEANUPS } from "../constants.js";

  let { parsedRows = $bindable(), onback, onnext } = $props();

  // Mode: "findReplace" or "extract"
  let mode = $state("findReplace");
  let showHelp = $state(false);

  // Find-and-replace inputs
  let findText = $state("");
  let replaceText = $state("");
  let isRegex = $state(false);
  let applyError = $state("");
  let lastApplyCount = $state(null);

  // Original titles for reset/diff
  let originalTitles = $state([]);

  // Recent cleanups from config
  let recentCleanups = $state([]);
  let showRecent = $state(false);

  // Applied operations log
  let appliedOps = $state([]);

  onMount(() => {
    // Snapshot original titles
    originalTitles = parsedRows.map(r => r.title);
    loadRecentCleanups();
  });

  async function loadRecentCleanups() {
    try {
      const json = await invoke("get_config", { key: "recent_title_cleanups" });
      if (json) {
        recentCleanups = JSON.parse(json);
        if (recentCleanups.length > 0) showRecent = true;
      }
    } catch { /* no recent cleanups */ }
  }

  function normalizeWhitespace(s) {
    return s.split(/\s+/).filter(Boolean).join(" ");
  }

  function applyFindReplace(find, replace, regex) {
    applyError = "";
    if (!find) {
      applyError = "Find field cannot be empty";
      return 0;
    }

    let re;
    try {
      re = regex ? new RegExp(find, "g") : new RegExp(find.replace(/[.*+?^${}()|[\]\\]/g, "\\$&"), "g");
    } catch (e) {
      applyError = `Invalid regex: ${e.message}`;
      return 0;
    }

    let count = 0;
    for (let i = 0; i < parsedRows.length; i++) {
      const current = parsedRows[i].title;
      const replaced = current.replace(re, replace);
      const cleaned = normalizeWhitespace(replaced);
      if (cleaned !== current) {
        parsedRows[i] = { ...parsedRows[i], title: cleaned };
        count++;
      }
    }
    return count;
  }

  function applyExtract(find, regex) {
    applyError = "";
    if (!find) {
      applyError = "Field cannot be empty";
      return 0;
    }

    let re;
    try {
      re = regex ? new RegExp(find) : new RegExp(find.replace(/[.*+?^${}()|[\]\\]/g, "\\$&"));
    } catch (e) {
      applyError = `Invalid regex: ${e.message}`;
      return 0;
    }

    let count = 0;
    for (let i = 0; i < parsedRows.length; i++) {
      const current = parsedRows[i].title;
      const match = current.match(re);
      if (match) {
        const extracted = normalizeWhitespace(match[0]);
        if (extracted !== current) {
          parsedRows[i] = { ...parsedRows[i], title: extracted };
          count++;
        }
      }
    }
    return count;
  }

  function handleApply() {
    let count;
    if (mode === "extract") {
      count = applyExtract(findText, isRegex);
    } else {
      count = applyFindReplace(findText, replaceText, isRegex);
    }
    if (applyError) return;
    lastApplyCount = count;
    appliedOps = [...appliedOps, { find: findText, replace: replaceText, is_regex: isRegex, mode }];
    findText = "";
    replaceText = "";
  }

  function handleRecentClick(pair) {
    mode = pair.mode || "findReplace";
    findText = pair.find;
    replaceText = pair.replace || "";
    isRegex = pair.is_regex;
  }

  function handleReapplyAll() {
    applyError = "";
    lastApplyCount = null;
    let totalCount = 0;
    for (const pair of recentCleanups) {
      let count;
      if (pair.mode === "extract") {
        count = applyExtract(pair.find, pair.is_regex);
      } else {
        count = applyFindReplace(pair.find, pair.replace, pair.is_regex);
      }
      if (applyError) return;
      totalCount += count;
      if (count > 0) {
        appliedOps = [...appliedOps, { find: pair.find, replace: pair.replace, is_regex: pair.is_regex, mode: pair.mode || "findReplace" }];
      }
    }
    lastApplyCount = totalCount;
  }

  function handleReset() {
    for (let i = 0; i < parsedRows.length; i++) {
      parsedRows[i] = { ...parsedRows[i], title: originalTitles[i] };
    }
    appliedOps = [];
    lastApplyCount = null;
    applyError = "";
  }

  async function handleNext() {
    // Persist applied operations as recent cleanups
    if (appliedOps.length > 0) {
      try {
        // Deduplicate: newest first, by (find, replace, is_regex)
        const seen = new Set();
        const merged = [];
        for (const op of [...appliedOps.reverse(), ...recentCleanups]) {
          const key = JSON.stringify([op.find, op.replace, op.is_regex, op.mode || "findReplace"]);
          if (!seen.has(key)) {
            seen.add(key);
            merged.push(op);
          }
        }
        const capped = merged.slice(0, MAX_RECENT_CLEANUPS);
        await invoke("save_config", { key: "recent_title_cleanups", value: JSON.stringify(capped) });
      } catch { /* best-effort */ }
    }

    onnext();
  }

  // Inline edit state
  let editingIndex = $state(null);
  let editingTitle = $state("");

  function startEdit(index) {
    editingIndex = index;
    editingTitle = parsedRows[index].title;
  }

  function saveEdit() {
    if (editingIndex == null) return;
    const cleaned = normalizeWhitespace(editingTitle);
    if (cleaned) {
      parsedRows[editingIndex] = { ...parsedRows[editingIndex], title: cleaned };
    }
    editingIndex = null;
    editingTitle = "";
  }

  function cancelEdit() {
    editingIndex = null;
    editingTitle = "";
  }

  function removeRow(index) {
    if (editingIndex === index) cancelEdit();
    parsedRows = parsedRows.filter((_, i) => i !== index);
    originalTitles = originalTitles.filter((_, i) => i !== index);
  }

  let modifiedCount = $derived(
    parsedRows.filter((r, i) => r.title !== originalTitles[i]).length
  );
</script>

<div class="flex flex-col" style="height: calc(100vh - 12rem);">
  <div class="flex-1 overflow-y-auto min-h-0">
    <!-- Mode toggle + help -->
    <div class="bg-gray-900 rounded-xl border border-gray-800 p-4 mb-4">
      <div class="flex items-center justify-between mb-3">
        <div class="flex rounded-lg bg-gray-800 p-0.5">
          <button
            onclick={() => mode = "findReplace"}
            class="px-3 py-1 rounded-md text-sm font-medium transition-colors
                   {mode === 'findReplace' ? 'bg-amber-500 text-gray-950' : 'text-gray-400 hover:text-gray-200'}"
          >
            Find & Replace
          </button>
          <button
            onclick={() => mode = "extract"}
            class="px-3 py-1 rounded-md text-sm font-medium transition-colors
                   {mode === 'extract' ? 'bg-amber-500 text-gray-950' : 'text-gray-400 hover:text-gray-200'}"
          >
            Extract
          </button>
        </div>
        <button
          onclick={() => showHelp = !showHelp}
          class="w-6 h-6 rounded-full border text-xs font-bold transition-colors
                 {showHelp ? 'border-amber-500 text-amber-400' : 'border-gray-600 text-gray-500 hover:border-gray-400 hover:text-gray-300'}"
          title="How does this work?"
        >?</button>
      </div>

      {#if showHelp}
        <div class="mb-3 bg-gray-800 rounded-lg border border-gray-700 p-3 text-sm text-gray-300 space-y-3">
          <div>
            <p class="font-medium text-gray-200 mb-1">Find & Replace</p>
            <p class="text-gray-400 mb-1.5">Finds text in titles and replaces it. Leave "Replace" empty to remove the matched text.</p>
            <div class="font-mono text-xs bg-gray-900 rounded px-2 py-1.5 space-y-0.5">
              <p><span class="text-gray-500">Before:</span> Payment LIDL 18,99PLN</p>
              <p><span class="text-gray-500">Find:</span> Payment <span class="text-gray-600">&rarr;</span> <span class="text-gray-500">Replace:</span> <span class="text-gray-600 italic">(empty)</span></p>
              <p><span class="text-gray-500">After:</span> <span class="text-emerald-400">LIDL 18,99PLN</span></p>
            </div>
          </div>
          <div>
            <p class="font-medium text-gray-200 mb-1">Extract</p>
            <p class="text-gray-400 mb-1.5">Keeps only the matched text, removing everything else. Titles without a match are left unchanged.</p>
            <div class="font-mono text-xs bg-gray-900 rounded px-2 py-1.5 space-y-0.5">
              <p><span class="text-gray-500">Before:</span> Payment twoja stara LIDL zaplacono 18,99PLN</p>
              <p><span class="text-gray-500">Keep only:</span> LIDL</p>
              <p><span class="text-gray-500">After:</span> <span class="text-emerald-400">LIDL</span></p>
            </div>
          </div>
        </div>
      {/if}

      <!-- Input fields -->
      <div class="flex items-end gap-3 flex-wrap">
        <div class="flex-1 min-w-48">
          <label for="cleanup-find" class="block text-xs text-gray-400 mb-1">
            {mode === "extract" ? "Keep only" : "Find"}
          </label>
          <input
            id="cleanup-find"
            type="text"
            bind:value={findText}
            placeholder={mode === "extract" ? "Text to keep (e.g. LIDL)..." : "Text to find..."}
            class="w-full bg-gray-800 border border-gray-700 rounded-lg px-3 py-2 text-sm
                   text-gray-100 font-mono placeholder-gray-600 focus:outline-none focus:border-amber-500"
            onkeydown={(e) => e.key === "Enter" && handleApply()}
          />
        </div>
        {#if mode === "findReplace"}
          <div class="flex-1 min-w-48">
            <label for="cleanup-replace" class="block text-xs text-gray-400 mb-1">Replace</label>
            <input
              id="cleanup-replace"
              type="text"
              bind:value={replaceText}
              placeholder="Leave empty to remove"
              class="w-full bg-gray-800 border border-gray-700 rounded-lg px-3 py-2 text-sm
                     text-gray-100 font-mono placeholder-gray-600 focus:outline-none focus:border-amber-500"
              onkeydown={(e) => e.key === "Enter" && handleApply()}
            />
          </div>
        {/if}
        <label class="flex items-center gap-1.5 text-sm text-gray-400 cursor-pointer pb-2">
          <input type="checkbox" bind:checked={isRegex}
            class="accent-amber-500" />
          Regex
        </label>
        <button
          onclick={handleApply}
          disabled={!findText}
          class="bg-amber-500 hover:bg-amber-400 disabled:bg-gray-700 disabled:text-gray-500
                 text-gray-950 px-4 py-2 rounded-lg text-sm font-medium transition-colors"
        >
          Apply
        </button>
      </div>
      {#if applyError}
        <p class="mt-2 text-sm text-red-400">{applyError}</p>
      {/if}
      {#if lastApplyCount !== null && !applyError}
        <p class="mt-2 text-sm text-emerald-400">
          {mode === "extract" ? "Extracted" : "Replaced"} in {lastApplyCount} title{lastApplyCount !== 1 ? "s" : ""}
        </p>
      {/if}
    </div>

    <!-- Recent cleanups -->
    {#if recentCleanups.length > 0}
      <div class="mb-4">
        <button
          onclick={() => showRecent = !showRecent}
          class="text-sm text-gray-400 hover:text-gray-300 flex items-center gap-1 transition-colors"
        >
          <svg class="w-3.5 h-3.5 transition-transform {showRecent ? 'rotate-90' : ''}"
            fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7" />
          </svg>
          Recent cleanups ({recentCleanups.length})
        </button>
        {#if showRecent}
          <div class="mt-2 bg-gray-900 rounded-xl border border-gray-800 divide-y divide-gray-800">
            {#each recentCleanups as pair}
              <button
                onclick={() => handleRecentClick(pair)}
                class="w-full text-left px-4 py-2.5 hover:bg-gray-800/50 transition-colors
                       flex items-center gap-2 text-sm"
              >
                {#if pair.mode === "extract"}
                  <span class="text-xs bg-gray-800 text-amber-500/70 px-1.5 py-0.5 rounded">extract</span>
                  <span class="font-mono text-gray-300">{pair.find}</span>
                {:else}
                  <span class="font-mono text-gray-300">{pair.find}</span>
                  <span class="text-gray-600">&rarr;</span>
                  <span class="font-mono text-gray-400">{pair.replace || "(remove)"}</span>
                {/if}
                {#if pair.is_regex}
                  <span class="text-xs bg-gray-800 text-gray-500 px-1.5 py-0.5 rounded">regex</span>
                {/if}
              </button>
            {/each}
            <div class="px-4 py-2">
              <button
                onclick={handleReapplyAll}
                class="text-sm text-amber-400 hover:text-amber-300 transition-colors"
              >
                Re-apply all
              </button>
            </div>
          </div>
        {/if}
      </div>
    {/if}

    <!-- Expense table -->
    <div class="bg-gray-900 rounded-xl border border-gray-800 overflow-hidden">
      <div class="max-h-96 overflow-y-auto">
        <table class="w-full text-sm">
          <thead class="sticky top-0 bg-gray-900 border-b border-gray-800">
            <tr>
              <th class="text-left px-4 py-2.5 text-gray-400 font-medium w-28">Date</th>
              <th class="text-left px-4 py-2.5 text-gray-400 font-medium">Title</th>
              <th class="text-right px-4 py-2.5 text-gray-400 font-medium w-28">Amount</th>
              <th class="w-10"></th>
            </tr>
          </thead>
          <tbody class="divide-y divide-gray-800/50">
            {#each parsedRows as row, i}
              {@const isModified = row.title !== originalTitles[i]}
              {#if editingIndex === i}
                <tr class="bg-gray-800/20">
                  <td class="px-4 py-2 text-gray-400 tabular-nums">{row.date}</td>
                  <td class="px-4 py-1.5" colspan="2">
                    <div class="flex items-center gap-2">
                      <input
                        type="text"
                        bind:value={editingTitle}
                        class="flex-1 bg-gray-800 border border-amber-500 rounded-lg px-3 py-1.5 text-sm
                               text-gray-100 font-mono focus:outline-none"
                        onkeydown={(e) => {
                          if (e.key === "Enter") saveEdit();
                          if (e.key === "Escape") cancelEdit();
                        }}
                      />
                      <!-- svelte-ignore element_invalid_self_closing_tag -->
                      <button onclick={saveEdit} class="text-amber-400 hover:text-amber-300 p-1 transition-colors" title="Save" aria-label="Save edit">
                        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
                        </svg>
                      </button>
                      <button onclick={cancelEdit} class="text-gray-500 hover:text-gray-300 p-1 transition-colors" title="Cancel" aria-label="Cancel edit">
                        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
                        </svg>
                      </button>
                    </div>
                  </td>
                  <td></td>
                </tr>
              {:else}
                <tr class="group {isModified ? 'bg-emerald-950/20' : ''}">
                  <td class="px-4 py-2 text-gray-400 tabular-nums">{row.date}</td>
                  <td class="px-4 py-2 text-gray-100" title={isModified ? `Original: ${originalTitles[i]}` : ""}>
                    {row.title}
                    {#if isModified}
                      <span class="ml-1 text-emerald-500 text-xs" title="Original: {originalTitles[i]}">*</span>
                    {/if}
                  </td>
                  <td class="px-4 py-2 text-right text-gray-300 tabular-nums">
                    {row.amount.toFixed(2)}
                  </td>
                  <td class="px-4 py-2">
                    <div class="flex gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
                      <button
                        onclick={() => startEdit(i)}
                        class="text-gray-500 hover:text-gray-300 p-1 transition-colors"
                        title="Edit title"
                        aria-label="Edit title"
                      >
                        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                            d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z" />
                        </svg>
                      </button>
                      <button
                        onclick={() => removeRow(i)}
                        class="text-gray-500 hover:text-red-400 p-1 transition-colors"
                        title="Remove"
                        aria-label="Remove expense"
                      >
                        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                            d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
                        </svg>
                      </button>
                    </div>
                  </td>
                </tr>
              {/if}
            {/each}
          </tbody>
        </table>
      </div>
    </div>
  </div>

  <!-- Footer -->
  <div class="shrink-0 flex items-center justify-between border-t border-gray-800 pt-4 mt-4">
    <button
      onclick={onback}
      class="px-6 py-2.5 rounded-lg bg-gray-800 hover:bg-gray-700 text-gray-300
             font-medium transition-colors"
    >
      Back
    </button>
    <div class="flex items-center gap-3">
      {#if modifiedCount > 0}
        <span class="text-sm text-gray-400">{modifiedCount} modified</span>
        <button
          onclick={handleReset}
          class="text-sm text-gray-400 hover:text-red-400 transition-colors"
        >
          Reset
        </button>
      {/if}
      <button
        onclick={handleNext}
        class="px-8 py-2.5 rounded-lg bg-amber-500 hover:bg-amber-400 text-gray-950
               font-medium transition-colors"
      >
        {appliedOps.length > 0 ? "Next" : "Skip"}
      </button>
    </div>
  </div>
</div>
