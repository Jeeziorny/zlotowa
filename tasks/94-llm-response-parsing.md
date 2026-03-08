# Task 94: LLM Response Parsing Robustness

## Goal

Replace blind JSON indexing in `http_classify()` path traversal with `.get()` and descriptive error messages when the LLM API returns an unexpected response structure.

## Problem

`http_classify()` uses `node[idx]` / `node[*key]` to walk a JSON path (e.g. `choices → 0 → message → content`). `serde_json` doesn't panic but silently returns `Null`, leading to a generic "No content in response" error with no indication of what went wrong.

## Change

Replace the path traversal loop (lines 196-201) with `.get()` + early `Err` return. The error message includes:
- Which key/index was missing
- The full expected path
- How far traversal got before failing

Reuses existing `LlmError::RequestFailed`. No new error variants.

## Files modified
- `crates/core/src/llm.rs`

## Tests added
7 unit tests covering all 3 provider response shapes (OpenAI, Anthropic, Ollama), missing top-level key, empty array, missing nested key, and non-string value at path end.

## How to verify
- `cargo test -p accountant-core -- llm` — all tests pass
- Existing LLM integration (bulk upload with configured provider) works unchanged
