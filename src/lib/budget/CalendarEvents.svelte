<script>
  import { invoke } from "@tauri-apps/api/core";

  let { year, month, events, onrefresh } = $props();

  let importing = $state(false);
  let importMsg = $state("");
  let file = $state(null);

  const monthNames = [
    "January", "February", "March", "April", "May", "June",
    "July", "August", "September", "October", "November", "December",
  ];

  function handleFileDrop(event) {
    event.preventDefault();
    const files = event.dataTransfer?.files;
    if (files?.length > 0) {
      file = files[0];
      importFile(files[0]);
    }
  }

  function handleFileSelect(event) {
    const f = event.target.files?.[0];
    if (f) {
      file = f;
      importFile(f);
    }
  }

  function handleDragOver(event) {
    event.preventDefault();
  }

  async function importFile(f) {
    importing = true;
    importMsg = "";
    try {
      const text = await f.text();
      const count = await invoke("import_calendar_events", {
        year, month,
        icsContent: text,
      });
      importMsg = `Imported ${count} event${count !== 1 ? "s" : ""} for ${monthNames[month - 1]} ${year}`;
      onrefresh();
    } catch (err) {
      importMsg = `Error: ${err}`;
    }
    importing = false;
  }
</script>

<div class="space-y-6">
  <!-- Upload area -->
  <div class="bg-gray-900 rounded-xl p-6 border border-gray-800">
    <h3 class="text-lg font-semibold mb-3">Import Calendar</h3>
    <p class="text-sm text-gray-400 mb-4">
      Upload an .ics file to import events for {monthNames[month - 1]} {year}.
      Re-importing replaces previous events.
    </p>

    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div
      ondrop={handleFileDrop}
      ondragover={handleDragOver}
      role="button"
      tabindex="0"
      class="border-2 border-dashed border-gray-700 rounded-lg p-8 text-center
             hover:border-emerald-500/50 transition-colors cursor-pointer"
    >
      {#if importing}
        <p class="text-gray-400">Importing...</p>
      {:else if file}
        <p class="text-emerald-400">{file.name}</p>
        <p class="text-xs text-gray-500 mt-1">{(file.size / 1024).toFixed(1)} KB</p>
      {:else}
        <p class="text-gray-400 mb-2">Drag & drop an .ics file here</p>
        <p class="text-sm text-gray-600">or</p>
      {/if}
      <label
        class="inline-block mt-3 px-4 py-2 bg-gray-800 rounded-lg text-sm
               text-gray-300 hover:bg-gray-700 cursor-pointer transition-colors"
      >
        Browse files
        <input
          type="file"
          accept=".ics,.ical"
          onchange={handleFileSelect}
          class="hidden"
        />
      </label>
    </div>

    {#if importMsg}
      <div class="text-sm mt-3 {importMsg.startsWith('Error') ? 'text-red-400' : 'text-emerald-400'}">
        {importMsg}
      </div>
    {/if}
  </div>

  <!-- Event list -->
  <div class="bg-gray-900 rounded-xl p-6 border border-gray-800">
    <h3 class="text-lg font-semibold mb-3">
      Events
      <span class="text-sm font-normal text-gray-500 ml-2">
        {events.length} event{events.length !== 1 ? "s" : ""} for {monthNames[month - 1]} {year}
      </span>
    </h3>

    {#if events.length > 0}
      <div class="overflow-x-auto">
        <table class="w-full text-sm">
          <thead>
            <tr class="border-b border-gray-700 text-gray-400">
              <th class="text-left px-3 py-2">Date</th>
              <th class="text-left px-3 py-2">Event</th>
              <th class="text-left px-3 py-2">Location</th>
            </tr>
          </thead>
          <tbody>
            {#each events as event}
              <tr class="border-b border-gray-800/50">
                <td class="px-3 py-2 text-gray-400 whitespace-nowrap">
                  {event.start_date}
                  {#if event.all_day}
                    <span class="text-xs text-gray-600 ml-1">all day</span>
                  {/if}
                </td>
                <td class="px-3 py-2 text-gray-300">{event.summary}</td>
                <td class="px-3 py-2 text-gray-500">{event.location || "—"}</td>
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
    {:else}
      <p class="text-sm text-gray-500">No events imported yet.</p>
    {/if}
  </div>
</div>
