<script>
  let { onnext } = $props();

  let inputText = $state("");
  let file = $state(null);
  let inputError = $state("");

  function handleFileDrop(event) {
    event.preventDefault();
    const files = event.dataTransfer?.files;
    if (files?.length > 0) {
      file = files[0];
      readFile(files[0]);
    }
  }

  function handleFileSelect(event) {
    const f = event.target.files?.[0];
    if (f) {
      file = f;
      readFile(f);
    }
  }

  function handleDragOver(event) {
    event.preventDefault();
  }

  function readFile(f) {
    const reader = new FileReader();
    reader.onload = (e) => {
      inputText = e.target.result;
    };
    reader.readAsText(f);
  }

  async function submit() {
    if (!inputText.trim()) {
      inputError = "Please paste data or upload a file.";
      return;
    }
    inputError = "";
    try {
      await onnext({ text: inputText, filename: file ? file.name : "Pasted data" });
    } catch (err) {
      inputError = `${err}`;
    }
  }
</script>

<div class="max-w-2xl space-y-6">
  <div class="bg-gray-900 rounded-xl p-6 border border-gray-800">
    <h3 class="text-lg font-semibold mb-3">Paste expense data</h3>
    <p class="text-sm text-gray-400 mb-3">
      Paste CSV content from your bank's export.
    </p>
    <textarea
      bind:value={inputText}
      rows="10"
      placeholder="date,title,amount&#10;2024-01-15,Grocery Store,45.99&#10;2024-01-16,Gas Station,62.30"
      class="w-full bg-gray-800 border border-gray-700 rounded-lg px-4 py-3
             text-gray-100 placeholder-gray-600 focus:outline-none focus:border-amber-500
             font-mono text-sm resize-y"
    ></textarea>
  </div>

  <div class="bg-gray-900 rounded-xl p-6 border border-gray-800">
    <h3 class="text-lg font-semibold mb-3">Or upload a file</h3>
    <div
      ondrop={handleFileDrop}
      ondragover={handleDragOver}
      onkeydown={(e) => {
        if (e.key === "Enter" || e.key === " ") {
          e.preventDefault();
          e.currentTarget.querySelector("input[type=file]")?.click();
        }
      }}
      role="button"
      tabindex="0"
      aria-label="Upload CSV file"
      class="border-2 border-dashed border-gray-700 rounded-lg p-8 text-center
             hover:border-amber-500/50 transition-colors cursor-pointer"
    >
      {#if file}
        <p class="text-amber-400">{file.name}</p>
        <p class="text-xs text-gray-500 mt-1"
          >{(file.size / 1024).toFixed(1)} KB</p
        >
      {:else}
        <p class="text-gray-400 mb-2">
          Drag & drop a .csv or .txt file here
        </p>
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

  {#if inputError}
    <div class="text-sm px-4 py-2 rounded-lg bg-red-900/50 text-red-400">
      {inputError}
    </div>
  {/if}

  <button
    onclick={submit}
    disabled={!inputText.trim()}
    class="w-full bg-amber-500 hover:bg-amber-400 disabled:bg-gray-700
           disabled:text-gray-500 text-gray-950 font-medium py-3 rounded-xl
           transition-colors"
  >
    Next: Map Columns
  </button>
</div>
