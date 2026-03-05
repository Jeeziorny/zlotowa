<script>
  import { focusTrap } from "./actions/focusTrap.js";

  let { onclose } = $props();

  let mod = typeof navigator !== "undefined" && /Mac|iPhone|iPad/.test(navigator.platform) ? "⌘" : "Ctrl+";

  let shortcuts = [
    { keys: `${mod}U`, label: "Import CSV" },
    { keys: `${mod}N`, label: "Add manually" },
    { keys: `${mod}K`, label: "Search expenses" },
    { keys: `${mod}1`, label: "Dashboard" },
    { keys: `${mod}2`, label: "Expenses" },
    { keys: `${mod}3`, label: "Categories" },
    { keys: `${mod}4`, label: "Budget" },
    { keys: "Esc", label: "Back / Close" },
  ];
</script>

<div
  class="fixed inset-0 bg-black/60 flex items-center justify-center z-50"
  role="presentation"
  onclick={(e) => { if (e.target === e.currentTarget) onclose(); }}
  onkeydown={(e) => { if (e.key === "Escape") onclose(); }}
>
  <div
    class="bg-gray-900 border border-gray-800 rounded-xl p-6 w-full max-w-sm mx-4"
    role="dialog"
    aria-modal="true"
    aria-labelledby="shortcuts-title"
    tabindex="-1"
    use:focusTrap
    onclick={(e) => e.stopPropagation()}
  >
    <div class="flex items-center justify-between mb-4">
      <h3 id="shortcuts-title" class="text-lg font-semibold">Keyboard Shortcuts</h3>
      <button
        onclick={onclose}
        class="text-gray-500 hover:text-gray-300 transition-colors"
        aria-label="Close"
      >&times;</button>
    </div>
    <div class="space-y-2">
      {#each shortcuts as s}
        <div class="flex items-center justify-between text-sm">
          <span class="text-gray-400">{s.label}</span>
          <kbd class="px-2 py-0.5 rounded bg-gray-800 border border-gray-700 text-gray-300 font-mono text-xs">{s.keys}</kbd>
        </div>
      {/each}
    </div>
  </div>
</div>
