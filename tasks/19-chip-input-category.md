# Task 19: Replace Category Text Input with Tag/Chip Input Component

**Track:** Frontend — UX improvement (Svelte component)
**Blocked by:** nothing
**Blocks:** nothing

## Problem

The category input in the BulkUpload review table (`src/lib/BulkUpload.svelte:325-334`) is a plain `<input>` capped at `max-w-48` (~12rem). This makes it hard to see the full category name, especially for longer values like "Restaurant & Dining" or "Public Transport". The same issue exists for the "Match keyword" field at `max-w-40`.

More importantly, the current design is a dead end: we want to support **multiple categories per expense** in the future. A single text input cannot represent multiple values — there's no way to display them, remove individual ones, or add new ones alongside existing ones.

The AddExpense page (`src/lib/AddExpense.svelte:152-178`) already has a working autocomplete dropdown for categories, but it's built inline rather than as a reusable component. The BulkUpload table has no autocomplete at all — users must type category names from memory.

## Current State

### BulkUpload review table (`src/lib/BulkUpload.svelte`)

The `{#snippet expenseTable(rows, showSource, showRulePattern)}` at line 285 renders the review table. Two input fields are relevant:

**Category input (line 325-334):**
```svelte
<td class="px-4 py-2">
  <input
    type="text"
    value={row.category || ""}
    onchange={(e) => editCategory(origIndex, e.target.value)}
    placeholder="Enter category"
    class="bg-gray-800 border border-gray-700 rounded px-2 py-1
           text-gray-100 placeholder-gray-600 focus:outline-none
           focus:border-emerald-500 w-full max-w-48"
  />
</td>
```

**Match keyword input (line 310-323):**
```svelte
<td class="px-4 py-2">
  <input
    type="text"
    value={row.rule_pattern || ""}
    onchange={(e) => onRulePatternChange(origIndex, e.target.value)}
    placeholder="e.g. LIDL"
    class="bg-gray-800 border border-gray-700 rounded px-2 py-1
           text-gray-100 placeholder-gray-600 focus:outline-none
           focus:border-emerald-500 w-full max-w-40 text-xs"
  />
</td>
```

**`editCategory()` (line 224-227):** Sets `classifiedRows[index].category` to a single string and marks source as "Manual".

**`onRulePatternChange()` (line 229-251):** Sets the rule pattern and auto-applies the category to other unclassified rows whose title contains the keyword.

**`saveApproved()` (line 253-270):** Sends `category` as `Option<String>` to backend's `bulk_save_expenses`.

### AddExpense autocomplete (`src/lib/AddExpense.svelte:150-178`)

Has a working pattern: fetch all categories via `invoke("get_categories")`, filter on input, show dropdown, select on click. But this is built inline, not reusable.

### Backend category data

- `get_categories` IPC command (`src-tauri/src/lib.rs:111`) returns `Vec<String>` from `db.get_all_categories()`
- DB query (`crates/core/src/db.rs:280`): `SELECT DISTINCT category FROM classification_rules ORDER BY category`
- `BulkSaveExpense.category` is `Option<String>` — single value today

### Data model note

The `expenses` table stores `category TEXT` — a single string column. Multi-category support will eventually require a join table, but that is **out of scope** for this task. This task only changes the UI component so it's structurally ready (array of chips), while still sending a single string to the backend.

## Scope

### 1. Create `ChipInput.svelte` component

**File:** `src/lib/ChipInput.svelte` (new)

A reusable chip/tag input component with these props (Svelte 5 `$props()`):

```svelte
let {
  values = [],          // string[] — current chip values
  onchange,             // (values: string[]) => void — called when chips change
  suggestions = [],     // string[] — autocomplete suggestions
  placeholder = "",     // placeholder when no chips and input is empty
  max = 0,              // max number of chips (0 = unlimited), use 1 for single-category mode
  disabled = false,
} = $props();
```

**Behavior:**
- Renders existing values as styled pills/chips, each with an `×` button to remove
- After the chips, an inline `<input>` for typing new values
- On Enter or comma: if input is non-empty, add as a new chip (trim whitespace, deduplicate)
- On Backspace with empty input: remove last chip
- When `max=1` and a chip exists: hide the input (single-select mode, user must remove the chip first to change it)
- Container has `display: flex; flex-wrap: wrap; gap` styling so chips wrap naturally and the row grows in height
- Clicking anywhere in the container focuses the input

**Autocomplete dropdown:**
- When input has text, filter `suggestions` by case-insensitive substring match
- Show a positioned dropdown below the input (like AddExpense does at line 165-177)
- Click or Enter on a suggestion adds it as a chip
- Arrow-key navigation through suggestions (optional, nice-to-have)
- Close dropdown on blur (with small timeout to allow click registration, as in AddExpense line 159)

