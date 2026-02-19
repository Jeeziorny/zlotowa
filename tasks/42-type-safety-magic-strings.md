# 42 — Type Safety: Replace Magic Strings with Enums

## Problem

Several places use raw string matching where enums or case-insensitive parsing would be safer. Mismatched casing silently fails.

### Locations

1. **`crates/core/src/llm.rs:17`** — `LlmConfig.provider` is a raw `String`. `create_provider()` (line 402) matches `"openai"`, `"anthropic"`, `"ollama"` case-sensitively. Storing `"OpenAI"` silently produces no provider.

2. **`crates/core/src/models.rs:37-43`** — `ClassificationSource::from_str_opt()` handles only two specific casings per variant (`"database" | "Database"`, `"llm" | "Llm"`, `"manual" | "Manual"`). `"DATABASE"` returns `None`, silently dropping source data.

3. **`src-tauri/src/lib.rs:306`** — `query.category == "uncategorized"` is a magic string for filtering uncategorized expenses. Should be a constant or enum variant.

4. **`src-tauri/src/lib.rs:603-608`** — Budget status strings `"over"`, `"approaching"`, `"under"` are constructed inline. Should be an enum with Display impl.

5. **`crates/core/src/llm.rs:79-84`** — `confidence_str_to_f64()` matches `"high"`, `"medium"`, `"low"` — acceptable for LLM output, but could be a typed enum.

## Scope

- Create a `LlmProviderType` enum (`OpenAi`, `Anthropic`, `Ollama`) with case-insensitive `FromStr`
- Change `LlmConfig.provider` from `String` to `LlmProviderType`
- Make `ClassificationSource::from_str_opt()` use `.to_lowercase()` or implement `FromStr`
- Extract `"uncategorized"` to a constant
- Create a `BudgetStatus` enum for `"over"` / `"approaching"` / `"under"`
