# 45 — LLM Provider Code Deduplication

## Problem

**`crates/core/src/llm.rs:149-396`** — The three LLM provider implementations (`OpenAiProvider`, `AnthropicProvider`, `OllamaProvider`) each repeat ~70 lines of near-identical logic in `classify_batch()`:

1. Early return for empty input
2. Check API key (except Ollama)
3. Build prompt via `build_classification_prompt()`
4. HTTP POST with provider-specific URL/headers/body
5. Check response status
6. Extract text content from provider-specific JSON response path
7. Parse via `parse_classification_response()`

The only differences are: URL, headers, request body shape, and response JSON path.

### Also

- `pub` visibility on `OpenAiProvider`, `AnthropicProvider`, `OllamaProvider` (lines 151, 234, 322) — these are only instantiated inside `create_provider()`. Could be private with the factory as the sole public API.

## Scope

- Extract a shared `http_classify()` helper that takes provider-specific config (url, headers, body builder, response extractor)
- OR use a struct-based approach: shared `HttpLlmProvider` with provider-specific `ProviderConfig`
- Reduce visibility of concrete provider structs to `pub(crate)` or private
- Target: eliminate ~120 lines of duplicated code
