<script>
  let { value, onchange, id = undefined, inputClass = "" } = $props();

  let open = $state(false);
  let inputRef = $state(null);
  let pickerRef = $state(null);

  // Parse current value or default to today
  let viewDate = $state(parseDate(value) || new Date());

  let viewYear = $derived(viewDate.getFullYear());
  let viewMonth = $derived(viewDate.getMonth());

  let selectedDate = $derived(parseDate(value));

  function getToday() {
    return new Date();
  }
  let todayStr = $derived(fmt(getToday()));

  function parseDate(s) {
    if (!s) return null;
    const [y, m, d] = s.split("-").map(Number);
    if (!y || !m || !d) return null;
    const dt = new Date(y, m - 1, d);
    return isNaN(dt.getTime()) ? null : dt;
  }

  function fmt(d) {
    const y = d.getFullYear();
    const m = String(d.getMonth() + 1).padStart(2, "0");
    const day = String(d.getDate()).padStart(2, "0");
    return `${y}-${m}-${day}`;
  }

  let daysInMonth = $derived(new Date(viewYear, viewMonth + 1, 0).getDate());
  let firstDayOfWeek = $derived(new Date(viewYear, viewMonth, 1).getDay());
  // Monday-first: shift Sunday (0) to 6, Mon (1) to 0, etc.
  let startOffset = $derived((firstDayOfWeek + 6) % 7);

  function prevMonth() {
    viewDate = new Date(viewYear, viewMonth - 1, 1);
  }

  function nextMonth() {
    viewDate = new Date(viewYear, viewMonth + 1, 1);
  }

  function selectDay(day) {
    const d = new Date(viewYear, viewMonth, day);
    onchange(fmt(d));
    open = false;
  }

  function handleInputChange(e) {
    const val = e.target.value;
    if (/^\d{4}-\d{2}-\d{2}$/.test(val)) {
      onchange(val);
      const parsed = parseDate(val);
      if (parsed) viewDate = new Date(parsed.getFullYear(), parsed.getMonth(), 1);
    }
  }

  function handleClickOutside(e) {
    if (pickerRef && !pickerRef.contains(e.target) && inputRef && !inputRef.contains(e.target)) {
      open = false;
    }
  }

  const monthNames = ["January", "February", "March", "April", "May", "June",
    "July", "August", "September", "October", "November", "December"];
  const dayLabels = ["Mo", "Tu", "We", "Th", "Fr", "Sa", "Su"];
</script>

<svelte:window onclick={handleClickOutside} />

<div class="relative">
  <input
    bind:this={inputRef}
    {id}
    type="text"
    value={value}
    oninput={handleInputChange}
    onfocus={() => open = true}
    placeholder="YYYY-MM-DD"
    class={inputClass || "w-full bg-gray-800 border border-gray-700 rounded-lg px-4 py-2.5 text-gray-100 focus:outline-none focus:border-amber-500 font-mono"}
  />

  {#if open}
    <div
      bind:this={pickerRef}
      class="absolute z-30 mt-1 bg-gray-900 border border-gray-700 rounded-lg shadow-xl p-3 w-72"
    >
      <!-- Month/year nav -->
      <div class="flex items-center justify-between mb-2">
        <button
          type="button"
          onclick={prevMonth}
          aria-label="Previous month"
          class="px-2 py-1 rounded text-gray-400 hover:bg-gray-800 hover:text-gray-200 transition-colors"
        >&lt;</button>
        <span class="text-sm font-medium text-gray-200">
          {monthNames[viewMonth]} {viewYear}
        </span>
        <button
          type="button"
          onclick={nextMonth}
          aria-label="Next month"
          class="px-2 py-1 rounded text-gray-400 hover:bg-gray-800 hover:text-gray-200 transition-colors"
        >&gt;</button>
      </div>

      <!-- Day headers -->
      <div class="grid grid-cols-7 text-center text-xs text-gray-500 mb-1">
        {#each dayLabels as d}
          <span class="py-1">{d}</span>
        {/each}
      </div>

      <!-- Day grid -->
      <div class="grid grid-cols-7 text-center text-sm">
        {#each Array(startOffset) as _}
          <span></span>
        {/each}
        {#each Array(daysInMonth) as _, i}
          {@const day = i + 1}
          {@const dateStr = fmt(new Date(viewYear, viewMonth, day))}
          {@const isSelected = value === dateStr}
          {@const isToday = todayStr === dateStr}
          <button
            type="button"
            onclick={() => selectDay(day)}
            class="py-1.5 rounded transition-colors
              {isSelected
                ? 'bg-amber-500 text-gray-950 font-medium'
                : isToday
                  ? 'bg-gray-800 text-amber-400 font-medium'
                  : 'text-gray-300 hover:bg-gray-800'}"
          >
            {day}
          </button>
        {/each}
      </div>

      <!-- Today shortcut -->
      <div class="mt-2 pt-2 border-t border-gray-800">
        <button
          type="button"
          onclick={() => { const now = getToday(); onchange(fmt(now)); open = false; viewDate = new Date(now.getFullYear(), now.getMonth(), 1); }}
          class="text-xs text-amber-500 hover:text-amber-400 transition-colors"
        >
          Today
        </button>
      </div>
    </div>
  {/if}
</div>
