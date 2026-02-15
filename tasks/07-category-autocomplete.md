# Task 7: Add Category Autocomplete to AddExpense

**Track:** D — Quick Win (standalone, frontend only)
**Blocked by:** nothing
**Blocks:** nothing

## Problem

The initial prompt (`initial_prompt.md` line 62) states: "when user types title, application
prompts possible classification using data from classification database." Currently, the
AddExpense form (`src/lib/AddExpense.svelte`) has a plain text input for category with
no suggestions. The `get_categories` Tauri command already exists and returns all known
categories.

## Current State

**`src/lib/AddExpense.svelte`:**
- Title input: plain `<input type="text">` (line 68-74)
- Category input: plain `<input type="text">` with placeholder "e.g. Groceries" (line 83-89)
- No `onMount` — doesn't load any reference data
- `get_categories` is NOT imported or called

**Available Tauri commands:**
- `get_categories` (`src-tauri/src/lib.rs:112-115`) — returns `Vec<String>` of distinct categories from classification_rules table
- No command to suggest a category given a title (would need regex matching)

**Classification rules in DB:**
- `classification_rules` table has `pattern` (regex) and `category` columns
- When user categorizes "LIDL" as "Groceries", rule `(?i)LIDL` → "Groceries" is saved
- These rules could suggest a category when the user types a title

## Scope

### 1. Load categories on mount

```js
import { onMount } from "svelte";

let allCategories = $state([]);

onMount(async () => {
    try {
        allCategories = await invoke("get_categories");
    } catch (err) {
        console.error("Failed to load categories:", err);
    }
});
```

### 2. Category autocomplete dropdown

When user types in the Category field, filter `allCategories` and show a dropdown:

```js
let categoryQuery = $state("");
let showCategorySuggestions = $state(false);

let filteredCategories = $derived(
    categoryQuery
        ? allCategories.filter(c => c.toLowerCase().includes(categoryQuery.toLowerCase()))
        : allCategories
);
```

Replace the category input with:
```svelte
<div class="relative">
    <input
        type="text"
        bind:value={category}
        oninput={(e) => { categoryQuery = e.target.value; showCategorySuggestions = true; }}
        onfocus={() => showCategorySuggestions = true}
        onblur={() => setTimeout(() => showCategorySuggestions = false, 150)}
        placeholder="e.g. Groceries"
        class="w-full bg-gray-800 border border-gray-700 rounded-lg px-4 py-2.5
               text-gray-100 placeholder-gray-600 focus:outline-none focus:border-emerald-500"
    />
    {#if showCategorySuggestions && filteredCategories.length > 0}
        <div class="absolute z-10 w-full mt-1 bg-gray-800 border border-gray-700
                    rounded-lg shadow-lg max-h-48 overflow-y-auto">
            {#each filteredCategories as cat}
                <button
                    type="button"
                    class="w-full text-left px-4 py-2 text-gray-200 hover:bg-gray-700
                           transition-colors text-sm"
                    onmousedown={() => { category = cat; showCategorySuggestions = false; }}
                >
                    {cat}
                </button>
            {/each}
        </div>
    {/if}
</div>
```

Note: use `onmousedown` instead of `onclick` because `onblur` fires before `onclick`.

### 3. Title-based category suggestion (new Tauri command)

Add a `suggest_category` command that runs classification rules against a typed title:

In `src-tauri/src/lib.rs`:
```rust
#[tauri::command]
fn suggest_category(state: State<AppState>, title: String) -> Result<Option<String>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let rules = db.get_all_rules().map_err(|e| e.to_string())?;
    let classifier = RegexClassifier::from_rules(&rules);
    let parsed = ParsedExpense {
        title,
        amount: 0.0,
        date: chrono::Local::now().date_naive(),
    };
    match classifier.classify(&parsed) {
        Ok(Some(result)) => Ok(Some(result.category)),
        _ => Ok(None),
    }
}
```

Register in `invoke_handler!`.

### 4. Show suggestion when typing title

In AddExpense, debounce the title input and call `suggest_category`:

```js
let suggestedCategory = $state("");

let debounceTimer;
function onTitleInput(value) {
    title = value;
    clearTimeout(debounceTimer);
    debounceTimer = setTimeout(async () => {
        if (value.trim().length >= 3) {
            try {
                const suggestion = await invoke("suggest_category", { title: value });
                suggestedCategory = suggestion || "";
            } catch { suggestedCategory = ""; }
        } else {
            suggestedCategory = "";
        }
    }, 300); // 300ms debounce
}
```

Show the suggestion as a subtle hint below the category input:
```svelte
{#if suggestedCategory && !category}
    <button
        type="button"
        onclick={() => category = suggestedCategory}
        class="mt-1 text-xs text-emerald-500 hover:text-emerald-400"
    >
        Suggested: {suggestedCategory} (click to apply)
    </button>
{/if}
```

### 5. Also add autocomplete to BulkUpload review step (stretch goal)

The category input in BulkUpload review (line 448-455) could also benefit from autocomplete.
This is a stretch — the inline input in a table row is tighter. Consider adding it if time
allows, or leave for a follow-up.

## Files to Change

| File | Change |
|---|---|
| `src/lib/AddExpense.svelte` | Add onMount, category autocomplete dropdown, title-based suggestion |
| `src-tauri/src/lib.rs` | Add `suggest_category` command, register in handler |

## Test Scenarios

### Backend

1. **`test_suggest_category_match`** — with rule `(?i)starbucks` → "Coffee", `suggest_category("Starbucks order")` returns `Some("Coffee")`
2. **`test_suggest_category_no_match`** — `suggest_category("unknown store")` returns `None`
3. **`test_suggest_category_empty_title`** — empty string returns `None`
4. **`test_suggest_category_no_rules`** — empty rules DB returns `None`

### Frontend (manual UI tests)

5. **Category dropdown appears** — focus on category field, see all categories listed
6. **Category filtering** — type "Gro", see only "Groceries" (if it exists)
7. **Category selection** — click a suggestion, field fills in, dropdown closes
8. **Title suggestion** — type "LIDL" in title (with existing rule), see "Suggested: Groceries" below category field
9. **Apply suggestion** — click "Suggested: Groceries", category field fills in
10. **No suggestion for unknown title** — type "XYZABC123" in title, no suggestion appears
11. **Suggestion disappears when category typed** — if user manually types a category, suggestion hint hides
12. **Debounce** — type quickly, verify only one suggestion request fires (no flicker)
13. **Works with empty DB** — no categories, no rules — no errors, just no suggestions

## Acceptance Criteria

- Category field shows autocomplete dropdown with existing categories
- Typing in title field suggests a matching category based on DB rules
- Suggestion is non-intrusive (hint text, not forced)
- 300ms debounce on title lookup (no excessive IPC calls)
- All existing AddExpense functionality preserved
- Follows dark theme conventions
