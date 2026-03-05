# Task 84: User-Selectable CSV Delimiter in Bulk Upload

**Track:** B — Bulk Upload (frontend + backend)
**Blocked by:** nothing
**Blocks:** nothing

## Problem

When uploading Polish bank CSVs like `historia.csv`, auto-detection fails with:

> Failed to parse file: Could not detect input format. Supported: CSV (comma, semicolon, tab delimited).

The root cause is a chain of two bugs in delimiter detection:

1. **Column-count consistency is all-or-nothing.** `detect_delimiter()` (`csv_parser.rs:9-41`) scores a delimiter by checking if *every* line has the same column count. A single inconsistent row (e.g. trailing comma giving 9 fields vs 8 in the header) scores that delimiter at 0. Real bank exports often have trailing delimiters or ragged rows.

2. **Tie-breaking picks `|` instead of `,`.** When all delimiters score 0, Rust's `max_by_key` returns the *last* equal element. Since the array is `[',', ';', '\t', '|']`, it picks `|`. With `|`, every line is 1 field, so `detect()` returns 0.0 — below the 0.3 threshold in `detect_parser()`, so no parser matches.

Even after fixing auto-detection, there will always be edge cases where it guesses wrong. The user should be able to override the delimiter manually.

## Current State

### Delimiter detection — `crates/core/src/parsers/csv_parser.rs:9-41`

```rust
fn detect_delimiter(input: &str) -> char {
    let first_lines: Vec<&str> = input.lines().take(5).collect();
    let delimiters = [',', ';', '\t', '|'];
    // ...scores each delimiter: column_count if ALL rows match, else 0
    // max_by_key picks last element on tie → '|' when all score 0
}
```

Used by `detect()` (:68), `preview_rows()` (:94), and `parse()` (:111) — each call re-detects the delimiter independently.

### Tauri commands — `src-tauri/src/lib.rs`

**`preview_csv`** (line 267): Takes `input: String`, calls `detect_parser()` → `parser.preview_rows()`. No way to pass a delimiter.

**`parse_csv_data`** (line 283): Takes `input: String` + `mapping: ColumnMapping`. Also calls `detect_parser()` with no delimiter override.

Both commands fail with the "Could not detect" error when `detect_parser` returns `None` (score < 0.3).

### Frontend flow — `src/lib/BulkUpload.svelte`

`handleFileInput()` (line 32) calls `invoke("preview_csv", { input: text })`. On success, advances to `column-mapping` step. On failure, shows error — no way to retry with a different delimiter.

`handleMapping()` (line 92) calls `invoke("parse_csv_data", { input: inputText, mapping })`. Same issue — no delimiter param.

### `FileInput.svelte`

The input step (`src/lib/bulk-upload/FileInput.svelte`) has a text area and file upload. The `submit()` function (line 37) calls `onnext({ text, filename })`. No delimiter state exists.

## Scope

### 1. Fix `detect_delimiter` fallback — `crates/core/src/parsers/csv_parser.rs`

Change the scoring to use a consistency ratio (like `detect()` already does at line 82-84) instead of requiring exact unanimity. When a delimiter has the most columns *and* >60% consistency, prefer it. Explicitly default to `,` when all delimiters still tie at 0.

```rust
// Sketch: score = col_count * consistency_ratio, default to ','
let score = if first > 1 {
    let consistent = counts.iter().filter(|&&c| c == first).count();
    let ratio = consistent as f64 / counts.len() as f64;
    if ratio > 0.6 { (first as f64 * ratio) as usize } else { 0 }
} else { 0 };
// ...
.unwrap_or(',')  // already correct, but now reachable less often
```

### 2. Add delimiter-aware methods to `CsvParser` — `crates/core/src/parsers/csv_parser.rs`

Add public methods that accept an explicit delimiter, bypassing `detect_delimiter()`:

```rust
impl CsvParser {
    pub fn preview_rows_with_delimiter(&self, input: &str, delimiter: char)
        -> Result<Vec<Vec<String>>, ParseError> { ... }

    pub fn parse_with_delimiter(&self, input: &str, mapping: &ColumnMapping, delimiter: char)
        -> Result<Vec<ParsedExpense>, ParseError> { ... }
}
```

These are thin wrappers around the existing logic but skip auto-detection and use the given `delimiter` directly.

### 3. Add optional `delimiter` param to Tauri commands — `src-tauri/src/lib.rs`

