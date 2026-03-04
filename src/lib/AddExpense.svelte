<script>
  import { invoke } from "@tauri-apps/api/core";
  import { onMount, onDestroy } from "svelte";
  import DatePicker from "./DatePicker.svelte";
  import Autocomplete from "./Autocomplete.svelte";

  let title = $state("");
  let amount = $state("");
  let date = $state(new Date().toISOString().split("T")[0]);
  let category = $state("");
  let message = $state("");
  let messageType = $state("");
  let saving = $state(false);
  let messageClearTimer;

  let rulePattern = $state("");
  let showRulePattern = $state(false);

  let allCategories = $state([]);
  let suggestedCategory = $state("");

  onMount(async () => {
    try {
      allCategories = await invoke("get_categories");
    } catch (err) {
      console.error("Failed to load categories:", err);
    }
  });

  function showMessage(msg, type) {
    message = msg;
    messageType = type;
    clearTimeout(messageClearTimer);
    if (type === "success") {
      messageClearTimer = setTimeout(() => { message = ""; }, 3000);
    }
  }

  let debounceTimer;
  function onTitleInput(e) {
    title = e.target.value;
    clearTimeout(debounceTimer);
    debounceTimer = setTimeout(async () => {
      if (title.trim().length >= 3) {
        try {
          const suggestion = await invoke("suggest_category", { title });
          suggestedCategory = suggestion || "";
        } catch (err) { console.warn("Category suggestion failed:", err); suggestedCategory = ""; }
      } else {
        suggestedCategory = "";
      }
    }, 300);
  }

  async function submit() {
    if (!title || !amount || !date) {
      showMessage("Please fill in all required fields.", "error");
      return;
    }
    const parsedAmount = parseFloat(amount);
    if (isNaN(parsedAmount) || parsedAmount <= 0) {
      showMessage("Amount must be greater than zero.", "error");
      return;
    }

    saving = true;
    try {
      await invoke("add_expense", {
        input: {
          title,
          amount: parsedAmount,
          date,
          category: category || null,
          rule_pattern: (showRulePattern && rulePattern.trim()) ? rulePattern.trim() : null,
        },
      });

      showMessage("Expense added successfully!", "success");
      title = "";
      amount = "";
      category = "";
      rulePattern = "";
      showRulePattern = false;
      suggestedCategory = "";
      date = new Date().toISOString().split("T")[0];

      // Refresh categories list
      try {
        allCategories = await invoke("get_categories");
      } catch (err) { console.warn("Failed to refresh categories:", err); }
    } catch (err) {
      showMessage(`Error: ${err}`, "error");
    }
    saving = false;
  }

  onDestroy(() => {
    clearTimeout(debounceTimer);
    clearTimeout(messageClearTimer);
  });
</script>

<div>
  <h2 class="text-2xl font-bold mb-6">Add Expense</h2>

  <div class="max-w-lg bg-gray-900 rounded-xl p-6 border border-gray-800">
    <div class="space-y-4">
      <div>
        <label for="expense-date" class="block text-sm text-gray-400 mb-1">Date</label>
        <DatePicker id="expense-date" value={date} onchange={(d) => date = d} />
      </div>

      <div>
        <label class="block text-sm text-gray-400 mb-1" for="title">Title</label>
        <input
          id="title"
          type="text"
          value={title}
          oninput={onTitleInput}
          placeholder="e.g. Grocery store"
          class="w-full bg-gray-800 border border-gray-700 rounded-lg px-4 py-2.5
                 text-gray-100 placeholder-gray-600 focus:outline-none focus:border-amber-500"
        />
      </div>

      <div>
        {#if !showRulePattern}
          <button
            type="button"
            onclick={() => { showRulePattern = true; rulePattern = title; }}
            class="text-xs text-gray-500 hover:text-gray-400 transition-colors"
          >
            Set match keyword...
          </button>
        {:else}
          <label class="block text-sm text-gray-400 mb-1" for="rule-pattern">Match keyword</label>
          <input
            id="rule-pattern"
            type="text"
            bind:value={rulePattern}
            placeholder="e.g. LIDL"
            class="w-full bg-gray-800 border border-gray-700 rounded-lg px-4 py-2.5
                   text-gray-100 placeholder-gray-600 focus:outline-none focus:border-amber-500 text-sm"
          />
          <p class="text-xs text-gray-600 mt-1">
            Future expenses matching this keyword will be auto-categorized.
          </p>
        {/if}
      </div>

      <div>
        <label class="block text-sm text-gray-400 mb-1" for="amount">Amount</label>
        <input
          id="amount"
          type="number"
          step="0.01"
          min="0.01"
          bind:value={amount}
          placeholder="0.00"
          class="w-full bg-gray-800 border border-gray-700 rounded-lg px-4 py-2.5
                 text-gray-100 placeholder-gray-600 focus:outline-none focus:border-amber-500"
        />
      </div>

      <div>
        <label class="block text-sm text-gray-400 mb-1">Category (optional)</label>
        <Autocomplete
          bind:value={category}
          options={allCategories}
          placeholder="e.g. Groceries"
          inputClass="w-full bg-gray-800 border border-gray-700 rounded-lg px-4 py-2.5
                      text-gray-100 placeholder-gray-600 focus:outline-none focus:border-amber-500"
        />
        {#if suggestedCategory && !category}
          <button
            type="button"
            onclick={() => category = suggestedCategory}
            class="mt-1 text-xs text-amber-500 hover:text-amber-400"
          >
            Suggested: {suggestedCategory} (click to apply)
          </button>
        {/if}
      </div>

      <button
        onclick={submit}
        disabled={saving}
        class="w-full bg-amber-500 hover:bg-amber-400 disabled:bg-gray-700
               disabled:text-gray-500 text-gray-950 font-medium
               py-2.5 rounded-lg transition-colors"
      >
        {saving ? "Saving..." : "Add Expense"}
      </button>

      {#if message}
        <div
          class="text-sm px-4 py-2 rounded-lg {messageType === 'success'
            ? 'bg-emerald-900/50 text-emerald-400'
            : 'bg-red-900/50 text-red-400'}"
        >
          {message}
        </div>
      {/if}
    </div>
  </div>
</div>
