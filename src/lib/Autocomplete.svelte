<script>
  let {
    value = $bindable(""),
    options = [],
    placeholder = "",
    onselect,
    class: className = "",
    inputClass = "",
    maxlength = undefined,
  } = $props();

  let open = $state(false);
  let highlighted = $state(-1);
  let wrapperEl = $state(null);

  let filtered = $derived(
    value
      ? options.filter((o) => o.toLowerCase().includes(value.toLowerCase()))
      : options
  );

  function handleFocus() {
    open = true;
  }

  function handleBlur(e) {
    if (wrapperEl?.contains(e.relatedTarget)) return;
    open = false;
    highlighted = -1;
  }

  function select(opt) {
    value = opt;
    open = false;
    highlighted = -1;
    onselect?.(opt);
  }

  function handleKeydown(e) {
    if (e.key === "ArrowDown") {
      if (!open) {
        open = true;
        highlighted = 0;
      } else {
        highlighted = Math.min(highlighted + 1, filtered.length - 1);
      }
      e.preventDefault();
    } else if (e.key === "ArrowUp") {
      highlighted = Math.max(highlighted - 1, -1);
      e.preventDefault();
    } else if (e.key === "Enter") {
      if (highlighted >= 0 && filtered[highlighted]) {
        select(filtered[highlighted]);
      } else if (filtered.length > 0) {
        select(filtered[0]);
      } else if (value.trim()) {
        const v = value.trim();
        open = false;
        highlighted = -1;
        onselect?.(v);
      }
      e.preventDefault();
    } else if (e.key === "Escape") {
      open = false;
      highlighted = -1;
      e.preventDefault();
    }
  }
</script>

<div class="relative {className}" bind:this={wrapperEl}>
  <input
    type="text"
    bind:value
    {placeholder}
    onfocus={handleFocus}
    onblur={handleBlur}
    onkeydown={handleKeydown}
    oninput={() => { open = true; highlighted = -1; }}
    {maxlength}
    class={inputClass}
  />
  {#if open && filtered.length > 0}
    <div
      class="absolute z-30 min-w-full w-max mt-1 bg-gray-800 border border-gray-700
             rounded-lg shadow-lg max-h-48 overflow-y-auto"
    >
      {#each filtered as opt, i}
        <button
          type="button"
          tabindex="-1"
          onmousedown={() => select(opt)}
          class="w-full text-left px-4 py-2 text-sm text-gray-200 cursor-pointer
                 transition-colors {i === highlighted ? 'bg-gray-700' : 'hover:bg-gray-700'}"
        >
          {opt}
        </button>
      {/each}
    </div>
  {/if}
</div>