**Styling:**
- Chip: `bg-emerald-900/50 text-emerald-400 rounded px-2 py-0.5 text-xs` with `×` button
- Container: `bg-gray-800 border border-gray-700 rounded px-2 py-1 flex flex-wrap gap-1 items-center min-h-[1.75rem]`
- Input: no border, transparent background, grows to fill remaining space (`flex: 1; min-width: 4rem`)
- Focus state: container gets `border-emerald-500` when input is focused

### 2. Integrate `ChipInput` into BulkUpload category column

**File:** `src/lib/BulkUpload.svelte`

- Import `ChipInput` at top of script
- Fetch categories on mount (before `goToReview`) or within `goToReview` alongside the `parse_and_classify` call, storing them in a new `let allCategories = $state([])` variable via `invoke("get_categories")`
- In the `expenseTable` snippet, replace the category `<input>` (line 325-334) with:

```svelte
<td class="px-4 py-2">
  <ChipInput
    values={row.category ? [row.category] : []}
    onchange={(vals) => editCategory(origIndex, vals[0] || "")}
    suggestions={allCategories}
    placeholder="Category"
    max={1}
  />
</td>
```

- This keeps `editCategory()` and `saveApproved()` unchanged — they still work with a single string. The `ChipInput` with `max={1}` wraps/unwraps the single value to/from an array.

### 3. Remove max-width constraints

**File:** `src/lib/BulkUpload.svelte`

- Remove `max-w-48` from the category input (now handled by ChipInput's flexible layout)
- Remove `max-w-40` from the match keyword input (line 317) — just drop the class, keep `w-full`

### 4. (Optional) Use `ChipInput` for match keyword field too

**File:** `src/lib/BulkUpload.svelte`

The match keyword field could also benefit from ChipInput in the future (multiple match patterns per rule). For now, simply remove `max-w-40` so it fills the cell. Converting it to ChipInput is a future enhancement and NOT required for this task.

## Files to Change

| File | Change |
|---|---|
| `src/lib/ChipInput.svelte` | **New file.** Reusable tag/chip input with autocomplete dropdown |
| `src/lib/BulkUpload.svelte` | Import `ChipInput`, fetch categories, replace category `<input>` in `expenseTable` snippet, remove `max-w-40` from match keyword input |

## Test Scenarios

### Frontend (manual UI tests)

1. **Single chip display:** Upload a CSV where some expenses get classified by regex rules. In the review step, verify that classified expenses show the category as a chip (pill with `×`) instead of plain text in an input.

2. **Remove and re-enter:** Click `×` on a chip to remove it. Verify the chip disappears and the text input appears. Type a new category, press Enter. Verify the new chip appears and the input hides (since `max=1`).

3. **Autocomplete from existing categories:** In the "Needs your input" section, click into the category field of an unclassified expense. Type a few letters of an existing category (e.g., "Gro" for "Groceries"). Verify a dropdown appears with matching categories. Click one. Verify it becomes a chip.

4. **Keyboard interaction:** Focus the chip input. Type "Food" and press Enter. Verify "Food" appears as a chip. Press Backspace. Verify the chip is removed and the input is empty again.

5. **Long category names:** Enter a long category name like "Restaurant & Dining Out". Verify the chip wraps correctly within the table cell and the row height adjusts — no truncation or overflow.

6. **Match keyword field width:** Verify the match keyword input now fills its table cell without the old `max-w-40` constraint. Long keywords like "BIEDRONKA STORE" should be fully visible.

7. **Save flow unchanged:** Classify expenses (mix of rule, LLM, and manual), edit some categories via chips, click Save. Verify the correct number of expenses is saved and the done screen appears with the right count.

8. **Auto-apply still works:** In the "Needs your input" section, set a category via chip input on one expense, then change the match keyword. Verify other unclassified expenses matching the keyword still get auto-categorized (the `+N` badge appears).

## Acceptance Criteria

- Category field in BulkUpload review table renders as a chip/tag input, not a plain text input
- Existing categories appear as autocomplete suggestions when typing
- Chips can be added (Enter/click) and removed (× button/Backspace)
- With `max=1`, only one chip is allowed at a time (input hides when a chip exists)
- The component is reusable (`ChipInput.svelte`) and can be used in AddExpense or elsewhere later
- No `max-w-48` or `max-w-40` constraints remain on the review table inputs
- All existing BulkUpload functionality is preserved: classification display, source grouping, duplicate detection, rule pattern auto-apply, save flow
- Component follows project conventions: Svelte 5 syntax, dark theme palette, Tailwind v4 classes
- No backend changes required — `category` is still sent as a single `Option<String>`
