# Task 10: Click-to-Assign Column Mapping for Bulk Upload

**Track:** E — UX Improvement (frontend + minor backend)
**Blocked by:** nothing
**Blocks:** nothing

## Problem

The column mapping step (Step 2) in Bulk Upload uses three separate `<select>` dropdowns to assign Title, Amount, and Date columns. This is clunky:
- With wide CSVs (10+ columns from bank exports), each dropdown is a wall of text
- Nothing prevents assigning the same column to multiple roles
- Date format must be guessed — user discovers the wrong format only after hitting "Next" and getting a parse error
- No validation feedback — the color-coded preview shows selection but not whether data actually parses

Client feedback confirms: "Process of choosing columns is a bit messy."

## Current State

**`src/lib/BulkUpload.svelte` (Step 2: column-mapping, lines 329-481):**
- Four `<select>` dropdowns in a 2×2 grid (Title, Amount, Date, Date format)
- Each dropdown lists all header row values — fully independent, no overlap prevention
- Preview table below shows first 5 data rows, color-coded by assignment (emerald=title, blue=amount, purple=date, gray=unmapped)
- Date format dropdown has 7 hardcoded options (lines 30-38), defaults to `%Y-%m-%d`
- Auto-detection runs in `goToMapping()` (lines 92-113): scans headers for keywords like `title`, `amount`, `date` (+ Polish equivalents). If nothing matches, defaults to col 0/1/2.

**Backend (`src-tauri/src/lib.rs`):**
- `preview_csv` (line 192): returns `PreviewResult { parser_name, rows: Vec<Vec<String>> }` — just raw rows, no column analysis
- `parse_and_classify` (line 206): receives `ColumnMapping { title_index, amount_index, date_index, date_format }`, re-detects parser, parses, classifies

**Parser (`crates/core/src/parsers/csv_parser.rs`):**
- `parse_amount()` (line 174): handles US/European formats, currency symbols
- Date parsing uses `NaiveDate::parse_from_str(value, format)` — strict, fails on mismatch

## Scope

### 1. Replace dropdowns with click-to-assign interaction on the preview table

Remove the four `<select>` elements. Instead, show the preview table immediately and let users assign columns by clicking column headers.

**Interaction model:**
- Each column header is a clickable button
- Clicking a column header opens a small popover/pill menu with options: **Title**, **Amount**, **Date**, **Ignore** (or unassign if already assigned)
- When a role is assigned to a column, the previous column with that role is automatically unassigned (radio-style — each role can only be assigned to one column)
- Assigned columns show a colored badge in the header: green "Title", blue "Amount", purple "Date"
- Unassigned columns show a subtle "Click to assign" hint on hover
- Pre-assignment via auto-detection still runs (same keyword matching logic), so the user lands on a pre-configured table

**State changes:**
```js
// Replace separate titleCol/amountCol/dateCol with a single mapping object
let columnRoles = $state({});  // { [colIndex]: "title" | "amount" | "date" }
let activePopover = $state(null);  // which column index has popover open, null = closed

// Derived helpers
let titleCol = $derived(
  Object.entries(columnRoles).find(([_, role]) => role === "title")?.[0]
);
// ... same for amountCol, dateCol
```

**Popover component (inline, not a separate file):**
```svelte
{#if activePopover === i}
  <div class="absolute z-20 mt-1 bg-gray-800 border border-gray-700 rounded-lg shadow-lg p-1 flex gap-1">
    <button onclick={() => assignRole(i, "title")}
      class="px-2 py-1 rounded text-xs hover:bg-emerald-900/50 text-emerald-400">
      Title
    </button>
    <button onclick={() => assignRole(i, "amount")}
      class="px-2 py-1 rounded text-xs hover:bg-blue-900/50 text-blue-400">
      Amount
    </button>
    <button onclick={() => assignRole(i, "date")}
      class="px-2 py-1 rounded text-xs hover:bg-purple-900/50 text-purple-400">
      Date
    </button>
    {#if columnRoles[i]}
      <button onclick={() => unassignRole(i)}
        class="px-2 py-1 rounded text-xs hover:bg-gray-700 text-gray-400">
        Clear
      </button>
    {/if}
  </div>
{/if}
```

**`assignRole(colIndex, role)` function:**
```js
function assignRole(colIndex, role) {
  // Remove role from any other column (radio behavior)
  const newRoles = { ...columnRoles };
  for (const [key, val] of Object.entries(newRoles)) {
    if (val === role) delete newRoles[key];
  }
  newRoles[colIndex] = role;
  columnRoles = newRoles;
  activePopover = null;
}
```

Close popover when clicking outside (use a backdrop overlay or `window` click listener).

### 2. Auto-detect date format from data

After auto-detecting column assignments (or when the user assigns the date column), try all 7 date formats against the first 3 non-empty values in that column. Pick the format that successfully parses all of them.

```js
function autoDetectDateFormat(colIndex) {
  const values = dataRows
    .slice(0, 3)
    .map(row => row[colIndex]?.trim())
    .filter(Boolean);

  if (values.length === 0) return;

  for (const fmt of dateFormats) {
    const allMatch = values.every(v => testDateFormat(v, fmt.value));
    if (allMatch) {
      dateFormat = fmt.value;
      return;
    }
  }
  // No format matched all values — keep current default
}
```

