# Task 14: Fix CSV Export — Replace Browser Blob Download with Native File Dialog

**Track:** B — Export Pipeline (frontend + backend)
**Blocked by:** nothing
**Blocks:** nothing

## Problem

Clicking "Download CSV" in the export modal does nothing — no file is saved, no error is shown, no feedback of any kind. The user has no way to know whether the export succeeded or failed.

The root cause is `downloadBlob()` in `src/lib/ExpenseList.svelte:43-51`, which uses a browser-only download technique: it creates a temporary `<a>` element with a blob URL and programmatically clicks it. This pattern works in regular browsers but **silently fails in Tauri's webview** — the `.click()` call is a no-op, no error is thrown, and the `catch` block never fires.

After the silent failure, `showExportModal = false` (line 67) hides the modal, making it appear as if the export succeeded.

## Current State

### Frontend — `src/lib/ExpenseList.svelte`

The `doExport()` function (lines 53-72) calls the Rust backend to generate CSV bytes, then tries to trigger a browser download:

```js
async function doExport() {
  exporting = true;
  exportError = "";
  try {
    const bytes = await invoke("export_expenses", { columns: { ... } });
    downloadBlob(bytes, `4ccountant-export-${new Date().toISOString().split("T")[0]}.csv`);
    showExportModal = false;  // hides modal even though download silently failed
  } catch (err) {
    exportError = `Export failed: ${err}`;
  }
  exporting = false;
}
```

The broken `downloadBlob()` (lines 43-51):

```js
function downloadBlob(bytes, filename) {
  const blob = new Blob([new Uint8Array(bytes)], { type: "text/csv" });
  const url = URL.createObjectURL(blob);
  const a = document.createElement("a");
  a.href = url;
  a.download = filename;
  a.click();              // ← silent no-op in Tauri webview
  URL.revokeObjectURL(url);
}
```

### Backend — `src-tauri/src/lib.rs`

`export_expenses` command (lines 312-331) works correctly — it fetches expenses from DB, calls `CsvExporter.export()`, and returns `Vec<u8>`:

```rust
#[tauri::command]
fn export_expenses(
    state: State<AppState>,
    columns: ExportColumnsInput,
) -> Result<Vec<u8>, String> {
    // ... fetches expenses, builds CSV bytes, returns them
}
```

### Core — `crates/core/src/exporters.rs`

`CsvExporter` (lines 82-106) generates CSV bytes from expenses with configurable column selection. Well-tested (10 unit tests), no changes needed here.

### Dependencies — `src-tauri/Cargo.toml`

Currently has `tauri-plugin-shell = "2"` as the only Tauri plugin. No dialog or filesystem plugin installed.

### Capabilities — `src-tauri/capabilities/default.json`

Only has `core:default` and `shell:allow-open`. No dialog permissions.

### JS Dependencies — `package.json`

Only has `@tauri-apps/api` and `@tauri-apps/cli`. No dialog plugin JS package.

## Scope

### 1. Add `tauri-plugin-dialog` Rust dependency

**File:** `src-tauri/Cargo.toml`
**What:** Add `tauri-plugin-dialog = "2"` to `[dependencies]`.

### 2. Register the dialog plugin

**File:** `src-tauri/src/lib.rs`
**What:** Add `.plugin(tauri_plugin_dialog::init())` to the Tauri builder chain (line 410, next to the existing `tauri_plugin_shell::init()`).

### 3. Add dialog permissions to capabilities

**File:** `src-tauri/capabilities/default.json`
**What:** Add `"dialog:default"` to the `permissions` array.

### 4. Install the dialog JS package

**What:** Run `npm install @tauri-apps/plugin-dialog`.

### 5. Modify `export_expenses` to write to a file path

**File:** `src-tauri/src/lib.rs`
**What:** Change `export_expenses` to accept a `path: String` parameter in addition to `columns`. Instead of returning `Vec<u8>`, generate the CSV bytes and write them to the given path using `std::fs::write()`. Return `Result<(), String>` on success.

