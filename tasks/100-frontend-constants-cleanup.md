# Task 100: Frontend Constants & Minor Cleanup

## Goal

Extract hardcoded magic numbers to named constants, fix minor code quality issues in frontend.

## Deliverables

### 1. Use `UNCATEGORIZED` constant — `db.rs:407-414`

Replace raw string `"uncategorized"` comparison with the existing `UNCATEGORIZED` constant from `models.rs`.

### 2. Tighten `RegexClassifier::rules` visibility — `classifiers.rs:34`

Change `pub rules` to `pub(crate) rules` (or private with a getter if needed externally).

### 3. Extract magic numbers to constants

| File | Value | Suggested constant |
|------|-------|--------------------|
| `src/lib/BulkUpload.svelte:43` | `2800` | `QUIP_INTERVAL_MS` in component scope |
| `src/lib/bulk-upload/TitleCleanupStep.svelte:170` | `20` | `MAX_RECENT_CLEANUPS` in `constants.js` |
| `src/lib/widgets/DailySpending.svelte:134` | `"#6b7280"` | `CHART_GRAY` in `constants.js` |
| `src/lib/widgets/DayOfWeek.svelte:54` | `"#fbbf24"` | `CHART_HIGHLIGHT` or reuse from shared palette |

### 4. Fix `$state` misuse — `src/lib/rules/RulesFilterBar.svelte:12`

Change `let debounceTimer = $state(null)` to `let debounceTimer = null`. Timer IDs don't drive rendering and don't need reactivity. Matches the pattern in `SearchFilterBar.svelte:19`.

## Files to modify
- `crates/core/src/db.rs`
- `crates/core/src/classifiers.rs`
- `src/lib/BulkUpload.svelte`
- `src/lib/bulk-upload/TitleCleanupStep.svelte`
- `src/lib/widgets/DailySpending.svelte`
- `src/lib/widgets/DayOfWeek.svelte`
- `src/lib/rules/RulesFilterBar.svelte`
- `src/lib/constants.js` (if adding shared chart color constants)

## Notes
- Check `src/lib/constants.js` for existing constants before adding — avoid duplication.
- The chart color constants may only be used in one place each; in that case, a component-level `const` is fine.
