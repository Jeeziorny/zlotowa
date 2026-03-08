# Task 96: Docs Drift Cleanup

## Goal

Fix documentation that references removed features or has outdated counts.

## Deliverables

### 1. CLI docs — `docs/src/cli.md`

Remove references to 3 non-existent commands: `insert`, `export`, `dashboard`. Only document the 4 actual commands: `llm-conf`, `bulk-insert`, `backup`, `restore`. Verify against `crates/cli/src/main.rs`.

### 2. Budget planning docs — `docs/src/features/budget-planning.md`

Remove "Planned expenses" from the feature list. The `planned_expenses` table was dropped in a migration (`db.rs:239`). The Overview tab no longer has this feature.

### 3. CLAUDE.md command count — `CLAUDE.md:53`

Update "37 commands grouped by domain" to the actual count. Enumerate all commands by grepping `#[tauri::command]` in `src-tauri/src/lib.rs` and count them. Update the grouped list to include any missing commands (likely `query_rules`, `add_rule`, `delete_rule`, `update_rule`, and others added after the docs were written).

### 4. Verify

Scan the rest of `docs/src/` for any other references to removed features (planned expenses, title cleanup rules table, display_title column).

## Files to modify
- `docs/src/cli.md`
- `docs/src/features/budget-planning.md`
- `CLAUDE.md`
- Any other `docs/src/` files with stale references

## Notes
- Run `mdbook build docs` after changes to verify no broken links.
