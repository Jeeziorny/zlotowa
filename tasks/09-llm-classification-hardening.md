# Task 9: Harden LLM Classification with Confidence & Prompt Engineering

**Track:** LLM Integration (improvement)
**Blocked by:** nothing (current LLM pipeline is functional)
**Blocks:** nothing

## Problem

The LLM classification pipeline works end-to-end but has several weak points that
degrade accuracy, consistency, and user trust. The core issues:

1. **No confidence signal** — the LLM returns bare category strings. The user sees
   a purple "LLM" badge but has no way to distinguish a high-confidence match
   (`LIDL STORE` → Groceries) from a wild guess (`POS REF 4829173` → Entertainment).
   Every LLM result looks equally trustworthy.

2. **Missing amount context** — `build_classification_prompt()` sends only expense
   titles. Amount is a strong signal being discarded: `STARBUCKS €5` is coffee,
   `STARBUCKS €350` is likely catering. `ZARA €30` vs `ZARA €800` may indicate
   different spending patterns.

3. **Unconstrained category invention** — the prompt says *"If none fit, suggest a
   new short category name"*. Across multiple batches and sessions, this causes
   category proliferation: `Groceries`, `Grocery`, `Food & Groceries`, `Supermarket`
   for the same thing.

4. **Fragile positional alignment** — the prompt sends a numbered list and expects a
   positional JSON array. If the LLM miscounts or skips an entry, the entire batch
   fails (`expected_count` check) and all expenses go unclassified. One bad entry
   kills the whole batch.

5. **Vague/cryptic titles cause hallucination** — bank transactions like
   `DD REF 92817`, `POS PAYMENT 38291` are meaningless. The LLM will guess
   confidently rather than admit uncertainty, producing plausible but wrong categories.

6. **Inconsistent temperature across providers** — OpenAI uses `temperature: 0.1`,
   Anthropic and Ollama use the default (typically 1.0). Same expense can get
   different categories depending on provider.

7. **Silent LLM failures** — if the API call fails, expenses silently stay
   unclassified. The user has no indication whether "Unclassified" means no LLM
   configured, API error, or genuinely unclassifiable.

## Current State

**Prompt** (`crates/core/src/llm.rs`, `build_classification_prompt()`):
- Sends only `title` per expense (no amount)
- Asks for flat JSON array: `["Groceries", "Transport"]`
- Freely allows new category invention

**Response parsing** (`parse_classification_response()`):
- Expects positional array matching input count exactly
- Returns `Vec<Option<String>>` — no confidence metadata

**ClassificationResult** (`crates/core/src/classifiers.rs`):
- Already has `confidence: f64` field
- RegexClassifier hardcodes it to `1.0`
- LLM path in `src-tauri/src/lib.rs:283-289` never sets confidence at all

**Frontend** (`src/lib/BulkUpload.svelte`):
- Shows source badge (DB Rule / LLM / Manual / Unclassified)
- No confidence indication for LLM results

## Acceptance Criteria

### AC1: Confidence tiers in prompt and response
- Change the LLM response format from `["Groceries"]` to
  `[{"id": 1, "category": "Groceries", "confidence": "high"}]`
- Use 3 tiers: `high`, `medium`, `low` (LLMs are bad at calibrated numeric
  probabilities but decent at ordinal ranking)
- Map to `ClassificationResult.confidence`: high=0.9, medium=0.6, low=0.3
- Update `parse_classification_response()` to handle the new format

### AC2: Include amounts in prompt
- Change expense list format from `1. LIDL STORE #42` to
  `1. LIDL STORE #42 — €45.20`
- Pass amount through to `build_classification_prompt()`

### AC3: Constrain category invention
- When existing categories are non-empty, instruct the LLM to prefer them strongly:
  *"Choose from the known categories. Only invent a new category if the expense
  genuinely doesn't fit ANY existing one."*
- When no categories exist yet, allow free invention (first-run experience)

### AC4: Keyed responses instead of positional
- Include `id` field in prompt and expected response
- Match results by `id` instead of array position
- If one entry is missing from the response, the others still apply
  (graceful partial failure instead of all-or-nothing)

### AC5: Handle vague titles explicitly
- Add to the prompt: *"If a title is too vague or cryptic to classify (reference
  numbers, generic payment descriptions), set confidence to 'low'."*

### AC6: Set temperature on all providers
- Anthropic: add `temperature: 0.1` to the request body
- Ollama: add `"options": {"temperature": 0.1}` to the request body
- Keep OpenAI at existing `0.1`

### AC7: Display confidence in the frontend
- Extend the LLM badge in `BulkUpload.svelte` to show confidence:
  - High — green tint, auto-suggested
  - Medium — yellow tint, review recommended
  - Low — red tint, flagged for manual input
- Consider sorting LLM-classified rows so low-confidence ones appear first
  (review workflow)

### AC8: Surface LLM errors to the user
- If the LLM API call fails, return error info in the response (not just silent
  fallback to unclassified)
- Frontend shows a dismissible warning: *"LLM classification failed: {reason}.
  X expenses left unclassified."*

## Implementation Notes

- AC1 + AC2 + AC3 + AC5 are all prompt changes in `build_classification_prompt()`
  and `parse_classification_response()` — they form one natural unit of work
- AC4 (keyed IDs) changes the parsing contract and should be done together with AC1
- AC6 is a one-liner per provider
- AC7 requires passing `confidence` through `ClassifiedExpenseRow` to the frontend
- AC8 requires a new field on the IPC response or a separate error channel

## Priority

| AC  | Effort  | Impact | Notes                                      |
|-----|---------|--------|--------------------------------------------|
| AC1 | Medium  | High   | Core improvement — confidence visibility   |
| AC2 | Trivial | High   | Better accuracy, one-line prompt change     |
| AC3 | Trivial | Medium | Prevents long-term category drift          |
| AC4 | Low     | Medium | Resilience — partial results beat none     |
| AC5 | Trivial | Medium | Part of prompt rewrite                     |
| AC6 | Trivial | Low    | Consistency across providers               |
| AC7 | Medium  | High   | User-facing payoff of AC1                  |
| AC8 | Low     | Low    | Better UX for troubleshooting              |
