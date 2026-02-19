# 47 — Docs Sync Round 2

## Problem

Task 37 synced CLAUDE.md IPC commands with code, but the audit found more drift.

### CLAUDE.md gaps

1. **Line 53** — "Key commands" list is still missing 8 commands: `update_expense`, `delete_expense`, `delete_expenses`, `get_category_stats`, `create_category`, `rename_category`, `delete_category`, `merge_categories`

2. **Architecture section** — `ical` module (`crates/core/src/ical.rs`) is completely undocumented. Contains `parse_ics()`, `filter_events_by_date_range()`, `ParsedCalendarEvent`.

3. **Line 42-47** — Trait method lists incomplete:
   - `Parser` missing `name()`
   - `Classifier` missing `name()` and `priority()`
   - `Exporter` missing `name()` and `extension()`

### User docs (mdBook) drift

4. **`docs/src/features/budget-planning.md:13`** — Says "monthly spending limits". Implementation uses arbitrary date ranges (`start_date`/`end_date`). The DB was migrated from year/month.

5. **`docs/src/features/dashboard.md:14`** — Says "Current month's budget vs. actual spending". Should say "active budget" since budgets are date-range-based.

6. **`docs/src/features/llm-config.md:18`** — Says "Claude Haiku" without specifying version. Code uses `claude-haiku-4-5-20251001` (Claude Haiku 4.5).

7. **`docs/src/features/llm-config.md:57-58`** — Says clearing config via CLI `4ccountant llm-conf`. The CLI command has no "clear" option — it always prompts to set a new provider.

8. **`docs/src/features/export.md:14`** — Says classification source display is "DB Rule, LLM, or Manual". Actual Display impl outputs "Database", "Llm", "Manual".

## Scope

- Update CLAUDE.md: add missing commands, document `ical` module, complete trait method lists
- Update budget docs: replace "monthly" with "date-range" language throughout
- Update dashboard docs: "active budget" instead of "current month's"
- Update LLM docs: specify Claude Haiku 4.5, fix CLI clear instructions
- Update export docs: correct classification source display strings
