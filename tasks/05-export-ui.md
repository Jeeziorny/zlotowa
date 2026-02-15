# Task 5: Add Export UI and Tauri IPC Command

**Track:** B — Export Feature (2 of 2)
**Blocked by:** Task 4
**Blocks:** nothing

## Problem

The initial prompt (`initial_prompt.md` lines 77-78) requires users to export their expense
database to CSV with column selection. There is currently no UI, no Tauri command, and no
docs for this feature. The `CsvExporter` is implemented in Task 4.

## Current State

**Backend (after Task 4):**
- `CsvExporter` implements `Exporter` trait in `crates/core/src/exporters.rs`
- `ExportColumns` struct controls which columns are included
- `Database::get_all_expenses()` returns all expenses sorted by date DESC

**Frontend:**
- `src/lib/ExpenseList.svelte` shows all expenses in a table — natural place for an export button
- No export button or modal exists anywhere in the app

**Tauri:**
- `src-tauri/src/lib.rs` has 11 commands registered, none for export
- `src-tauri/Cargo.toml` has `tauri-plugin-shell` but no `tauri-plugin-dialog` (needed for save dialog)

## Scope

### 1. Add Tauri command `export_expenses`

In `src-tauri/src/lib.rs`:
```rust
#[derive(Serialize, Deserialize)]
pub struct ExportColumnsInput {
    pub date: bool,
    pub title: bool,
    pub amount: bool,
    pub category: bool,
    pub classification_source: bool,
}

#[tauri::command]
fn export_expenses(
    state: State<AppState>,
    columns: ExportColumnsInput,
) -> Result<Vec<u8>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let expenses = db.get_all_expenses().map_err(|e| e.to_string())?;

    let export_columns = ExportColumns {
        date: columns.date,
        title: columns.title,
        amount: columns.amount,
        category: columns.category,
        classification_source: columns.classification_source,
    };

    let exporter = CsvExporter;
    exporter.export(&expenses, &export_columns).map_err(|e| e.to_string())
}
```

Register in `invoke_handler!`.

### 2. File save approach

Two options:
- **Option A: Tauri dialog plugin** — use `tauri-plugin-dialog` for native Save As picker. Requires adding the plugin and writing to disk on the Rust side.
- **Option B: Return bytes to frontend** — return CSV bytes from the command, use the browser's `Blob` + `URL.createObjectURL` + `<a download>` trick to trigger a download.

**Recommended: Option B** — simpler, no plugin dependency, works well for the data sizes we deal with. Tauri webview supports the download pattern.

Frontend download helper:
```js
function downloadBlob(bytes, filename) {
    const blob = new Blob([new Uint8Array(bytes)], { type: "text/csv" });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = filename;
    a.click();
    URL.revokeObjectURL(url);
}
```

### 3. Add export UI to ExpenseList

Add an "Export CSV" button in the header area of `src/lib/ExpenseList.svelte`:

```svelte
<div class="flex items-center justify-between mb-6">
    <h2 class="text-2xl font-bold">Expenses</h2>
    {#if expenses.length > 0}
        <button onclick={() => showExportModal = true} class="...">
            Export CSV
        </button>
    {/if}
</div>
```

### 4. Export modal with column picker

A simple modal/panel with checkboxes:

```svelte
{#if showExportModal}
<div class="bg-gray-900 rounded-xl p-6 border border-gray-800 mb-6">
    <h3 class="text-lg font-semibold mb-3">Export Settings</h3>
    <div class="space-y-2 mb-4">
        <label class="flex items-center gap-2">
            <input type="checkbox" bind:checked={exportDate} /> Date
        </label>
        <label class="flex items-center gap-2">
            <input type="checkbox" bind:checked={exportTitle} /> Title
        </label>
        <label class="flex items-center gap-2">
            <input type="checkbox" bind:checked={exportAmount} /> Amount
        </label>
        <label class="flex items-center gap-2">
            <input type="checkbox" bind:checked={exportCategory} /> Category
        </label>
        <label class="flex items-center gap-2">
            <input type="checkbox" bind:checked={exportSource} /> Classification Source
        </label>
    </div>
    <div class="flex gap-3">
        <button onclick={doExport} class="...">Download CSV</button>
        <button onclick={() => showExportModal = false} class="...">Cancel</button>
    </div>
</div>
{/if}
```

State variables:
```js
let showExportModal = $state(false);
let exportDate = $state(true);
let exportTitle = $state(true);
let exportAmount = $state(true);
let exportCategory = $state(true);
let exportSource = $state(false);
let exportError = $state("");
```

### 5. Export function

```js
async function doExport() {
    try {
        const bytes = await invoke("export_expenses", {
            columns: {
                date: exportDate,
                title: exportTitle,
                amount: exportAmount,
                category: exportCategory,
                classification_source: exportSource,
            },
        });
        downloadBlob(bytes, `4ccountant-export-${new Date().toISOString().split('T')[0]}.csv`);
        showExportModal = false;
    } catch (err) {
        exportError = `Export failed: ${err}`;
    }
}
```

## Files to Change

| File | Change |
|---|---|
| `src-tauri/src/lib.rs` | Add `export_expenses` command + `ExportColumnsInput` type, register in handler |
| `src/lib/ExpenseList.svelte` | Add export button, modal with column picker, download logic |

## Test Scenarios

### Backend (Rust)

1. **`test_export_command_default_columns`** — call `export_expenses` with all columns true, verify non-empty byte result starting with header
2. **`test_export_command_empty_db`** — with no expenses, returns just the header line

### Frontend (manual UI tests)

3. **Export button visibility** — button hidden when no expenses, visible when expenses exist
4. **Modal open/close** — clicking "Export CSV" opens modal, Cancel closes it
5. **Column checkboxes** — all checked by default except "Classification Source"
6. **Download triggers** — clicking Download creates a file download with correct filename pattern `4ccountant-export-YYYY-MM-DD.csv`
7. **Exported file content** — open downloaded CSV, verify headers match checked columns, data rows match expenses
8. **Uncheck all columns** — should either disable Download button or export empty CSV (decide on UX)
9. **Large dataset** — export with 1000+ expenses, verify no timeout or truncation
10. **Special characters** — expense with commas/quotes in title exports correctly (CSV escaping from Task 4)

## Acceptance Criteria

- Export button appears on Expenses page when data exists
- Column picker allows selecting/deselecting columns
- Downloaded CSV is valid and openable in Excel/Google Sheets
- File is named with current date
- No external plugins required (pure Tauri + JS approach)
- Follows dark theme design conventions
