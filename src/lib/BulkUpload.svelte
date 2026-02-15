<script>
  let inputText = $state("");
  let file = $state(null);

  function handleFileDrop(event) {
    event.preventDefault();
    const files = event.dataTransfer?.files;
    if (files?.length > 0) {
      file = files[0];
    }
  }

  function handleFileSelect(event) {
    file = event.target.files?.[0] || null;
  }

  function handleDragOver(event) {
    event.preventDefault();
  }

  function submit() {
    // TODO: implement parsing and classification flow
    console.log("Bulk upload submitted", { inputText, file });
  }
</script>

<div>
  <h2 class="text-2xl font-bold mb-6">Bulk Upload</h2>

  <div class="max-w-2xl space-y-6">
    <!-- Text input -->
    <div class="bg-gray-900 rounded-xl p-6 border border-gray-800">
      <h3 class="text-lg font-semibold mb-3">Paste expense data</h3>
      <p class="text-sm text-gray-400 mb-3">
        Paste CSV content or HTML from your bank's expense page.
      </p>
      <textarea
        bind:value={inputText}
        rows="8"
        placeholder="Paste your expense data here..."
        class="w-full bg-gray-800 border border-gray-700 rounded-lg px-4 py-3
               text-gray-100 placeholder-gray-600 focus:outline-none focus:border-emerald-500
               font-mono text-sm resize-y"
      ></textarea>
    </div>

    <!-- File upload -->
    <div class="bg-gray-900 rounded-xl p-6 border border-gray-800">
      <h3 class="text-lg font-semibold mb-3">Or upload a file</h3>
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <div
        ondrop={handleFileDrop}
        ondragover={handleDragOver}
        role="button"
        tabindex="0"
        class="border-2 border-dashed border-gray-700 rounded-lg p-8 text-center
               hover:border-emerald-500/50 transition-colors cursor-pointer"
      >
        {#if file}
          <p class="text-emerald-400">{file.name}</p>
          <p class="text-xs text-gray-500 mt-1">{(file.size / 1024).toFixed(1)} KB</p>
        {:else}
          <p class="text-gray-400 mb-2">Drag & drop a .csv or .txt file here</p>
          <p class="text-sm text-gray-600">or</p>
        {/if}
        <label
          class="inline-block mt-3 px-4 py-2 bg-gray-800 rounded-lg text-sm
                 text-gray-300 hover:bg-gray-700 cursor-pointer transition-colors"
        >
          Browse files
          <input
            type="file"
            accept=".csv,.txt"
            onchange={handleFileSelect}
            class="hidden"
          />
        </label>
      </div>
    </div>

    <button
      onclick={submit}
      disabled={!inputText && !file}
      class="w-full bg-emerald-600 hover:bg-emerald-500 disabled:bg-gray-700
             disabled:text-gray-500 text-white font-medium py-3 rounded-xl
             transition-colors"
    >
      Process Expenses
    </button>
  </div>
</div>
