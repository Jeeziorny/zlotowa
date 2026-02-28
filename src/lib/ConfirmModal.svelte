<script>
  import { focusTrap } from "./actions/focusTrap.js";

  let {
    title,
    confirmLabel = "Delete",
    confirmStyle = "danger",
    onconfirm,
    onclose,
    children,
  } = $props();

  let loading = $state(false);
  let error = $state("");

  async function handleConfirm() {
    loading = true;
    error = "";
    try {
      await onconfirm();
    } catch (err) {
      error = `${err}`;
    }
    loading = false;
  }

  function handleOverlayClick(e) {
    if (e.target === e.currentTarget && !loading) onclose();
  }

  function handleKeydown(e) {
    if (e.key === "Escape" && !loading) onclose();
  }

  let confirmClass = $derived(
    confirmStyle === "danger"
      ? "bg-red-600 hover:bg-red-500 text-white"
      : "bg-amber-500 hover:bg-amber-400 text-gray-950"
  );
</script>

<div
  class="fixed inset-0 bg-black/60 flex items-center justify-center z-50"
  role="presentation"
  onclick={handleOverlayClick}
  onkeydown={handleKeydown}
>
  <div
    class="bg-gray-900 border border-gray-800 rounded-xl p-6 max-w-sm w-full mx-4 shadow-xl"
    role="dialog"
    aria-modal="true"
    aria-labelledby="confirm-modal-title"
    tabindex="-1"
    use:focusTrap
    onclick={(e) => e.stopPropagation()}
  >
    <h3 id="confirm-modal-title" class="text-lg font-semibold text-gray-100 mb-3">{title}</h3>

    {@render children()}

    {#if error}
      <div class="text-sm bg-red-900/50 text-red-400 px-4 py-2 rounded-lg mb-3 mt-3">{error}</div>
    {/if}

    <div class="flex gap-3 justify-end mt-5">
      <button
        onclick={onclose}
        disabled={loading}
        class="bg-gray-800 hover:bg-gray-700 text-gray-300 px-4 py-2 rounded-lg
               text-sm transition-colors disabled:opacity-50"
      >
        Cancel
      </button>
      <button
        onclick={handleConfirm}
        disabled={loading}
        class="{confirmClass} disabled:opacity-50 px-4 py-2
               rounded-lg text-sm font-medium transition-colors"
      >
        {loading ? "Working..." : confirmLabel}
      </button>
    </div>
  </div>
</div>
