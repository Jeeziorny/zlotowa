# Task 28: Title Cleanup Explanation & Bulk Upload Integration

**Track:** Frontend — UX improvement
**Priority:** LOW
**Blocked by:** nothing
**Blocks:** nothing

## Problem

Client feedback:
1. **No explanation** — The Title Cleanup page doesn't explain what it is or how it works. The only hint is in the empty state message which disappears once rules are added.
2. **Bulk upload integration unclear** — Do title cleanup rules apply to freshly uploaded expenses in Bulk Upload? If not, should they?

## Current State

### TitleCleanup.svelte
- Empty state shows: "Add rules to strip noise from bank transaction titles — card numbers, payment codes, etc."
- Once rules exist, there's no persistent description or help text
- Form labels: "Pattern (text to find)" and "Replacement (empty = remove match)"

### Bulk Upload Integration
- Need to verify: does `parse_and_classify` in the backend apply title cleanup rules, or are they only applied manually from the Title Cleanup page?

## Scope

### 1. Add Persistent Help Text
- Add a description section at the top of the Title Cleanup page (visible even when rules exist):
  - What it does: "Clean up messy bank transaction titles by defining find-and-replace rules."
  - How it works: "Rules match text in expense titles and replace it (or remove it if replacement is empty). Use literal text for exact matches or regex for patterns."
  - Example: e.g. pattern `CARD *1234` → replacement `` (empty) strips card numbers

### 2. Clarify/Implement Bulk Upload Integration
- Investigate whether title cleanup runs during bulk upload
- If not: add a note on the Title Cleanup page saying "Rules apply when you click 'Preview & Apply'. They do not automatically apply to new uploads."
- If client wants auto-apply on upload: that's a separate scope item (flag but don't implement here)

## Files to Change

| File | Change |
|---|---|
| `src/lib/TitleCleanup.svelte` | Add persistent help/explanation section at top of page |

## Acceptance Criteria

- Title Cleanup page has a clear, always-visible explanation of what it does and how it works
- Integration with bulk upload is documented (either in UI or as a known limitation)
