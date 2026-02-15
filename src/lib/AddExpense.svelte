<script>
  import { invoke } from "@tauri-apps/api/core";

  let title = $state("");
  let amount = $state("");
  let date = $state(new Date().toISOString().split("T")[0]);
  let category = $state("");
  let message = $state("");
  let messageType = $state("");

  async function submit() {
    if (!title || !amount || !date) {
      message = "Please fill in all required fields.";
      messageType = "error";
      return;
    }

    try {
      await invoke("add_expense", {
        input: {
          title,
          amount: parseFloat(amount),
          date,
          category: category || null,
        },
      });

      message = "Expense added successfully!";
      messageType = "success";
      title = "";
      amount = "";
      category = "";
    } catch (err) {
      message = `Error: ${err}`;
      messageType = "error";
    }
  }
</script>

<div>
  <h2 class="text-2xl font-bold mb-6">Add Expense</h2>

  <div class="max-w-lg bg-gray-900 rounded-xl p-6 border border-gray-800">
    <div class="space-y-4">
      <div>
        <label class="block text-sm text-gray-400 mb-1" for="date">Date</label>
        <input
          id="date"
          type="date"
          bind:value={date}
          class="w-full bg-gray-800 border border-gray-700 rounded-lg px-4 py-2.5
                 text-gray-100 focus:outline-none focus:border-emerald-500
                 [color-scheme:dark]"
        />
      </div>

      <div>
        <label class="block text-sm text-gray-400 mb-1" for="title">Title</label>
        <input
          id="title"
          type="text"
          bind:value={title}
          placeholder="e.g. Grocery store"
          class="w-full bg-gray-800 border border-gray-700 rounded-lg px-4 py-2.5
                 text-gray-100 placeholder-gray-600 focus:outline-none focus:border-emerald-500"
        />
      </div>

      <div>
        <label class="block text-sm text-gray-400 mb-1" for="amount">Amount</label>
        <input
          id="amount"
          type="number"
          step="0.01"
          bind:value={amount}
          placeholder="0.00"
          class="w-full bg-gray-800 border border-gray-700 rounded-lg px-4 py-2.5
                 text-gray-100 placeholder-gray-600 focus:outline-none focus:border-emerald-500"
        />
      </div>

      <div>
        <label class="block text-sm text-gray-400 mb-1" for="category">Category (optional)</label>
        <input
          id="category"
          type="text"
          bind:value={category}
          placeholder="e.g. Groceries"
          class="w-full bg-gray-800 border border-gray-700 rounded-lg px-4 py-2.5
                 text-gray-100 placeholder-gray-600 focus:outline-none focus:border-emerald-500"
        />
      </div>

      <button
        onclick={submit}
        class="w-full bg-emerald-600 hover:bg-emerald-500 text-white font-medium
               py-2.5 rounded-lg transition-colors"
      >
        Add Expense
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
