<script>
  let { classifiedRows = $bindable(), allCategories, onback, onsave } = $props();

  let reviewError = $state("");
  let activeCategoryDropdown = $state(null);
  let categoryInputText = $state({});

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
                aria-label="Remove category {row.category}"
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
      onclick={onback}
      class="px-6 bg-gray-800 hover:bg-gray-700 text-gray-300 font-medium
             py-3 rounded-xl transition-colors"
    >
      Back
    </button>
    <button
      onclick={doSave}
      disabled={nonDuplicateRows.length === 0}
      class="flex-1 bg-emerald-600 hover:bg-emerald-500 disabled:bg-gray-700
             disabled:text-gray-500 text-white font-medium py-3 rounded-xl
             transition-colors"
    >
      Save {nonDuplicateRows.length} Expenses
    </button>
  </div>
</div>