```rust
#[tauri::command]
fn export_expenses(
    state: State<AppState>,
    columns: ExportColumnsInput,
    path: String,
) -> Result<(), String> {
    // ... generate bytes same as before ...
    std::fs::write(&path, &bytes).map_err(|e| format!("Failed to write file: {}", e))
}
```

### 6. Rewrite `doExport()` to use native save dialog

**File:** `src/lib/ExpenseList.svelte`
**What:** Replace the `downloadBlob()` approach with Tauri's native save dialog:

1. Import `save` from `@tauri-apps/plugin-dialog`
2. Open a native save file dialog with a default filename and CSV filter
3. If the user picks a path, call the modified `export_expenses` with that path
4. If the user cancels the dialog, do nothing (don't show an error)
5. Remove the `downloadBlob()` function entirely

Sketch:

```js
import { save } from "@tauri-apps/plugin-dialog";

async function doExport() {
  exporting = true;
  exportError = "";
  try {
    const path = await save({
      defaultPath: `4ccountant-export-${new Date().toISOString().split("T")[0]}.csv`,
      filters: [{ name: "CSV", extensions: ["csv"] }],
    });
    if (!path) { exporting = false; return; } // user cancelled
    await invoke("export_expenses", {
      columns: { date: exportDate, title: exportTitle, ... },
      path,
    });
    // show success feedback (see sub-task 7)
  } catch (err) {
    exportError = `Export failed: ${err}`;
  }
  exporting = false;
}
```

### 7. Add success feedback after export

**File:** `src/lib/ExpenseList.svelte`
**What:** After a successful export, show the user a temporary success message with the file path (or just the filename). Add an `exportSuccess` state variable. Display it as a green banner below the export buttons (similar pattern to the existing `exportError` red banner). Auto-dismiss after a few seconds or on next modal toggle. Close the modal only after success, not immediately after invoking the command.

## Files to Change

| File | Change |
|---|---|
| `src-tauri/Cargo.toml` | Add `tauri-plugin-dialog = "2"` dependency |
| `src-tauri/src/lib.rs` | Register dialog plugin; modify `export_expenses` to accept `path` and write file |
| `src-tauri/capabilities/default.json` | Add `"dialog:default"` permission |
| `package.json` | Add `@tauri-apps/plugin-dialog` (via `npm install`) |
| `src/lib/ExpenseList.svelte` | Replace `downloadBlob()` with native save dialog + success feedback |

## Test Scenarios

### Backend (Rust)

No new unit tests needed — `CsvExporter` is already well-tested (10 tests in `crates/core/src/exporters.rs:108-322`). The `std::fs::write()` call is a single-line stdlib operation. The export logic itself is unchanged.

### Frontend (manual UI tests)

1. **Happy path:** Go to Expenses → click "Export CSV" → check columns → click "Download CSV" → OS save dialog appears → pick a location → file is saved → success message shows the filename → open the saved file and verify CSV content matches the selected columns.
2. **Cancel dialog:** Click "Download CSV" → OS save dialog appears → click Cancel → modal stays open, no error shown, no file written.
3. **Invalid path / permissions error:** Try to save to a read-only location (if possible) → error message appears in the red banner.
4. **Empty expenses:** With no expenses in DB, the "Export CSV" button should not appear (existing behavior, line 78: `{#if expenses.length > 0}`).
5. **All columns unchecked:** Uncheck all column checkboxes → export → file is created with empty-ish CSV (header line only) — matches existing core behavior per `test_export_no_columns_selected`.

## Acceptance Criteria

- Clicking "Download CSV" opens the OS-native file save dialog
- The save dialog defaults to filename `4ccountant-export-YYYY-MM-DD.csv` and filters to `.csv` files
- After saving, the CSV file exists at the chosen path with correct content
- A success message is visible to the user after a successful export
- Cancelling the save dialog does not produce an error or close the modal
- Write errors (permissions, disk full) are shown in the existing red error banner
- The `downloadBlob()` function is removed — no browser blob download code remains
- `tauri-plugin-dialog` is properly installed (Rust dep, JS package, capability permission, plugin registered)
