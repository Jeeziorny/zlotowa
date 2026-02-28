<script>
  import { invoke } from "@tauri-apps/api/core";
  import DatePicker from "../DatePicker.svelte";

  let { allCategories, averages, oncreated } = $props();

  let step = $state(1);

  // Step 1: Date range
  let startDate = $state("");
  let endDate = $state("");
  let dateError = $state("");
  let checking = $state(false);

  // Calendar events (optional, parsed from ICS in step 1)
  let calendarEvents = $state([]);
  let icsFile = $state(null);
  let icsLoading = $state(false);
  let icsMsg = $state("");

  // Step 2: Category budgets
  let categoryBudgets = $state([]);
  let showAddCategories = $state(false);
  let categoryError = $state("");
  let showEvents = $state(false);

  // Step 3: Creating
  let creating = $state(false);
  let createError = $state("");

  function getAverage(category) {
    const avg = averages.find((a) => a.category === category);
    return avg ? avg.average : 0;
  }

  async function parseIcsFile(file) {
    if (!startDate || !endDate) {
      icsMsg = "Set start and end dates first.";
      return;
    }
    icsLoading = true;
    icsMsg = "";
    try {
      const text = await file.text();
      const events = await invoke("parse_calendar_events", {
        icsContent: text,
        startDate,
        endDate,
      });
      calendarEvents = events.sort((a, b) =>
        a.start_date.localeCompare(b.start_date),
      );
      icsMsg = `${calendarEvents.length} event${calendarEvents.length !== 1 ? "s" : ""} found`;
    } catch (err) {
      icsMsg = `Error: ${err}`;
      calendarEvents = [];
    }
    icsLoading = false;
  }

  function handleFileDrop(event) {
    event.preventDefault();
    const files = event.dataTransfer?.files;
    if (files?.length > 0) {
      icsFile = files[0];
      parseIcsFile(files[0]);
    }
  }

  function handleFileSelect(event) {
    const f = event.target.files?.[0];
    if (f) {
      icsFile = f;
      parseIcsFile(f);
    }
  }

  function handleDragOver(event) {
    event.preventDefault();
  }

  async function validateDates() {
    dateError = "";
    if (!startDate || !endDate) {
      dateError = "Both start and end dates are required.";
      return;
    }
    if (startDate >= endDate) {
      dateError = "Start date must be before end date.";
      return;
    }
    checking = true;
    try {
      const overlaps = await invoke("check_budget_overlap", {
        startDate,
        endDate,
      });
      if (overlaps) {
        dateError = "This date range overlaps with an existing budget.";
        checking = false;
        return;
      }
      // Populate categories from averages
      categoryBudgets = averages.map((a) => ({
        category: a.category,
        average: a.average,
        amount: Math.round(a.average * 100) / 100,
      }));
      step = 2;
    } catch (err) {
      dateError = `${err}`;
    }
    checking = false;
  }

  function addCategory(cat) {
    if (categoryBudgets.find((c) => c.category === cat)) return;
    categoryBudgets = [
      ...categoryBudgets,
      { category: cat, average: 0, amount: 0 },
    ];
  }

  let remainingCategories = $derived(
    allCategories.filter(
      (cat) => !categoryBudgets.find((cb) => cb.category === cat),
    ),
  );

  function validateCategories() {
    categoryError = "";
    const invalid = categoryBudgets.filter(
      (c) => !c.amount || Number(c.amount) <= 0,
    );
    if (invalid.length > 0) {
      const names = invalid.map((c) => c.category).join(", ");
      categoryError = `Every category needs an amount > 0. Fix: ${names}`;
      return false;
    }
    return true;
  }

  async function createBudget() {
    creating = true;
    createError = "";
    try {
      const cats = categoryBudgets
        .filter((c) => c.amount > 0)
        .map((c) => ({ category: c.category, amount: Number(c.amount) }));
      await invoke("create_budget", {
        startDate,
        endDate,
        categories: cats,
      });
      oncreated();
    } catch (err) {
      createError = `${err}`;
    }
    creating = false;
  }
</script>

