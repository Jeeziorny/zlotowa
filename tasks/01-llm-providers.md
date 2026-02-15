# Task 1: Implement LLM Provider HTTP Clients

**Track:** A — LLM Integration (1 of 3)
**Blocked by:** nothing
**Blocks:** Task 2

## Problem

The LLM classification feature is a core differentiator described in the initial prompt
(`initial_prompt.md` lines 10-28, 43-44). The Settings UI already lets users configure
a provider and API key, and the classification docs (`docs/src/features/classification.md`)
describe LLM as a working pipeline step. In reality, `crates/core/src/llm.rs` contains
only a trait definition with zero implementations — no HTTP calls are made.

## Current State

**`crates/core/src/llm.rs`** defines:
- `LlmError` enum: `NotConfigured`, `InvalidApiKey`, `RequestFailed(String)`
- `LlmConfig` struct: `provider: String`, `api_key: String`
- `LlmProvider` trait with three methods:
  - `name() -> &str`
  - `validate(&self, config: &LlmConfig) -> Result<(), LlmError>`
  - `classify_batch(&self, expenses: &[ParsedExpense], existing_categories: &[String], config: &LlmConfig) -> Result<Vec<Option<String>>, LlmError>`

**`crates/core/Cargo.toml`** dependencies (no HTTP client yet):
```toml
rusqlite, serde, serde_json, chrono, regex, thiserror, dirs
```

**Settings UI** (`src/lib/Settings.svelte`) saves to DB config table:
- `llm_provider` — one of: `"openai"`, `"anthropic"`, `"ollama"`
- `llm_api_key` — API key string or Ollama endpoint URL

## Scope

Implement the `LlmProvider` trait for all three providers.

### 1. Add dependencies to `crates/core/Cargo.toml`

```toml
reqwest = { version = "0.12", features = ["json", "blocking"] }
```

Decision: use **blocking** reqwest since the Tauri command layer already handles async.
The classifier pipeline is synchronous (`classify_pipeline` returns `Vec`, no futures).
If async is preferred, the trait signature and all callers must change — blocking is simpler
for this stage.

### 2. Implement `OpenAiProvider`

- Endpoint: `https://api.openai.com/v1/chat/completions`
- Model: `gpt-4o-mini` (cheap, fast, good enough for classification)
- `validate()`: send a minimal chat completion request, check for 401/403 → `InvalidApiKey`
- `classify_batch()`: build a prompt with the expense titles and existing category list,
  ask the model to return a JSON array of categories (one per expense)
- Parse the response, return `Vec<Option<String>>`

Prompt design (suggestion):
```
You are an expense classifier. Given these expense titles and a list of known categories,
assign each expense to the most appropriate category. If none fit, suggest a new one.

Known categories: [Groceries, Transport, Entertainment, ...]

Expenses:
1. LIDL STORE #42
2. UBER TRIP
3. NETFLIX SUBSCRIPTION

Respond with a JSON array of category strings, one per expense:
["Groceries", "Transport", "Entertainment"]
```

### 3. Implement `AnthropicProvider`

- Endpoint: `https://api.anthropic.com/v1/messages`
- Model: `claude-haiku-4-5-20251001` (fast, cheap)
- Headers: `x-api-key`, `anthropic-version: 2023-06-01`
- Same prompt strategy as OpenAI
- `validate()`: send a minimal message, check for auth errors

### 4. Implement `OllamaProvider`

- Endpoint: user-configured (default `http://localhost:11434`)
- API: `POST /api/generate` or `/api/chat`
- Model: use a sensible default like `llama3` (or let user configure — stretch goal)
- `validate()`: `GET /api/tags` to check if Ollama is running
- `classify_batch()`: same prompt, local inference

### 5. Factory function

Add a helper to instantiate the right provider from config:
```rust
pub fn create_provider(provider_name: &str) -> Option<Box<dyn LlmProvider>>
```

## Files to Change

| File | Change |
|---|---|
| `crates/core/Cargo.toml` | Add `reqwest` dependency |
| `crates/core/src/llm.rs` | Add `OpenAiProvider`, `AnthropicProvider`, `OllamaProvider` impls + `create_provider()` |

## Test Scenarios

### Unit tests (no real API calls)

1. **`test_create_provider_openai`** — `create_provider("openai")` returns `Some`, name is `"openai"`
2. **`test_create_provider_anthropic`** — same for anthropic
3. **`test_create_provider_ollama`** — same for ollama
4. **`test_create_provider_unknown`** — `create_provider("grok")` returns `None`
5. **`test_openai_validate_bad_key`** — mock/stub: an empty API key should return `InvalidApiKey`
6. **`test_parse_classification_response`** — given a JSON string `["Groceries","Transport"]`, the internal parser returns the correct `Vec<Option<String>>`
7. **`test_parse_malformed_response`** — non-JSON or wrong shape returns `RequestFailed`
8. **`test_classify_batch_empty_expenses`** — empty input returns empty output

### Integration tests (require real API key, marked `#[ignore]`)

9. **`test_openai_real_classify`** — `#[ignore]` test that actually hits OpenAI with 3 sample expenses
10. **`test_anthropic_real_classify`** — same for Anthropic
11. **`test_ollama_real_classify`** — same for Ollama (requires local Ollama running)

## Acceptance Criteria

- `cargo test -p accountant-core` passes with new tests
- No changes to `src-tauri/` or `src/` (that's Task 2 and 3)
- The trait remains backward-compatible (existing code that imports it still compiles)
- Response parsing is resilient: if LLM returns garbage, it's `RequestFailed`, not a panic
