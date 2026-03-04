<script>
  import Autocomplete from "../Autocomplete.svelte";

  let { classifiedRows = $bindable(), allCategories, onback, onsave } = $props();

  let reviewError = $state("");

  let nonDuplicateRows = $derived(classifiedRows.filter((r) => !r.is_duplicate && r.amount < 0));
  let incomeRows = $derived(classifiedRows.filter((r) => !r.is_duplicate && r.amount >= 0));
  let duplicateRows = $derived(classifiedRows.filter((r) => r.is_duplicate));

  let dbClassified = $derived(nonDuplicateRows.filter(r => r._originalSource === "Database"));
  let llmClassified = $derived(nonDuplicateRows.filter(r => r._originalSource === "Llm"));
  let unclassified = $derived(nonDuplicateRows.filter(r => !r._originalSource));

  function editCategory(index, newCategory) {
    classifiedRows[index].category = newCategory;
    classifiedRows[index].source = "Manual";
  }

  function selectCategory(index, cat) {
    editCategory(index, cat);
  }

  function removeCategory(index) {
    editCategory(index, "");
  }

  async function doSave() {
    reviewError = "";
    try {
      await onsave(nonDuplicateRows);
    } catch (err) {
      reviewError = `${err}`;
    }
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
            <span class="inline-flex items-center gap-1.5 shrink-0">
              {#if row.confidence >= 0.8}
                <span class="px-2 py-0.5 rounded text-xs bg-emerald-900/50 text-emerald-400">High</span>
              {:else if row.confidence >= 0.5}
                <span class="px-2 py-0.5 rounded text-xs bg-yellow-900/50 text-yellow-400">Medium</span>
              {:else}
                <span class="px-2 py-0.5 rounded text-xs bg-red-900/50 text-red-400">Low</span>
              {/if}
            </span>
          {/if}
        </div>
        <!-- Bottom row: Category chip input -->
        <div class="relative">
          <span class="text-[10px] text-gray-500 uppercase tracking-wider mb-1 block">Category</span>
          {#if row.category}
            <span class="inline-flex items-center gap-1.5 px-3 py-1 rounded-full bg-amber-900/40 text-amber-400 border border-amber-800/50 text-sm">
              {row.category}
              <button
                onclick={() => removeCategory(origIndex)}
                class="text-amber-400 hover:text-amber-300 text-base leading-none"
                aria-label="Remove category {row.category}"
              >&times;</button>
            </span>
          {:else}
            <Autocomplete
              options={allCategories}
              placeholder="Type category..."
              onselect={(cat) => selectCategory(origIndex, cat)}
              inputClass="bg-gray-800 border border-gray-700 rounded-lg px-3 py-1.5
                          text-gray-100 placeholder-gray-600 focus:outline-none
                          focus:border-amber-500 w-full text-sm"
            />
          {/if}
        </div>
      </div>
    {/each}
  </div>
{/snippet}

<div class="flex flex-col" style="height: calc(100vh - 12rem);">
  <div class="bg-gray-900 rounded-xl p-6 border border-gray-800 shrink-0">
    <h3 class="text-lg font-semibold mb-1">Review Classifications</h3>
    <p class="text-sm text-gray-400">
      Edit categories as needed. Click a category to change it.
    </p>
  </div>

  <div class="flex-1 overflow-y-auto min-h-0 mt-6 space-y-6 pr-1 review-scroll">
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
          <span class="ml-auto inline-flex items-center gap-1.5 text-xs text-gray-400 font-normal">
            LLM confidence
            <span class="relative group">
              <span class="inline-flex items-center justify-center w-4 h-4 rounded-full border border-gray-600 text-[10px] text-gray-500 cursor-help">i</span>
              <span class="absolute bottom-full right-0 mb-1.5 w-56 px-3 py-2 rounded-lg bg-gray-800 border border-gray-700 text-xs text-gray-300 leading-relaxed shadow-lg opacity-0 pointer-events-none group-hover:opacity-100 transition-opacity z-40">
                How confident the AI is in its category pick.
                <strong class="text-emerald-400">High</strong> — clear, unambiguous title.
                <strong class="text-yellow-400">Medium</strong> — reasonable guess, worth checking.
                <strong class="text-red-400">Low</strong> — vague or cryptic title, review carefully.
              </span>
            </span>
          </span>
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
  </div>

  {#if reviewError}
    <div class="text-sm px-4 py-2 rounded-lg bg-red-900/50 text-red-400 mt-4 shrink-0">
      {reviewError}
    </div>
  {/if}

  <div class="flex gap-3 mt-4 shrink-0">
    <button
      onclick={onback}
      class="px-6 bg-gray-800 hover:bg-gray-700 text-gray-300 font-medium
             py-3 rounded-xl transition-colors"
    >
      Back
    </button>
    <button
      onclick={doSave}
      disabled={nonDuplicateRows.length === 0}
      class="flex-1 bg-amber-500 hover:bg-amber-400 disabled:bg-gray-700
             disabled:text-gray-500 text-gray-950 font-medium py-3 rounded-xl
             transition-colors"
    >
      Save {nonDuplicateRows.length} Expenses
    </button>
  </div>
</div>

<style>
  .review-scroll::-webkit-scrollbar {
    width: 6px;
  }
  .review-scroll::-webkit-scrollbar-track {
    background: transparent;
  }
  .review-scroll::-webkit-scrollbar-thumb {
    background: #374151;
    border-radius: 3px;
  }
  .review-scroll::-webkit-scrollbar-thumb:hover {
    background: #4b5563;
  }
</style>
