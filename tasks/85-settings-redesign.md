# Task 85: Settings Page Redesign — Grouped Tabs

## Problem

The Settings page is a flat stack of 4 unrelated cards (LLM config, Upload History, Backup & Restore, Advanced toggle) with no hierarchy. Everything has equal visual weight, the page scrolls long, and unrelated concerns are mixed together. It feels like a junk drawer.

## Solution

Replace the flat card stack with a tabbed layout grouping related settings:

- **General** — Rules tab toggle, and future preferences
- **LLM** — Provider config + privacy warning
- **Data** — Upload History + Backup & Restore

Tab bar at the top of the Settings page. Each tab shows only its section's content.

## UX Issues to Fix Along the Way

### Layout
- `max-w-lg` is too narrow — widen to `max-w-2xl`
- "Advanced" section is a full card for a single checkbox — demote it to an inline item in the General tab

### LLM Configuration
- "Save" shows toast + clears inline message + reveals "Current Configuration" card — too many feedback channels, pick one (toast)
- "Current Configuration" card is redundant — the form already shows provider and key
- Privacy warning only appears after saving — should show when configuring
- Provider value displayed raw (`openai`) instead of display name (`OpenAI`)
- "Save" and "Test Connection" do nearly the same thing — consider collapsing or making Test a secondary link

### Upload History
- No pagination or scroll constraint — 50+ batches push everything off-screen
- "Undo" label is misleading for a destructive delete — rename to "Delete" or "Remove"
- Success/error messages never auto-dismiss

### Backup & Restore
- Restore warning says "replace ALL current data" but the backend actually merges/skips duplicates — misleading and more alarming than necessary
- "Back up current data first" button is easy to miss — make it more prominent

### General
- No keyboard submit (Enter) on the LLM form
- Amber accent in Settings vs emerald elsewhere — consider aligning (amber is fine if intentional post-task-82)

## Files to Modify

- `src/lib/Settings.svelte` — add tab bar, restructure layout
- `src/lib/settings/LlmSettings.svelte` — remove redundant config card, move privacy warning, clean up feedback
- `src/lib/settings/UploadHistory.svelte` — add max-height/scroll, rename "Undo", auto-dismiss messages
- `src/lib/settings/BackupRestore.svelte` — fix restore warning copy
