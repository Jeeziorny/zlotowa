<script>
  let { onconfirm, oncancel } = $props();
</script>

<div class="fixed inset-0 bg-black/60 flex items-center justify-center z-50"
     role="presentation"
     onclick={oncancel}
     onkeydown={(e) => { if (e.key === "Escape") oncancel(); }}>
  <div class="bg-gray-900 border border-gray-800 rounded-xl p-6 max-w-sm w-full mx-4 shadow-xl"
       role="dialog"
       aria-modal="true"
       aria-labelledby="leave-modal-title"
       tabindex="-1"
       onclick={(e) => e.stopPropagation()}>
    <h3 id="leave-modal-title" class="text-lg font-semibold text-gray-100 mb-2">Leave bulk upload?</h3>
    <p class="text-sm text-gray-400 mb-5">You'll lose your upload progress.</p>
    <div class="flex gap-3 justify-end">
      <button
        onclick={oncancel}
        class="bg-gray-800 hover:bg-gray-700 text-gray-300 px-4 py-2 rounded-lg
               text-sm transition-colors"
      >
        Stay
      </button>
      <button
        onclick={onconfirm}
        class="bg-red-600 hover:bg-red-500 text-white px-4 py-2 rounded-lg
               text-sm font-medium transition-colors"
      >
        Leave
      </button>
    </div>
  </div>
</div>
