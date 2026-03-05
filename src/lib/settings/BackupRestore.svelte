<script>
  import { invoke } from "@tauri-apps/api/core";
  import { save, open } from "@tauri-apps/plugin-dialog";

  let backupMessage = $state("");
  let backupMessageType = $state("");
  let backingUp = $state(false);

  let restoreMessage = $state("");
  let restoreMessageType = $state("");
  let restoring = $state(false);

  // Two-step restore state
  let preview = $state(null);
  let restorePath = $state("");
  let confirmed = $state(false);
  let previewing = $state(false);

  async function doBackup() {
    backingUp = true;
    backupMessage = "";
    try {
      const path = await save({
        defaultPath: `zlotowa-backup-${new Date().toISOString().slice(0, 19).replace("T", "_").replaceAll(":", "-")}.json`,
        filters: [{ name: "JSON", extensions: ["json"] }],
      });
      if (!path) {
        backingUp = false;
        return;
      }
      await invoke("backup_database", { path });
      const filename = path.split("/").pop() || path.split("\\").pop() || path;
      backupMessage = `Backup saved to ${filename}`;
      backupMessageType = "success";
    } catch (err) {
      backupMessage = `Backup failed: ${err}`;
      backupMessageType = "error";
    }
    backingUp = false;
  }

  async function selectRestoreFile() {
    restoreMessage = "";
    preview = null;
    confirmed = false;
    previewing = true;
    try {
      const path = await open({
        filters: [{ name: "JSON", extensions: ["json"] }],
        multiple: false,
      });
      if (!path) {
        previewing = false;
        return;
      }
      restorePath = path;
      preview = await invoke("preview_backup", { path });
    } catch (err) {
      restoreMessage = `Failed to read backup: ${err}`;
      restoreMessageType = "error";
    }
    previewing = false;
  }

  function cancelRestore() {
    preview = null;
    restorePath = "";
    confirmed = false;
  }

  async function backupFirst() {
    await doBackup();
  }

  async function doRestore() {
    restoring = true;
    restoreMessage = "";
    try {
      const summary = await invoke("restore_database", { path: restorePath });
      const parts = [];
      if (summary.expenses_inserted > 0 || summary.expenses_skipped > 0) {
        let s = `${summary.expenses_inserted} expenses`;
        if (summary.expenses_skipped > 0) s += ` (${summary.expenses_skipped} skipped)`;
        parts.push(s);
      }
      if (summary.rules_upserted > 0) parts.push(`${summary.rules_upserted} rules`);
      if (summary.budgets_inserted > 0 || summary.budgets_skipped > 0) {
        let s = `${summary.budgets_inserted} budgets`;
        if (summary.budgets_skipped > 0) s += ` (${summary.budgets_skipped} skipped)`;
        parts.push(s);
      }
      restoreMessage = parts.length > 0
        ? `Restored: ${parts.join(", ")}`
        : "Nothing to restore — all data already exists.";
      restoreMessageType = "success";
      preview = null;
      restorePath = "";
      confirmed = false;
    } catch (err) {
      restoreMessage = `Restore failed: ${err}`;
      restoreMessageType = "error";
    }
    restoring = false;
  }

  function formatDate(iso) {
    if (!iso) return "Unknown";
    try {
      const d = new Date(iso);
      return d.toLocaleString();
    } catch {
      return iso;
    }
  }
</script>

