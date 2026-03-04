<script>
  /**
   * EmptyState — shared empty/zero-data placeholder.
   *
   * Props:
   *   title    — main message (required)
   *   subtitle — secondary hint (optional)
   *   icon     — "inbox" | "tag" | "search" | "wallet" | "list" (default: "inbox")
   *   cta      — { label: string, onclick: () => void } (optional)
   *   variant  — "page" (full card) | "widget" (compact inline) (default: "page")
   */
  let { title, subtitle = "", icon = "inbox", cta = null, variant = "page" } = $props();
</script>

{#if variant === "page"}
  <div class="bg-gray-900 rounded-xl p-12 border border-gray-800 text-center text-gray-500">
    <div class="flex justify-center mb-4 text-gray-700">
      {#if icon === "inbox"}
        <svg xmlns="http://www.w3.org/2000/svg" width="40" height="40" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
          <polyline points="22 12 16 12 14 15 10 15 8 12 2 12"/>
          <path d="M5.45 5.11 2 12v6a2 2 0 0 0 2 2h16a2 2 0 0 0 2-2v-6l-3.45-6.89A2 2 0 0 0 16.76 4H7.24a2 2 0 0 0-1.79 1.11z"/>
        </svg>
      {:else if icon === "tag"}
        <svg xmlns="http://www.w3.org/2000/svg" width="40" height="40" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
          <path d="M20.59 13.41l-7.17 7.17a2 2 0 0 1-2.83 0L2 12V2h10l8.59 8.59a2 2 0 0 1 0 2.82z"/>
          <line x1="7" y1="7" x2="7.01" y2="7"/>
        </svg>
      {:else if icon === "search"}
        <svg xmlns="http://www.w3.org/2000/svg" width="40" height="40" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
          <circle cx="11" cy="11" r="8"/>
          <line x1="21" y1="21" x2="16.65" y2="16.65"/>
        </svg>
      {:else if icon === "wallet"}
        <svg xmlns="http://www.w3.org/2000/svg" width="40" height="40" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
          <path d="M21 12V7H5a2 2 0 0 1 0-4h14v4"/>
          <path d="M3 5v14a2 2 0 0 0 2 2h16v-5"/>
          <path d="M18 12a2 2 0 0 0 0 4h4v-4z"/>
        </svg>
      {:else if icon === "list"}
        <svg xmlns="http://www.w3.org/2000/svg" width="40" height="40" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
          <line x1="8" y1="6" x2="21" y2="6"/>
          <line x1="8" y1="12" x2="21" y2="12"/>
          <line x1="8" y1="18" x2="21" y2="18"/>
          <line x1="3" y1="6" x2="3.01" y2="6"/>
          <line x1="3" y1="12" x2="3.01" y2="12"/>
          <line x1="3" y1="18" x2="3.01" y2="18"/>
        </svg>
      {/if}
    </div>
    <p class="text-lg mb-2">{title}</p>
    {#if subtitle}
      <p class="text-sm">{subtitle}</p>
    {/if}
    {#if cta}
      <button
        onclick={cta.onclick}
        class="mt-4 px-4 py-2 text-sm bg-amber-500 hover:bg-amber-400 text-gray-950
               font-medium rounded-lg transition-colors"
      >
        {cta.label}
      </button>
    {/if}
  </div>
{:else}
  <p class="text-sm text-gray-500">{title}</p>
{/if}
