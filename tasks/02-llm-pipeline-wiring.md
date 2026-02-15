# Task 2: Wire LLM into Classification Pipeline and Tauri IPC

**Track:** A — LLM Integration (2 of 3)
**Blocked by:** Task 1
**Blocks:** Task 3

## Problem

The `parse_and_classify` Tauri command (`src-tauri/src/lib.rs:162-207`) currently only
uses `RegexClassifier` from DB rules. When no rule matches, expenses are left unclassified.
The LLM provider (implemented in Task 1) needs to be plugged in as a fallback classifier,
and the `save_llm_config` command needs to validate the API key before saving.

## Current State

**`src-tauri/src/lib.rs:162-207`** — `parse_and_classify`:
```rust
// Classify using regex rules from DB
let rules = db.get_all_rules().map_err(|e| e.to_string())?;
let regex_classifier = RegexClassifier::from_rules(&rules);
let classifiers: Vec<Box<dyn Classifier>> = vec![Box::new(regex_classifier)];
let classified = classify_pipeline(&parsed, &classifiers);
```
Only `RegexClassifier` is in the pipeline. No LLM fallback.

**`src-tauri/src/lib.rs:128-136`** — `save_llm_config`:
```rust
fn save_llm_config(state: State<AppState>, config: LlmConfigInput) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.set_config("llm_provider", &config.provider).map_err(|e| e.to_string())?;
    db.set_config("llm_api_key", &config.api_key).map_err(|e| e.to_string())?;
    Ok(())
}
```
Saves blindly — no validation.

**Classification pipeline** (`crates/core/src/classifiers.rs:79-95`):
- `classify_pipeline()` takes `&[Box<dyn Classifier>]`, runs them by priority, first match wins
- `RegexClassifier` has priority `10`
- An LLM classifier should have priority `20` (higher number = tried later)

**`crates/core/src/llm.rs`** (after Task 1):
- `LlmProvider` trait with `validate()` and `classify_batch()`
- `create_provider(name: &str) -> Option<Box<dyn LlmProvider>>`

## Scope

### 1. Create `LlmClassifier` wrapper

In `crates/core/src/classifiers.rs`, add a struct that wraps an `LlmProvider` and
implements the `Classifier` trait:

```rust
pub struct LlmClassifier {
    provider: Box<dyn LlmProvider>,
    config: LlmConfig,
    categories: Vec<String>,
}
```

Implementation notes:
- `priority()` returns `20` (after RegexClassifier's `10`)
- `classify()` calls `provider.classify_batch()` with a single expense
- OR: batch all unclassified at once for efficiency (requires changing how the pipeline works)

**Decision point:** The current pipeline calls `classify()` per-expense, per-classifier.
For LLM this is wasteful (one HTTP call per expense). Two options:
- **Option A:** Keep per-expense calls, accept N HTTP calls (simpler, works with current pipeline)
- **Option B:** Add a `classify_batch()` method to the `Classifier` trait, call it for all unclassified after the pipeline (more efficient, breaks trait)
- **Recommended: Option B** — add a post-pipeline LLM batch call in the Tauri command, outside `classify_pipeline()`. This avoids changing the trait while being efficient.

### 2. Update `parse_and_classify` command

After the existing `classify_pipeline()` call, add:
```rust
// Check if LLM is configured
let llm_provider_name = db.get_config("llm_provider")?;
let llm_api_key = db.get_config("llm_api_key")?;

if let (Some(provider_name), Some(api_key)) = (&llm_provider_name, &llm_api_key) {
    if !provider_name.is_empty() && !api_key.is_empty() {
        // Collect unclassified expenses
        let unclassified_indices: Vec<usize> = rows.iter().enumerate()
            .filter(|(_, r)| r.category.is_none() && !r.is_duplicate)
            .map(|(i, _)| i)
            .collect();

        if !unclassified_indices.is_empty() {
            let provider = create_provider(provider_name);
            let config = LlmConfig { provider: provider_name.clone(), api_key: api_key.clone() };
            let categories = db.get_all_categories()?;
            // ... call classify_batch, update rows
        }
    }
}
```

### 3. Add validation to `save_llm_config`

```rust
fn save_llm_config(state: State<AppState>, config: LlmConfigInput) -> Result<(), String> {
    // Validate before saving
    let provider = create_provider(&config.provider)
        .ok_or("Unknown provider")?;
    let llm_config = LlmConfig { provider: config.provider.clone(), api_key: config.api_key.clone() };
    provider.validate(&llm_config).map_err(|e| e.to_string())?;

    // Save only if valid
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.set_config("llm_provider", &config.provider).map_err(|e| e.to_string())?;
    db.set_config("llm_api_key", &config.api_key).map_err(|e| e.to_string())?;
    Ok(())
}
```

### 4. Add `validate_llm_config` command (new)

Separate command for the "Test Connection" button (Task 3 frontend):
```rust
#[tauri::command]
fn validate_llm_config(config: LlmConfigInput) -> Result<(), String> {
    let provider = create_provider(&config.provider).ok_or("Unknown provider")?;
    let llm_config = LlmConfig { ... };
    provider.validate(&llm_config).map_err(|e| e.to_string())
}
```
Register in `invoke_handler!`.

## Files to Change

| File | Change |
|---|---|
| `src-tauri/src/lib.rs` | Update `parse_and_classify`, `save_llm_config`; add `validate_llm_config`; register new command |
| `crates/core/src/classifiers.rs` | Optionally add `LlmClassifier` wrapper (if going with Option A) |
| `src-tauri/Cargo.toml` | May need to re-export or use `accountant_core::llm` |

## Test Scenarios

1. **`test_parse_and_classify_without_llm_config`** — when no LLM config in DB, pipeline runs DB rules only, unclassified expenses stay unclassified (existing behavior preserved)
2. **`test_parse_and_classify_with_empty_llm_config`** — provider/key are empty strings, same as no config
3. **`test_save_llm_config_rejects_invalid_key`** — saving with a bad API key returns an error, config is NOT persisted
4. **`test_save_llm_config_saves_valid`** — saving with a valid key succeeds, config is persisted
5. **`test_validate_llm_config_unknown_provider`** — returns error for unknown provider name
6. **`test_classify_with_llm_fallback`** — given expenses where some match DB rules and some don't, verify DB-matched keep "Database" source and unmatched get "Llm" source after LLM call
7. **`test_llm_failure_doesnt_block_import`** — if LLM call fails (network error), expenses remain unclassified but the command doesn't error out
8. **`test_duplicates_not_sent_to_llm`** — expenses flagged as duplicates are not included in the LLM batch

## Acceptance Criteria

- `cargo build -p accountant-app` compiles
- Existing `cargo test` passes
- `parse_and_classify` returns `ClassifiedExpenseRow` with `source: Some("Llm")` for LLM-classified items
- LLM failure is graceful (logged, not propagated as error to frontend)
- `save_llm_config` rejects invalid keys
- New `validate_llm_config` command is registered and callable from frontend
