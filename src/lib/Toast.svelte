<script>
  import { fly } from "svelte/transition";
  import { getToasts, removeToast } from "./stores/toast.svelte.js";
  import { getPrefersReducedMotion } from "./stores/reduced-motion.svelte.js";

  let toasts = $derived(getToasts());
  let duration = $derived(getPrefersReducedMotion() ? 0 : 200);
</script>

{#if toasts.length > 0}
  <div class="fixed top-4 left-1/2 -translate-x-1/2 z-50 pointer-events-none">
    {#each toasts as toast (toast.id)}
      <div
        class="pointer-events-auto flex items-center gap-3 px-4 py-3 rounded-lg shadow-lg border
               bg-gray-900 text-sm min-w-[280px] max-w-md
               {toast.type === 'success' ? 'border-l-4 border-l-emerald-500 border-gray-700' : ''}
               {toast.type === 'error' ? 'border-l-4 border-l-red-500 border-gray-700' : ''}
               {toast.type === 'info' ? 'border-l-4 border-l-amber-500 border-gray-700' : ''}"
        role="status"
        aria-live="polite"
        in:fly={{ y: -20, duration }}
        out:fly={{ y: -20, duration }}
      >
        <span class="flex-1 {toast.type === 'success' ? 'text-emerald-300' : ''} {toast.type === 'error' ? 'text-red-300' : ''} {toast.type === 'info' ? 'text-amber-300' : ''}">
          {toast.message}
        </span>
        <button
          onclick={() => removeToast(toast.id)}
          class="text-gray-500 hover:text-gray-300 text-lg leading-none shrink-0"
          aria-label="Dismiss"
        >&times;</button>
      </div>
    {/each}
  </div>
{/if}
