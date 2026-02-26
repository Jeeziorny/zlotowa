# 54 — Auto-suggest Title Cleanup During Bulk Upload

## Problem

Users import messy bank CSVs, then have to manually go to Title Cleanup to fix titles. This should be suggested automatically during import.

## Dependencies

- **Task 53** — needs `display_name` field to store cleaned versions without overwriting raw titles

## Changes

### Bulk Upload Step 3 (Review)

After classification completes:

1. Run all existing cleanup rules against new expense titles
2. For matching expenses, show inline suggestion: original title with strikethrough + cleaned version
3. User can accept/reject per-expense or "Accept all suggestions"
4. Accepted cleanups populate `display_name` on save (requires Task 53)

### Classification Order

Classify on raw `title` FIRST, then suggest `display_name` cleanups. This keeps regex classification rules stable.

### Backend

- New IPC command or extend `parse_and_classify` to return cleanup suggestions
- New method in `db.rs` to batch-preview cleanup rules against a list of titles

## Scope

- `src/lib/BulkUpload.svelte` — Step 3 UI additions (suggestion display, accept/reject controls)
- `src-tauri/src/lib.rs` — new IPC command or extend existing
- `crates/core/src/db.rs` — method to batch-preview cleanup rules against a list of titles
