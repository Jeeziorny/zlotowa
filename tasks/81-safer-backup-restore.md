# Task 81 — Safer Backup Restore Flow

Item: #20 (restore is too easy to trigger destructively).

## Problem

Restoring a backup immediately overwrites the entire database after a single confirmation modal. There's no preview of what the backup contains, no warning about data loss severity, and no option to back up current data first. This is a high-stakes action with a low-effort gate.

## Solution

### Two-step restore flow

**Step 1 — Preview:**

After the user selects a backup file, parse it and display a summary before any destructive action:

```
This backup contains:
  342 expenses
  28 classification rules
  15 categories
  2 budgets
  Created: 2025-12-15 10:30
```

Display in a card with `bg-gray-800 border border-gray-700 rounded-lg p-4`. Use a simple list with counts.

**Step 2 — Warning + Confirm:**

Below the preview, show a prominent warning block:
```html
<div class="bg-red-900/20 border border-red-800/50 rounded-lg p-4 mt-4">
  <p class="text-red-300 font-medium">⚠ This will replace ALL current data</p>
  <p class="text-red-400/80 text-sm mt-1">This action cannot be undone. Your current expenses, rules, categories, and budgets will be permanently replaced.</p>
</div>
```

Add a convenience button: `"Back up current data first"` — runs the existing backup flow before proceeding. Styled as `bg-gray-800 hover:bg-gray-700 text-sm`.

Add a confirmation checkbox: `"I understand this will replace all my data"`. The "Restore" button is disabled until the checkbox is checked.

### Backend: preview_backup command

Add a new IPC command `preview_backup` that reads and parses the backup file but does NOT apply it. Returns a summary:

```rust
#[derive(Serialize)]
struct BackupPreview {
    expense_count: usize,
    rule_count: usize,
    category_count: usize,
    budget_count: usize,
    created_at: Option<String>,
}
```

Implementation: read the JSON file, deserialize just enough to count entries, return the preview. Reuse the existing backup format parsing.

### Frontend flow

1. User clicks "Restore from Backup" → file dialog opens
2. File selected → call `preview_backup` → show preview card
3. Preview card shows: summary + warning + "backup first" button + confirmation checkbox
4. Checkbox checked → "Restore" button enabled
5. Click "Restore" → call existing `restore_database` → show result

The preview replaces the current instant-modal pattern with an inline expandable section inside `BackupRestore.svelte`.

## Decision Needed

**Confirmation gate strength:**
1. Checkbox only: `"I understand this will replace all my data"` — simple, low friction
2. Type-to-confirm: user must type `RESTORE` in an input field — higher friction, prevents muscle-memory accidents
3. Both: checkbox + 3-second delay on the Restore button — medium friction

## Files

| File | Action |
|------|--------|
| `crates/core/src/backup.rs` | Modify — add `preview_backup()` function |
| `src-tauri/src/lib.rs` | Modify — add `preview_backup` IPC command |
| `src/lib/settings/BackupRestore.svelte` | Modify — two-step flow with preview, warning, checkbox |

## Verification
1. Select backup file → preview appears with counts (no data changed yet)
2. Warning block is prominently visible
3. "Restore" button disabled until checkbox checked
4. "Back up current data first" runs backup, then re-enables restore flow
5. Restore succeeds with same behavior as before
6. Cancel at any point → no data changed
7. `cargo test` — preview_backup function works with valid and invalid files
