<script>
  let { currentPage, onnavigate, showRules = false, onshowshortcuts = () => {} } = $props();

  let items = $derived([
    { id: "expenses", label: "Expenses" },
    { id: "categories", label: "Categories" },
    { id: "budget", label: "Budget" },
    ...(showRules ? [{ id: "rules", label: "Rules" }] : []),
  ]);
</script>

{#snippet expensesIcon()}
  <svg class="w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
    <path d="M9 5H7a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2V7a2 2 0 00-2-2h-2" />
    <rect x="9" y="3" width="6" height="4" rx="1" />
    <path d="M9 14h6M9 18h4" />
  </svg>
{/snippet}

{#snippet categoriesIcon()}
  <svg class="w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
    <path d="M20.59 13.41l-7.17 7.17a2 2 0 01-2.83 0L2 12V2h10l8.59 8.59a2 2 0 010 2.82z" />
    <circle cx="7" cy="7" r="1" />
  </svg>
{/snippet}

{#snippet budgetIcon()}
  <svg class="w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
    <circle cx="12" cy="12" r="10" />
    <path d="M12 2a10 10 0 0110 10" />
    <path d="M12 2v10l7.07 7.07" />
  </svg>
{/snippet}

{#snippet rulesIcon()}
  <svg class="w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
    <path d="M13 2L3 14h9l-1 8 10-12h-9l1-8z" />
  </svg>
{/snippet}

{#snippet settingsIcon()}
  <svg class="w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
    <circle cx="12" cy="12" r="3" />
    <path d="M19.4 15a1.65 1.65 0 00.33 1.82l.06.06a2 2 0 01-2.83 2.83l-.06-.06a1.65 1.65 0 00-1.82-.33 1.65 1.65 0 00-1 1.51V21a2 2 0 01-4 0v-.09a1.65 1.65 0 00-1-1.51 1.65 1.65 0 00-1.82.33l-.06.06a2 2 0 01-2.83-2.83l.06-.06a1.65 1.65 0 00.33-1.82 1.65 1.65 0 00-1.51-1H3a2 2 0 010-4h.09a1.65 1.65 0 001.51-1 1.65 1.65 0 00-.33-1.82l-.06-.06a2 2 0 012.83-2.83l.06.06a1.65 1.65 0 001.82.33h.09a1.65 1.65 0 001-1.51V3a2 2 0 014 0v.09a1.65 1.65 0 001 1.51 1.65 1.65 0 001.82-.33l.06-.06a2 2 0 012.83 2.83l-.06.06a1.65 1.65 0 00-.33 1.82v.09a1.65 1.65 0 001.51 1H21a2 2 0 010 4h-.09a1.65 1.65 0 00-1.51 1z" />
  </svg>
{/snippet}

<aside class="w-56 bg-gray-900 border-r border-gray-800 flex flex-col">
  <div class="p-6">
    <button onclick={() => onnavigate("dashboard")} class="flex items-center gap-2 text-xl font-bold text-amber-400 hover:text-amber-300 transition-colors">
      <svg class="w-8 h-8" version="1.2" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 32 32" width="32" height="32">
        <defs>
          <image width="26" height="26" id="img1" href="data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAABoAAAAaCAMAAACelLz8AAAAAXNSR0IB2cksfwAAADlQTFRF8Y8G8Y8GAAAAAAAAAAAAAAAA/+yO8Y8GAAAA/+yO8Y8G8ZAH8Y8G/+yN8ZEK+cNS8Y8G8Y8G8Y8GaKutPwAAABN0Uk5TAP9w10DThHFm/9v/mf//WzpoLCJ55BIAAAB+SURBVHicrdLZCoAgEAXQri0QEdX/f6QEFUHLZFZommLRPLjMUQdkEJ2BaxGRmXEQtITa+QlaWjGFE1udFC863Z9LZvWkQenkoAwYH0gUpRyDnK1boJz1YoRFKLoSaFG1JsmzqDkabtFeA/Rcy/wqH/3w8y/oW2+oxBEBfbgBN0c9JXwDi5AAAAAASUVORK5CYII="/>
          <image width="22" height="22" id="img2" href="data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAABYAAAAWBAMAAAA2mnEIAAAAAXNSR0IB2cksfwAAABJQTFRF/+yO/+yO8pUP8ZIK8ZAH8ZILecShyQAAAAZ0Uk5TAP//////enng/gAAAEtJREFUeJxjZGBgEARihvcMDIwobDALzCOKDWcC9eJhv2dk5H/PKAAR/8Bkd4ABwv7P+F7wA5T9QcD+IIz9npGfEa4em5mkuhOHfwFD6RjBvhO95AAAAABJRU5ErkJggg=="/>
        </defs>
        <use href="#img1" x="3" y="3"/>
        <use href="#img2" x="5" y="5"/>
      </svg>
      złotówa
    </button>
  </div>

  <div class="px-3 mb-2">
    <button
      onclick={() => onnavigate("expenses:add")}
      class="w-full bg-amber-500 hover:bg-amber-400 text-gray-950 rounded-lg py-2 text-sm font-medium transition-colors"
    >
      + Add
    </button>
  </div>

  <nav class="flex-1 px-3">
    {#each items as item}
      <button
        class="w-full text-left px-4 py-3 rounded-lg mb-1 flex items-center gap-3 transition-colors
          {currentPage === item.id
          ? 'border-l-2 border-amber-400 bg-amber-400/5 text-amber-400'
          : 'border-l-2 border-transparent text-gray-400 hover:bg-gray-800/50 hover:text-gray-200'}"
        onclick={() => onnavigate(item.id)}
      >
        {#if item.id === "expenses"}{@render expensesIcon()}
        {:else if item.id === "categories"}{@render categoriesIcon()}
        {:else if item.id === "budget"}{@render budgetIcon()}
        {:else if item.id === "rules"}{@render rulesIcon()}
        {/if}
        <span>{item.label}</span>
      </button>
    {/each}
  </nav>

  <div class="px-3 mb-3">
    <button
      class="w-full text-left px-4 py-3 rounded-lg flex items-center gap-3 transition-colors
        {currentPage === 'settings'
        ? 'border-l-2 border-amber-400 bg-amber-400/5 text-amber-400'
        : 'border-l-2 border-transparent text-gray-400 hover:bg-gray-800/50 hover:text-gray-200'}"
      onclick={() => onnavigate("settings")}
    >
      {@render settingsIcon()}
      <span>Settings</span>
    </button>
  </div>

  <div class="p-4 border-t border-gray-800/50 pt-3 text-xs text-gray-600 flex items-center justify-between">
    <span>złotówa v0.1.0</span>
    <button
      onclick={onshowshortcuts}
      class="w-5 h-5 rounded border border-gray-700 text-gray-500 hover:text-gray-300 hover:border-gray-600 transition-colors text-xs leading-none flex items-center justify-center"
      aria-label="Keyboard shortcuts"
      title="Keyboard shortcuts"
    >?</button>
  </div>
</aside>
