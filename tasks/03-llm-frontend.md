# Task 3: Add LLM Status Feedback to Frontend

**Track:** A — LLM Integration (3 of 3)
**Blocked by:** Task 2
**Blocks:** nothing

## Problem

The frontend has all the visual plumbing for LLM (badges, provider dropdown, save/clear)
but lacks feedback on whether the LLM integration actually works. The initial prompt
(`initial_prompt.md` line 17) requires API key validation. The bulk import review step
doesn't distinguish between DB-classified, LLM-classified, and unclassified expenses
in separate visual groups as specified in the prompt (lines 46-49).

## Current State

**`src/lib/Settings.svelte`:**
- Provider dropdown: openai/anthropic/ollama (line 81-86)
- API key input with show/hide toggle
- Save button calls `invoke("save_llm_config", ...)` — no validation feedback
- Clear button calls `invoke("clear_llm_config")`
- No "Test Connection" button

**`src/lib/BulkUpload.svelte` review step (lines 418-547):**
- All non-duplicate rows shown in a single flat table
- Source badges exist: "DB Rule" (blue), "LLM" (purple), "Manual" (gray), "Unclassified" (yellow)
- No visual grouping by source — DB-classified and unclassified are interleaved

**`src/lib/ExpenseList.svelte`:**
- Source badges already handle `Database`, `Llm`, `Manual` — this is fine as-is

## Scope

### 1. Settings: Validation feedback on save

When `save_llm_config` (Task 2) now validates the key, the frontend should:
- Show a loading spinner on the Save button during the call
- On success: green message "Configuration saved and validated."
- On error: red message with the error (e.g., "Invalid API key" or "Could not connect to Ollama")

### 2. Settings: Test Connection button

Add a "Test Connection" button next to Save that calls the new `validate_llm_config` command:
```js
async function testConnection() {
    testing = true;
    try {
        await invoke("validate_llm_config", { config: { provider, api_key: apiKey } });
        message = "Connection successful!";
        messageType = "success";
    } catch (err) {
        message = `Connection failed: ${err}`;
        messageType = "error";
    }
    testing = false;
}
```

Layout: Save and Test Connection buttons side by side, with Clear on the right.

### 3. BulkUpload: Group expenses by classification source

In the review step, split `nonDuplicateRows` into three derived groups:

```js
let dbClassified = $derived(nonDuplicateRows.filter(r => r.source === "Database"));
let llmClassified = $derived(nonDuplicateRows.filter(r => r.source === "Llm"));
let unclassified = $derived(nonDuplicateRows.filter(r => !r.source || r.source === "Manual"));
```

Display order (matching initial_prompt.md lines 46-49):
1. **DB Rule matches** — header: "Classified by rules ({count})" with blue accent
2. **LLM suggestions** — header: "Classified by AI ({count})" with purple accent
3. **Unclassified** — header: "Needs your input ({count})" with yellow accent
4. **Duplicates** — already shown separately (lines 490-521)

Each group is a separate section with its own header, collapsible if empty.

### 4. BulkUpload: Loading indicator during classification

The `goToReview()` function (line 115) calls `parse_and_classify` which now may include
LLM API calls (could take 2-10 seconds). Add a loading state:

```js
let classifying = $state(false);

async function goToReview() {
    classifying = true;
    // ... existing logic
    classifying = false;
}
```

Show a spinner/message: "Classifying expenses... (checking rules and AI)" while waiting.

## Files to Change

| File | Change |
|---|---|
| `src/lib/Settings.svelte` | Add loading states, Test Connection button, better feedback messages |
| `src/lib/BulkUpload.svelte` | Group rows by source, add loading indicator |

## Design Notes

Follow existing dark theme conventions:
- Spinner: use a simple CSS animation (tailwind `animate-spin` on a border element)
- Section headers: match existing card style (`bg-gray-900 rounded-xl p-6 border border-gray-800`)
- Group accent colors: blue for DB (existing), purple for LLM (existing), yellow for unclassified

## Test Scenarios

These are manual UI tests (no automated test framework for Svelte currently).

1. **Settings — save with valid key:** enter a valid OpenAI key, click Save. Expect green "saved and validated" message.
2. **Settings — save with invalid key:** enter `sk-invalid`, click Save. Expect red error message. Config should NOT be saved (verify by refreshing page — fields should be empty).
3. **Settings — test connection button:** click Test Connection with valid config. Expect green "Connection successful!" without saving.
4. **Settings — test connection with bad key:** click Test Connection with bad key. Expect red error.
5. **Settings — Ollama not running:** set provider to Ollama, enter `http://localhost:11434`, click Test. Expect "Connection failed" if Ollama is not running.
6. **BulkUpload — mixed classification sources:** import CSV where some expenses match DB rules, some don't. Verify three groups appear: DB rules at top, LLM in middle, unclassified at bottom.
7. **BulkUpload — no LLM configured:** import CSV without LLM config. Only two groups: DB rules and unclassified (LLM group hidden or shows 0).
8. **BulkUpload — loading indicator:** import a large CSV with LLM configured. Verify spinner shows during classification.
9. **BulkUpload — edit LLM suggestion:** in the LLM group, change a category. Verify source badge changes to "Manual".
10. **ExpenseList — LLM badge:** after saving LLM-classified expenses, verify they appear in expense list with purple "LLM" badge.

## Acceptance Criteria

- Settings page shows validation result on save
- Test Connection button works independently of save
- BulkUpload review step shows expenses in three visual groups
- Loading indicator appears during classification
- All existing functionality preserved (no regressions)
- Follows dark theme conventions
