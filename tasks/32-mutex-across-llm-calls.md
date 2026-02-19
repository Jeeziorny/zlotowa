# 32 — Release DB Mutex Before LLM Calls

## Problem

In `parse_and_classify()` (`src-tauri/src/lib.rs:250-346`), the DB mutex is acquired at line 250 and held through `provider.classify_batch()` (line 306), which makes blocking HTTP requests. If the LLM provider is slow or hangs, the entire app's database access is frozen — no other Tauri command can read or write.

## Scope

- Restructure `parse_and_classify()` to:
  1. Acquire mutex → read rules + config → release mutex
  2. Run classification pipeline (regex) without lock
  3. Call LLM provider (no lock held)
  4. Acquire mutex → save results → release mutex
- Ensure the data extracted before release is sufficient for classification
- Also review `bulk_save_expenses()` for similar patterns
