<script>
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";

  let { classifiedRows = $bindable(), onback, onnext } = $props();

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
    originalTitles = classifiedRows.map(r => r.title);
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
    for (let i = 0; i < classifiedRows.length; i++) {
      const current = classifiedRows[i].title;
      const replaced = current.replace(re, replace);
      const cleaned = normalizeWhitespace(replaced);
      if (cleaned !== current) {
        classifiedRows[i] = { ...classifiedRows[i], title: cleaned };
        count++;
      }
    }
    return count;
  }

  function handleApply() {
    const count = applyFindReplace(findText, replaceText, isRegex);
    if (applyError) return;
    lastApplyCount = count;
    appliedOps = [...appliedOps, { find: findText, replace: replaceText, is_regex: isRegex }];
    findText = "";
    replaceText = "";
  }

  function handleRecentClick(pair) {
    findText = pair.find;
    replaceText = pair.replace;
    isRegex = pair.is_regex;
  }

  function handleReapplyAll() {
    applyError = "";
    lastApplyCount = null;
    let totalCount = 0;
    for (const pair of recentCleanups) {
      const count = applyFindReplace(pair.find, pair.replace, pair.is_regex);
      if (applyError) return;
      totalCount += count;
      if (count > 0) {
        appliedOps = [...appliedOps, { find: pair.find, replace: pair.replace, is_regex: pair.is_regex }];
      }
    }
    lastApplyCount = totalCount;
  }

  function handleReset() {
    for (let i = 0; i < classifiedRows.length; i++) {
      classifiedRows[i] = { ...classifiedRows[i], title: originalTitles[i] };
    }
    appliedOps = [];
    lastApplyCount = null;
    applyError = "";
  }

  async function handleNext() {
    // Set display_title for rows where title differs from original
    for (let i = 0; i < classifiedRows.length; i++) {
      if (classifiedRows[i].title !== originalTitles[i]) {
        classifiedRows[i] = { ...classifiedRows[i], display_title: classifiedRows[i].title, title: originalTitles[i] };
      } else {
        classifiedRows[i] = { ...classifiedRows[i], display_title: null };
      }
    }

    // Persist applied operations as recent cleanups
    if (appliedOps.length > 0) {
      try {
        // Deduplicate: newest first, by (find, replace, is_regex)
        const seen = new Set();
        const merged = [];
        for (const op of [...appliedOps.reverse(), ...recentCleanups]) {
          const key = JSON.stringify([op.find, op.replace, op.is_regex]);
          if (!seen.has(key)) {
            seen.add(key);
            merged.push(op);
          }
        }
        const capped = merged.slice(0, 20);
        await invoke("save_config", { key: "recent_title_cleanups", value: JSON.stringify(capped) });
      } catch { /* best-effort */ }
    }

    onnext();
  }

  let modifiedCount = $derived(
    classifiedRows.filter((r, i) => r.title !== originalTitles[i]).length
  );
</script>

<div>
  <!-- Find-and-replace bar -->
  <div class="bg-gray-900 rounded-xl border border-gray-800 p-4 mb-4">
    <div class="flex items-end gap-3 flex-wrap">
      <div class="flex-1 min-w-48">
        <label for="cleanup-find" class="block text-xs text-gray-400 mb-1">Find</label>
        <input
          id="cleanup-find"
          type="text"
          bind:value={findText}
          placeholder="Text to find..."
          class="w-full bg-gray-800 border border-gray-700 rounded-lg px-3 py-2 text-sm
                 text-gray-100 font-mono placeholder-gray-600 focus:outline-none focus:border-emerald-500"
          onkeydown={(e) => e.key === "Enter" && handleApply()}
        />
      </div>
      <div class="flex-1 min-w-48">
        <label for="cleanup-replace" class="block text-xs text-gray-400 mb-1">Replace</label>
        <input
          id="cleanup-replace"
          type="text"
          bind:value={replaceText}
          placeholder="Leave empty to remove"
          class="w-full bg-gray-800 border border-gray-700 rounded-lg px-3 py-2 text-sm
                 text-gray-100 font-mono placeholder-gray-600 focus:outline-none focus:border-emerald-500"
          onkeydown={(e) => e.key === "Enter" && handleApply()}
        />
      </div>
      <label class="flex items-center gap-1.5 text-sm text-gray-400 cursor-pointer pb-2">
        <input type="checkbox" bind:checked={isRegex}
          class="accent-emerald-500" />
        Regex
      </label>
      <button
        onclick={handleApply}
        disabled={!findText}
        class="bg-emerald-600 hover:bg-emerald-500 disabled:bg-gray-700 disabled:text-gray-500
               text-white px-4 py-2 rounded-lg text-sm font-medium transition-colors"
      >
        Apply
      </button>
    </div>
    {#if applyError}
      <p class="mt-2 text-sm text-red-400">{applyError}</p>
    {/if}
    {#if lastApplyCount !== null && !applyError}
      <p class="mt-2 text-sm text-emerald-400">Replaced in {lastApplyCount} title{lastApplyCount !== 1 ? "s" : ""}</p>
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
              <span class="font-mono text-gray-300">{pair.find}</span>
              <span class="text-gray-600">&rarr;</span>
              <span class="font-mono text-gray-400">{pair.replace || "(remove)"}</span>
              {#if pair.is_regex}
                <span class="text-xs bg-gray-800 text-gray-500 px-1.5 py-0.5 rounded">regex</span>
              {/if}
            </button>
          {/each}
          <div class="px-4 py-2">
            <button
              onclick={handleReapplyAll}
              class="text-sm text-emerald-400 hover:text-emerald-300 transition-colors"
            >
              Re-apply all
            </button>
          </div>
        </div>
      {/if}
    </div>
  {/if}

  <!-- Expense table -->
  <div class="bg-gray-900 rounded-xl border border-gray-800 overflow-hidden mb-4">
    <div class="max-h-96 overflow-y-auto">
      <table class="w-full text-sm">
        <thead class="sticky top-0 bg-gray-900 border-b border-gray-800">
          <tr>
            <th class="text-left px-4 py-2.5 text-gray-400 font-medium w-28">Date</th>
            <th class="text-left px-4 py-2.5 text-gray-400 font-medium">Title</th>
            <th class="text-right px-4 py-2.5 text-gray-400 font-medium w-28">Amount</th>
          </tr>
        </thead>
        <tbody class="divide-y divide-gray-800/50">
          {#each classifiedRows as row, i}
            {@const isModified = row.title !== originalTitles[i]}
            <tr class="{isModified ? 'bg-emerald-950/20' : ''}">
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
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
  </div>

  <!-- Footer -->
  <div class="flex items-center justify-between">
    <button
      onclick={onback}
      class="text-gray-400 hover:text-emerald-400 text-sm inline-flex items-center gap-1 transition-colors"
    >
      <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7" />
      </svg>
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
        class="bg-emerald-600 hover:bg-emerald-500 text-white px-5 py-2 rounded-lg
               text-sm font-medium transition-colors"
      >
        {appliedOps.length > 0 ? "Next" : "Skip"}
      </button>
    </div>
  </div>
</div>