**`preview_csv`** (line 267): Add `delimiter: Option<String>` parameter. When `Some`, create `CsvParser` directly and call `preview_rows_with_delimiter()` with `delimiter.chars().next()`. When `None`, use existing `detect_parser()` flow.

**`parse_csv_data`** (line 283): Same pattern — accept `delimiter: Option<String>`, use `parse_with_delimiter()` when provided.

```rust
#[tauri::command]
fn preview_csv(input: String, delimiter: Option<String>) -> Result<PreviewResult, String> {
    if let Some(d) = delimiter {
        let delim_char = d.chars().next()
            .ok_or("Invalid delimiter")?;
        let parser = CsvParser;
        let rows = parser.preview_rows_with_delimiter(&input, delim_char)
            .map_err(|e| e.to_string())?;
        return Ok(PreviewResult { parser_name: "CSV".into(), rows });
    }
    // ... existing auto-detect flow
}
```

### 4. Add delimiter chooser to `FileInput.svelte`

Add a `delimiter` state variable (default `null` = auto-detect). When `submit()` fails with the "Could not detect" error, show an inline delimiter picker with options: Auto, Comma, Semicolon, Tab, Pipe.

Pass `delimiter` through `onnext({ text, filename, delimiter })`.

```svelte
let delimiter = $state(null);
let showDelimiterPicker = $state(false);

const delimiters = [
  { value: null, label: "Auto-detect" },
  { value: ",", label: "Comma (,)" },
  { value: ";", label: "Semicolon (;)" },
  { value: "\t", label: "Tab" },
  { value: "|", label: "Pipe (|)" },
];
```

Show the picker either always (as a collapsed "Advanced" section) or only after auto-detect fails. The picker should appear inline above the "Next" button.

### 5. Thread delimiter through BulkUpload.svelte

- `handleFileInput()` (line 32): Accept `delimiter` from FileInput, pass to `invoke("preview_csv", { input: text, delimiter })`.
- Store `delimiter` in component state so it persists across steps.
- `handleMapping()` (line 92): Pass `delimiter` to `invoke("parse_csv_data", { input: inputText, mapping, delimiter })`.

## Files to Change

| File | Change |
|---|---|
| `crates/core/src/parsers/csv_parser.rs` | Fix `detect_delimiter` scoring/fallback; add `preview_rows_with_delimiter` and `parse_with_delimiter` methods |
| `src-tauri/src/lib.rs` | Add `delimiter: Option<String>` to `preview_csv` and `parse_csv_data` commands |
| `src/lib/bulk-upload/FileInput.svelte` | Add delimiter picker UI; pass delimiter through `onnext` callback |
| `src/lib/BulkUpload.svelte` | Store delimiter state; pass it to `preview_csv` and `parse_csv_data` invocations |

## Test Scenarios

### Backend (Rust unit tests)

1. **`detect_delimiter` with trailing comma:** Input has 8-col header and 9-col data rows (trailing comma). Should still detect `,` as delimiter.
2. **`detect_delimiter` all-zero fallback:** Input where no delimiter is consistent. Should return `,` not `|`.
3. **`preview_rows_with_delimiter` with forced semicolon:** Pass semicolon-delimited input with comma in quoted fields. Should split on semicolons only.
4. **`parse_with_delimiter` with forced comma:** Polish bank CSV with `"-12,50"` quoted amounts. Should parse correctly when comma delimiter is forced.

### Frontend (manual UI tests)

1. Paste well-formed CSV (comma-delimited, consistent columns). Auto-detect works, no delimiter picker shown. Proceeds to column mapping normally.
2. Paste Polish bank CSV with trailing commas and quoted amounts. Auto-detect fails. Delimiter picker appears. Select "Comma". Preview loads correctly with all columns visible.
3. Select "Semicolon" for a semicolon-delimited file. Verify preview shows correct column split.
4. Change delimiter selection back to "Auto-detect". Verify it retries auto-detection.
5. Complete full upload flow with manual delimiter selected. Verify the same delimiter is used in both the preview and parse steps (columns don't shift between steps).

## Acceptance Criteria

- Auto-detection works for CSVs with trailing delimiters (up to 1 extra/missing column per row)
- When auto-detection fails, user can manually pick a delimiter and retry without going back
- Selected delimiter is used consistently for both `preview_csv` and `parse_csv_data` calls
- The delimiter picker does not appear when auto-detection succeeds (no unnecessary UI noise)
- `detect_delimiter` defaults to `,` (not `|`) when all delimiters score equally
- Existing CSV uploads that worked before continue to work identically (no regressions)