<div class="bg-gray-900 rounded-xl p-6 border border-gray-800">
  <h3 class="text-lg font-semibold mb-1">Backup & Restore</h3>
  <p class="text-sm text-gray-400 mb-4">
    Create a full backup of your expenses, classification rules, and budgets.
    Restore from a backup to recover data or migrate to another machine.
  </p>

  <div class="space-y-4">
    <div class="flex gap-3">
      <button
        onclick={doBackup}
        disabled={backingUp || restoring || previewing}
        class="flex-1 bg-amber-500 hover:bg-amber-400 disabled:opacity-50 text-gray-950
               font-medium py-2.5 rounded-lg transition-colors"
      >
        {#if backingUp}
          <span class="inline-block w-4 h-4 border-2 border-white/30 border-t-white rounded-full animate-spin mr-2 align-middle"></span>
          Backing up...
        {:else}
          Create Backup
        {/if}
      </button>
      {#if !preview}
        <button
          onclick={selectRestoreFile}
          disabled={backingUp || restoring || previewing}
          class="flex-1 bg-gray-800 hover:bg-gray-700 disabled:opacity-50 text-gray-300
                 font-medium py-2.5 rounded-lg transition-colors"
        >
          {#if previewing}
            <span class="inline-block w-4 h-4 border-2 border-gray-500/30 border-t-gray-300 rounded-full animate-spin mr-2 align-middle"></span>
            Reading...
          {:else}
            Restore from Backup
          {/if}
        </button>
      {/if}
    </div>

    {#if backupMessage}
      <div
        class="text-sm px-4 py-2 rounded-lg {backupMessageType === 'success'
          ? 'bg-emerald-900/50 text-emerald-400'
          : 'bg-red-900/50 text-red-400'}"
      >
        {backupMessage}
      </div>
    {/if}

    <!-- Restore preview -->
    {#if preview}
      <div class="bg-gray-800 border border-gray-700 rounded-lg p-4">
        <h4 class="text-sm font-medium text-gray-200 mb-3">This backup contains:</h4>
        <div class="space-y-1.5 text-sm">
          <div class="flex justify-between">
            <span class="text-gray-400">Expenses</span>
            <span class="text-gray-200 font-mono">{preview.expense_count}</span>
          </div>
          <div class="flex justify-between">
            <span class="text-gray-400">Classification rules</span>
            <span class="text-gray-200 font-mono">{preview.rule_count}</span>
          </div>
          <div class="flex justify-between">
            <span class="text-gray-400">Categories</span>
            <span class="text-gray-200 font-mono">{preview.category_count}</span>
          </div>
          <div class="flex justify-between">
            <span class="text-gray-400">Budgets</span>
            <span class="text-gray-200 font-mono">{preview.budget_count}</span>
          </div>
          <div class="flex justify-between pt-1.5 border-t border-gray-700">
            <span class="text-gray-500">Created</span>
            <span class="text-gray-400 text-xs">{formatDate(preview.created_at)}</span>
          </div>
        </div>
      </div>

      <div class="bg-red-900/20 border border-red-800/50 rounded-lg p-4">
        <p class="text-red-300 font-medium text-sm">This will replace ALL current data</p>
        <p class="text-red-400/80 text-xs mt-1">
          This action cannot be undone. Your current expenses, rules, categories, and budgets will be permanently replaced.
        </p>
      </div>

      <button
        onclick={backupFirst}
        disabled={backingUp}
        class="w-full bg-gray-800 hover:bg-gray-700 disabled:opacity-50 text-gray-300
               text-sm py-2 rounded-lg transition-colors"
      >
        {backingUp ? "Backing up..." : "Back up current data first"}
      </button>

      <label class="flex items-center gap-2 cursor-pointer">
        <input
          type="checkbox"
          bind:checked={confirmed}
          class="rounded bg-gray-800 border-gray-600 text-amber-500 focus:ring-amber-500"
        />
        <span class="text-sm text-gray-300">I understand this will replace all my data</span>
      </label>

      <div class="flex gap-3">
        <button
          onclick={cancelRestore}
          class="flex-1 bg-gray-800 hover:bg-gray-700 text-gray-300
                 text-sm py-2.5 rounded-lg transition-colors"
        >
          Cancel
        </button>
        <button
          onclick={doRestore}
          disabled={!confirmed || restoring}
          class="flex-1 bg-red-600 hover:bg-red-500 disabled:opacity-40 disabled:hover:bg-red-600
                 text-white font-medium text-sm py-2.5 rounded-lg transition-colors"
        >
          {#if restoring}
            <span class="inline-block w-4 h-4 border-2 border-white/30 border-t-white rounded-full animate-spin mr-2 align-middle"></span>
            Restoring...
          {:else}
            Restore
          {/if}
        </button>
      </div>
    {/if}

    {#if restoreMessage}
      <div
        class="text-sm px-4 py-2 rounded-lg {restoreMessageType === 'success'
          ? 'bg-emerald-900/50 text-emerald-400'
          : 'bg-red-900/50 text-red-400'}"
      >
        {restoreMessage}
      </div>
    {/if}
  </div>
</div>
