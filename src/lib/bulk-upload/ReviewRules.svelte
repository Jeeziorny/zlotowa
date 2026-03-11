<script>
  let { pendingRules, onback, onsave, onskip } = $props();

  // Internal editable state
  // eslint-disable-next-line -- pendingRules is set once before mount
  let rules = $state([...pendingRules].map(r => ({
    title: r.title,
    category: r.category,
    patternText: r.title,
    removed: false,
    editing: false,
  })));

  // Group active rules by category
  let activeRules = $derived(rules.filter(r => !r.removed));
  let grouped = $derived(() => {
    const groups = {};
    for (const rule of activeRules) {
      if (!groups[rule.category]) groups[rule.category] = [];
      groups[rule.category].push(rule);
    }
    return Object.entries(groups).sort((a, b) => a[0].localeCompare(b[0]));
  });

  function removeRule(rule) {
    rule.removed = true;
  }

  function restoreRule(rule) {
    rule.removed = false;
  }

  function startEdit(rule) {
    rule.editing = true;
  }

  function stopEdit(rule) {
    rule.editing = false;
  }

  function handleSave() {
    const toSave = activeRules.map(r => ({
      pattern_text: r.patternText,
      category: r.category,
    }));
    onsave(toSave);
  }

  let removedCount = $derived(rules.filter(r => r.removed).length);
  let showHelp = $state(false);
</script>

<div class="flex flex-col" style="height: calc(100vh - 12rem);">
  <div class="flex-1 overflow-y-auto min-h-0 max-w-3xl">
    <div class="mb-4">
      <div class="flex items-center gap-2 mb-1">
        <h3 class="text-lg font-semibold">Review Classification Rules</h3>
        <button
          onclick={() => showHelp = !showHelp}
          class="inline-flex items-center justify-center w-5 h-5 rounded-full border
                 text-xs cursor-help transition-colors
                 {showHelp ? 'border-amber-500 text-amber-400' : 'border-gray-600 text-gray-500 hover:border-gray-400 hover:text-gray-300'}"
          aria-label="Toggle help"
        >?</button>
      </div>
      {#if showHelp}
        <div class="bg-gray-800/60 border border-gray-700 rounded-lg px-4 py-3 mb-3 text-sm text-gray-300 space-y-2">
          <p>
            Rules let the app <strong class="text-gray-100">automatically categorize</strong> future expenses whose title <strong class="text-gray-100">contains</strong> the pattern text — it's not an exact match.
          </p>
          <p>
            For example, if you categorize <span class="font-mono text-amber-400/80">APTEKA OLIMPIA WROCLAW</span>
            as <span class="text-amber-400">Health</span>, a rule is created so that any future expense
            containing that text in its title is categorized instantly — no AI or manual work needed.
          </p>
          <p>
            You can <strong class="text-gray-100">trim the pattern</strong> to make it broader:
            shortening it to just <span class="font-mono text-amber-400/80">APTEKA</span> would also match
            titles like <span class="font-mono text-gray-400">APTEKA NOVA KRAKÓW</span> or
            <span class="font-mono text-gray-400">APTEKA POD ORŁEM GDAŃSK</span> in future imports.
          </p>
        </div>
      {/if}
    </div>

    {#if rules.length === 0}
      <div class="bg-gray-900 rounded-xl border border-gray-800 p-8 text-center mb-4">
        <p class="text-gray-400 mb-2">No new rules to review.</p>
        <p class="text-sm text-gray-500">All expenses were matched by existing rules and no categories were changed.</p>
      </div>
    {:else if activeRules.length === 0 && removedCount > 0}
      <div class="bg-gray-900 rounded-xl border border-gray-800 p-8 text-center mb-4">
        <p class="text-gray-400">All rules have been removed.</p>
      </div>
    {:else}
      {#each grouped() as [category, categoryRules]}
        <div class="mb-4">
          <div class="text-sm font-medium text-amber-400 mb-2 px-1">{category}</div>
          <div class="space-y-2">
            {#each categoryRules as rule}
              {@const idx = rules.indexOf(rule)}
              <div class="bg-gray-900 rounded-xl border border-gray-800 px-4 py-3">
                <div class="flex items-start gap-3">
                  <div class="flex-1 min-w-0">
                    <span class="text-[10px] text-gray-500 uppercase tracking-wider">Original title</span>
                    <p class="text-xs text-gray-400 mb-2 break-words">{rule.title}</p>
                    <span class="text-[10px] text-gray-500 uppercase tracking-wider">Rule pattern <span class="normal-case tracking-normal">(click to edit)</span></span>
                    {#if rule.editing}
                      <input
                        type="text"
                        bind:value={rules[idx].patternText}
                        onblur={() => stopEdit(rule)}
                        onkeydown={(e) => { if (e.key === 'Enter') stopEdit(rule); if (e.key === 'Escape') { rules[idx].patternText = rule.title; stopEdit(rule); } }}
                        class="w-full bg-gray-800 border border-gray-700 rounded px-2 py-1 text-sm
                               font-mono text-gray-200 focus:border-amber-500 focus:ring-1
                               focus:ring-amber-500 focus:outline-none"
                      />
                    {:else}
                      <button
                        onclick={() => startEdit(rule)}
                        class="text-sm font-mono text-gray-200 hover:text-amber-400 transition-colors
                               cursor-text text-left w-full break-words"
                        title="Click to edit pattern"
                      >
                        {rule.patternText}
                      </button>
                    {/if}
                  </div>
                  <button
                    onclick={() => removeRule(rule)}
                    class="text-gray-500 hover:text-red-400 p-1 transition-colors shrink-0 mt-0.5"
                    title="Remove rule"
                    aria-label="Remove rule"
                  >
                    <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
                    </svg>
                  </button>
                </div>
              </div>
            {/each}
          </div>
        </div>
      {/each}
    {/if}

    {#if removedCount > 0}
      <div class="mb-4">
        <button
          onclick={() => rules.forEach(r => r.removed = false)}
          class="text-sm text-gray-500 hover:text-gray-300 transition-colors"
        >
          Restore {removedCount} removed {removedCount === 1 ? 'rule' : 'rules'}
        </button>
      </div>
    {/if}
  </div>

  <div class="shrink-0 flex items-center justify-between border-t border-gray-800 pt-4 mt-4">
    <button
      onclick={onback}
      class="px-6 py-2.5 rounded-lg bg-gray-800 hover:bg-gray-700 text-gray-300
             font-medium transition-colors"
    >
      Back
    </button>
    <div class="flex items-center gap-3">
      {#if activeRules.length > 0}
        <button
          onclick={onskip}
          class="px-6 py-2.5 rounded-lg bg-gray-800 hover:bg-gray-700 text-gray-300
                 font-medium transition-colors border border-gray-700"
        >
          Skip All
        </button>
        <button
          onclick={handleSave}
          class="px-8 py-2.5 rounded-lg bg-amber-500 hover:bg-amber-400 text-gray-950
                 font-medium transition-colors"
        >
          Save {activeRules.length} {activeRules.length === 1 ? 'Rule' : 'Rules'}
        </button>
      {:else}
        <button
          onclick={onskip}
          class="px-8 py-2.5 rounded-lg bg-amber-500 hover:bg-amber-400 text-gray-950
                 font-medium transition-colors"
        >
          Continue
        </button>
      {/if}
    </div>
  </div>
</div>
