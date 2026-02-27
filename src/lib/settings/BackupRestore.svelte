<script>
  import { invoke } from "@tauri-apps/api/core";
  import { save, open } from "@tauri-apps/plugin-dialog";

  let backupMessage = $state("");
  let backupMessageType = $state("");
  let backingUp = $state(false);

  let restoreMessage = $state("");
  let restoreMessageType = $state("");
  let restoring = $state(false);

  async function doBackup() {
    backingUp = true;
    backupMessage = "";
    try {
      const path = await save({
        defaultPath: `4ccountant-backup-${new Date().toISOString().slice(0, 19).replace("T", "_").replaceAll(":", "-")}.json`,
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

  async function doRestore() {
    restoring = true;
    restoreMessage = "";
    try {
      const path = await open({
        filters: [{ name: "JSON", extensions: ["json"] }],
        multiple: false,
      });
      if (!path) {
        restoring = false;
        return;
      }
      const summary = await invoke("restore_database", { path });
      const parts = [];
      if (summary.expenses_inserted > 0 || summary.expenses_skipped > 0) {
        let s = `${summary.expenses_inserted} expenses`;
        if (summary.expenses_skipped > 0) s += ` (${summary.expenses_skipped} skipped)`;
        parts.push(s);
      }
      if (summary.rules_upserted > 0) parts.push(`${summary.rules_upserted} rules`);
      if (summary.cleanup_rules_upserted > 0) parts.push(`${summary.cleanup_rules_upserted} cleanup rules`);
      if (summary.budgets_inserted > 0 || summary.budgets_skipped > 0) {
        let s = `${summary.budgets_inserted} budgets`;
        if (summary.budgets_skipped > 0) s += ` (${summary.budgets_skipped} skipped)`;
        parts.push(s);
      }
      restoreMessage = parts.length > 0
        ? `Restored: ${parts.join(", ")}`
        : "Nothing to restore — all data already exists.";
      restoreMessageType = "success";
    } catch (err) {
      restoreMessage = `Restore failed: ${err}`;
      restoreMessageType = "error";
    }
    restoring = false;
  }
</script>

<div class="bg-gray-900 rounded-xl p-6 border border-gray-800">
  <h3 class="text-lg font-semibold mb-1">Backup & Restore</h3>
  <p class="text-sm text-gray-400 mb-4">
    Create a full backup of your expenses, classification rules, title cleanup rules, and budgets.
    Restore from a backup to recover data or migrate to another machine.
  </p>

  <div class="space-y-4">
    <div class="flex gap-3">
      <button
        onclick={doBackup}
        disabled={backingUp || restoring}
        class="flex-1 bg-emerald-600 hover:bg-emerald-500 disabled:opacity-50 text-white
               font-medium py-2.5 rounded-lg transition-colors"
      >
        {#if backingUp}
          <span class="inline-block w-4 h-4 border-2 border-white/30 border-t-white rounded-full animate-spin mr-2 align-middle"></span>
          Backing up...
        {:else}
          Create Backup
        {/if}
      </button>
      <button
        onclick={doRestore}
        disabled={backingUp || restoring}
        class="flex-1 bg-gray-800 hover:bg-gray-700 disabled:opacity-50 text-gray-300
               font-medium py-2.5 rounded-lg transition-colors"
      >
        {#if restoring}
          <span class="inline-block w-4 h-4 border-2 border-gray-500/30 border-t-gray-300 rounded-full animate-spin mr-2 align-middle"></span>
          Restoring...
        {:else}
          Restore from Backup
        {/if}
      </button>
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