The `testDateFormat` function is a simple JS heuristic that mirrors the strftime patterns. For example, `%d/%m/%Y` expects `DD/MM/YYYY` — check that the string has `/` separators, the right segment lengths, and reasonable ranges (day 1-31, month 1-12, year 1900-2100). This doesn't need to be perfect — it's a best-guess that the user can override.

Show the detected format as a small pill below the table: `"Date format: DD/MM/YYYY (auto-detected)"` with a dropdown to override.

### 3. Inline parse validation in preview cells

In the preview table, for assigned columns, show validation indicators:

- **Amount column**: try parsing each cell value with the same logic as `parse_amount()` (strip currency symbols, handle comma/dot decimals). Show the parsed number in blue, or a red warning icon if it fails.
- **Date column**: try parsing each cell with the selected date format. Show the parsed date in purple, or a red warning icon if it fails.
- **Title column**: no validation needed — just show in green.

This can be done purely in the frontend with JS parsing heuristics. It doesn't need to match the Rust parser perfectly — it's a visual aid, not a guarantee.

```svelte
<!-- In preview table cells -->
{#if columnRoles[i] === "amount"}
  {@const parsed = tryParseAmount(cell)}
  <td class="px-3 py-2">
    <span class="text-blue-300">{cell}</span>
    {#if parsed !== null}
      <span class="text-xs text-blue-500 ml-1">({parsed.toFixed(2)})</span>
    {:else}
      <span class="text-xs text-red-400 ml-1">⚠</span>
    {/if}
  </td>
{:else if columnRoles[i] === "date"}
  {@const valid = testDateFormat(cell, dateFormat)}
  <td class="px-3 py-2">
    <span class="text-purple-300">{cell}</span>
    {#if !valid}
      <span class="text-xs text-red-400 ml-1">⚠</span>
    {/if}
  </td>
{/if}
```

### 4. Prevent proceeding with incomplete/invalid mapping

The "Next: Classify & Review" button should be disabled unless:
- All three roles (title, amount, date) are assigned to distinct columns
- No parse validation errors in the preview rows (or at least warn)

```js
let mappingComplete = $derived(
  titleCol != null && amountCol != null && dateCol != null
);
```

Show a subtle message if roles are missing: "Assign all three columns (Title, Amount, Date) to continue."

### 5. Wire the new state to the existing `goToReview()` call

The `invoke("parse_and_classify", ...)` call (line 125) currently reads `titleCol`, `amountCol`, `dateCol`, `dateFormat`. Update it to derive these from `columnRoles`:

```js
async function goToReview() {
  const tc = Number(Object.entries(columnRoles).find(([_, r]) => r === "title")?.[0]);
  const ac = Number(Object.entries(columnRoles).find(([_, r]) => r === "amount")?.[0]);
  const dc = Number(Object.entries(columnRoles).find(([_, r]) => r === "date")?.[0]);
  // ... rest same, use tc/ac/dc as title_index/amount_index/date_index
}
```

## Files to Change

| File | Change |
|---|---|
| `src/lib/BulkUpload.svelte` | Replace dropdown-based column mapping with click-to-assign table interaction, add date format auto-detection, add inline parse validation, add mapping completeness check |

No backend changes needed — the `preview_csv` and `parse_and_classify` IPC commands stay the same. The `ColumnMapping` struct is unchanged.

## Test Scenarios

### Frontend (manual UI tests)

1. **Auto-detection still works** — upload a CSV with headers `date,description,amount`; Title, Amount, Date should be pre-assigned to correct columns with colored badges
2. **Click-to-assign** — upload a CSV with unfamiliar headers; click a column header, see popover with Title/Amount/Date/Clear options; click "Title", column gets green badge
3. **Radio behavior** — assign Title to column A, then assign Title to column B; column A should be unassigned automatically
4. **Clear assignment** — click an assigned column, click "Clear"; badge disappears, column returns to unassigned state
5. **Popover closes on outside click** — open popover, click anywhere else; popover closes
6. **Date format auto-detected** — upload CSV with dates like `15/01/2024`; after assigning date column, format should auto-detect to `DD/MM/YYYY`, not default `YYYY-MM-DD`
7. **Date format override** — auto-detected format is wrong; user changes it via the dropdown; preview validation updates immediately
8. **Amount validation in preview** — amount column shows parsed values; cells with `$1,234.56` show `(1234.56)` in blue; cells with text like `N/A` show red warning
9. **Date validation in preview** — date column cells show red warning if they don't match the selected format
10. **Incomplete mapping blocks Next** — with only Title and Amount assigned (no Date), the "Next" button is disabled with a message
11. **Duplicate column assignment prevented** — assigning the same column to two roles is impossible due to radio behavior
12. **Wide CSV (10+ columns)** — upload a bank export with many columns; all columns visible in table, scrollable, easy to click-assign without scrolling through long dropdowns
13. **Back button preserves state** — go to Step 2, assign columns, go back to Step 1, come back; assignments should re-run auto-detection (since preview data reloads)
14. **Reset clears everything** — after Step 4 "Upload More", column roles are cleared

## Acceptance Criteria

- Column assignment happens by clicking on the preview table, not via separate dropdowns
- Each role (Title/Amount/Date) can only be assigned to one column at a time
- Date format is auto-detected from data when date column is assigned
- Preview cells show inline validation (parsed values or warnings)
- Incomplete mapping prevents proceeding to review
- Auto-detection of column names still works as a smart default
- All existing Bulk Upload functionality preserved (Steps 1, 3, 4 unchanged)
- Follows dark theme conventions (gray-950/900/800, emerald/blue/purple accents)