<div class="space-y-6">
  <!-- Step indicator -->
  <div class="flex items-center gap-2 mb-2">
    {#each [1, 2, 3] as s}
      <div
        class="flex items-center gap-2 {s <= step ? 'text-emerald-400' : 'text-gray-600'}"
      >
        <div
          class="w-7 h-7 rounded-full flex items-center justify-center text-xs font-bold border
          {s === step
            ? 'border-emerald-400 bg-emerald-400/10'
            : s < step
              ? 'border-emerald-600 bg-emerald-600/20'
              : 'border-gray-700'}"
        >
          {s}
        </div>
        <span class="text-sm {s === step ? 'font-medium' : ''}">
          {s === 1 ? "Dates" : s === 2 ? "Categories" : "Review"}
        </span>
      </div>
      {#if s < 3}
        <div class="flex-1 h-px {s < step ? 'bg-emerald-600' : 'bg-gray-800'}">
        </div>
      {/if}
    {/each}
  </div>

  {#if step === 1}
    <div class="bg-gray-900 rounded-xl p-6 border border-gray-800">
      <h3 class="text-lg font-semibold mb-4">Set Budget Period</h3>
      <div class="flex gap-4 items-end">
        <div>
          <label for="budget-start-date" class="text-xs text-gray-500 block mb-1">Start Date</label>
          <div class="w-48">
            <DatePicker
              value={startDate}
              onchange={(d) => (startDate = d)}
              id="budget-start-date"
            />
          </div>
        </div>
        <div>
          <label for="budget-end-date" class="text-xs text-gray-500 block mb-1">End Date</label>
          <div class="w-48">
            <DatePicker
              value={endDate}
              onchange={(d) => (endDate = d)}
              id="budget-end-date"
            />
          </div>
        </div>
        <button
          onclick={validateDates}
          disabled={checking}
          class="px-5 py-2 bg-emerald-600 hover:bg-emerald-500 disabled:opacity-50
                 text-white text-sm font-medium rounded-lg transition-colors"
        >
          {checking ? "Checking..." : "Next"}
        </button>
      </div>
      {#if dateError}
        <div class="text-sm text-red-400 mt-3">{dateError}</div>
      {/if}

      <!-- Optional ICS upload -->
      <div class="mt-6 pt-5 border-t border-gray-800">
        <p class="text-sm text-gray-400 mb-3">
          Optionally upload an .ics calendar file to see upcoming events while setting budgets.
        </p>
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
          aria-label="Upload iCal file"
          class="border border-dashed border-gray-700 rounded-lg p-4 text-center
                 hover:border-emerald-500/50 transition-colors cursor-pointer"
        >
          {#if icsLoading}
            <p class="text-gray-400 text-sm">Parsing...</p>
          {:else if icsFile}
            <p class="text-emerald-400 text-sm">{icsFile.name}</p>
          {:else}
            <p class="text-gray-500 text-sm">Drag & drop .ics file or
              <label class="text-emerald-400 hover:text-emerald-300 cursor-pointer underline">
                browse
                <input
                  type="file"
                  accept=".ics,.ical"
                  onchange={handleFileSelect}
                  class="hidden"
                />
              </label>
            </p>
          {/if}
        </div>
        {#if icsMsg}
          <div class="text-xs mt-2 {icsMsg.startsWith('Error') ? 'text-red-400' : 'text-emerald-400'}">
            {icsMsg}
          </div>
        {/if}
      </div>
    </div>
  {:else if step === 2}
    <!-- Upcoming events panel (if ICS was uploaded) -->
    {#if calendarEvents.length > 0}
      <div class="bg-gray-800/50 rounded-xl p-4 border border-gray-800">
        <button
          onclick={() => (showEvents = !showEvents)}
          class="flex items-center justify-between w-full text-sm text-gray-400 hover:text-gray-200 transition-colors"
        >
          <span>
            Upcoming Events
            <span class="ml-2 px-2 py-0.5 text-xs rounded-full bg-gray-700 text-gray-300">
              {calendarEvents.length}
            </span>
          </span>
          <span class="text-xs">{showEvents ? "Hide" : "Show"}</span>
        </button>
        {#if showEvents}
          <div class="mt-3 space-y-1.5 max-h-48 overflow-y-auto">
            {#each calendarEvents as event}
              <div class="flex gap-3 text-xs">
                <span class="text-gray-500 font-mono whitespace-nowrap">{event.start_date}</span>
                <span class="text-gray-300 truncate">{event.summary}</span>
                {#if event.location}
                  <span class="text-gray-600 truncate ml-auto">{event.location}</span>
                {/if}
              </div>
            {/each}
          </div>
        {/if}
      </div>
    {/if}

    <div class="bg-gray-900 rounded-xl p-6 border border-gray-800">
      <h3 class="text-lg font-semibold mb-4">Category Budgets</h3>
      <p class="text-sm text-gray-400 mb-4">
        Budget amounts are pre-filled from 3-month averages. Adjust as needed.
      </p>

      {#if categoryBudgets.length > 0}
        <table class="w-full text-sm mb-4">
          <thead>
            <tr class="border-b border-gray-700 text-gray-400">
              <th class="text-left px-3 py-2">Category</th>
              <th class="text-right px-3 py-2">Avg (3mo)</th>
              <th class="text-right px-3 py-2">Budget</th>
            </tr>
          </thead>
          <tbody>
            {#each categoryBudgets as cb, i}
              <tr class="border-b border-gray-800/50">
                <td class="px-3 py-2 text-gray-300">{cb.category}</td>
                <td
                  class="px-3 py-2 text-right text-gray-500 font-mono text-xs"
                >
                  {cb.average > 0 ? cb.average.toFixed(2) : "\u2014"}
                </td>
                <td class="px-3 py-2 text-right">
                  <input
                    type="number"
                    step="0.01"
                    min="0"
                    bind:value={categoryBudgets[i].amount}
                    class="w-28 bg-gray-800 border border-gray-700 rounded px-2 py-1
                           text-right text-gray-100 font-mono focus:outline-none
                           focus:border-emerald-500"
                  />
                </td>
              </tr>
            {/each}
          </tbody>
        </table>
      {:else}
        <p class="text-sm text-gray-500 mb-4">
          No categories with recent data. Add categories below.
        </p>
      {/if}

      {#if remainingCategories.length > 0}
        <div class="mb-4">
          <button
            onclick={() => (showAddCategories = !showAddCategories)}
            class="text-sm text-emerald-400 hover:text-emerald-300 transition-colors"
          >
            {showAddCategories ? "Hide" : "+ Add more categories"}
          </button>
          {#if showAddCategories}
            <div class="mt-2 flex flex-wrap gap-2">
              {#each remainingCategories as cat}
                <button
                  onclick={() => addCategory(cat)}
                  class="px-3 py-1 text-xs bg-gray-800 hover:bg-gray-700 text-gray-300
                         rounded-lg border border-gray-700 transition-colors"
                >
                  + {cat}
                </button>
              {/each}
            </div>
          {/if}
        </div>
      {/if}

      <div class="flex gap-3">
        <button
          onclick={() => (step = 1)}
          class="px-4 py-2 text-sm text-gray-400 hover:text-gray-200 transition-colors"
        >
          Back
        </button>
        <button
          onclick={() => {
            if (validateCategories()) step = 3;
          }}
          class="px-5 py-2 bg-emerald-600 hover:bg-emerald-500
                 text-white text-sm font-medium rounded-lg transition-colors"
        >
          Next
        </button>
      </div>

      {#if categoryError}
        <div class="text-sm text-red-400 mt-3">{categoryError}</div>
      {/if}
    </div>
  {:else if step === 3}
    <div class="bg-gray-900 rounded-xl p-6 border border-gray-800">
      <h3 class="text-lg font-semibold mb-4">Review Budget</h3>

      <div class="space-y-3 mb-6">
        <div class="flex justify-between text-sm">
          <span class="text-gray-400">Period</span>
          <span class="text-gray-200 font-mono">{startDate} — {endDate}</span>
        </div>
        <div class="flex justify-between text-sm">
          <span class="text-gray-400">Categories</span>
          <span class="text-gray-200"
            >{categoryBudgets.filter((c) => c.amount > 0).length}</span
          >
        </div>
        <div class="flex justify-between text-sm">
          <span class="text-gray-400">Total budgeted</span>
          <span class="text-gray-200 font-mono font-bold">
            {categoryBudgets
              .reduce((sum, c) => sum + Number(c.amount), 0)
              .toFixed(2)}
          </span>
        </div>
      </div>

      {#if categoryBudgets.filter((c) => c.amount > 0).length > 0}
        <div class="mb-6 space-y-1">
          {#each categoryBudgets.filter((c) => c.amount > 0) as cb}
            <div class="flex justify-between text-sm">
              <span class="text-gray-400">{cb.category}</span>
              <span class="text-gray-300 font-mono"
                >{Number(cb.amount).toFixed(2)}</span
              >
            </div>
          {/each}
        </div>
      {/if}

      {#if createError}
        <div class="text-sm text-red-400 mb-4">{createError}</div>
      {/if}

      <div class="flex gap-3">
        <button
          onclick={() => (step = 2)}
          class="px-4 py-2 text-sm text-gray-400 hover:text-gray-200 transition-colors"
        >
          Back
        </button>
        <button
          onclick={createBudget}
          disabled={creating}
          class="px-5 py-2 bg-emerald-600 hover:bg-emerald-500 disabled:opacity-50
                 text-white text-sm font-medium rounded-lg transition-colors"
        >
          {creating ? "Creating..." : "Create Budget"}
        </button>
      </div>
    </div>
  {/if}
</div>
