<script>
  let { year, month, onchange } = $props();

  const monthNames = [
    "January", "February", "March", "April", "May", "June",
    "July", "August", "September", "October", "November", "December",
  ];

  function prev() {
    let m = month - 1;
    let y = year;
    if (m < 1) { m = 12; y--; }
    onchange({ year: y, month: m });
  }

  function next() {
    let m = month + 1;
    let y = year;
    if (m > 12) { m = 1; y++; }
    onchange({ year: y, month: m });
  }
</script>

<div class="flex items-center gap-4">
  <button
    onclick={prev}
    class="w-9 h-9 flex items-center justify-center rounded-lg bg-gray-800
           hover:bg-gray-700 text-gray-400 hover:text-gray-200 transition-colors"
  >
    &lt;
  </button>
  <span class="text-lg font-semibold min-w-[180px] text-center">
    {monthNames[month - 1]} {year}
  </span>
  <button
    onclick={next}
    class="w-9 h-9 flex items-center justify-center rounded-lg bg-gray-800
           hover:bg-gray-700 text-gray-400 hover:text-gray-200 transition-colors"
  >
    &gt;
  </button>
</div>
