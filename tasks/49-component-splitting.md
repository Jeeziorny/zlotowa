# 49 — Split Large Svelte Components

## Problem

Several components have excessive script logic, making them hard to maintain and test.

### Candidates

1. **`src/lib/BulkUpload.svelte`** — 327 lines of script logic managing 4 distinct steps:
   - File input & format detection
   - Column mapping with popover assignment
   - Review classified expenses with category editing
   - Done state with summary

   Should be split into per-step sub-components (e.g., `BulkInput`, `ColumnMapping`, `ReviewClassified`, `BulkDone`).

2. **`src/lib/ExpenseList.svelte`** — 274 lines of script logic handling:
   - Search, filtering, pagination
   - Export modal
   - Inline editing
   - Single delete + batch delete modals

   Consider extracting export modal, inline edit row, and delete modals.

3. **`src/lib/Settings.svelte`** — 131 lines managing two unrelated features:
   - LLM configuration
   - Upload batch history

   Could be split into `LlmSettings` and `UploadHistory`.

### Also: category placeholder hack

**`crates/core/src/db.rs:862-868`** — `create_category()` inserts a fake classification rule with pattern `__category_placeholder__<name>`. This conflates the rules table with a categories list. A dedicated `categories` table would be cleaner (but is a larger change).

## Scope

- Split `BulkUpload.svelte` into 3-4 step sub-components with shared state
- Extract delete modals and export modal from `ExpenseList.svelte`
- Split `Settings.svelte` into `LlmSettings` + `UploadHistory`
- Consider a `categories` table (may be deferred)
