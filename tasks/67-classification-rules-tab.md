# Task 67 — Classification Rules Tab

## Summary
Add a "Rules" sidebar page (opt-in via Settings toggle) showing classification rules in a table with filtering, inline editing, delete, add-new-rule, and per-rule match counts. Power-user feature for inspecting and managing auto-generated regex rules.

## Design Decisions
- Separate sidebar item (visible only when toggled on in Settings)
- Columns: Pattern (monospace), Category (badge), Match Count (right-aligned)
- Inline click-to-edit for pattern and category
- Single-delete with confirmation modal
- Add-new-rule button with inline form
- Pagination (reuses PaginationBar)
- Filtering by pattern text + category dropdown

---

## Backend

### New model types — `crates/core/src/models.rs`
Add after `ClassificationRule` (~line 85):
- `RuleWithMatchCount { id: i64, pattern: String, category: String, match_count: i64 }`
- `RuleQuery { search: Option<String>, category: Option<String>, limit: Option<i64>, offset: Option<i64> }` (derive Default)
- `RuleQueryResult { rules: Vec<RuleWithMatchCount>, total_count: i64 }`

### New DB methods — `crates/core/src/db.rs`
Add in classification rules section (~line 623):

1. **`delete_rule(id: i64) -> Result<(), DbError>`** — DELETE by id, error if not found
2. **`update_rule(rule: &ClassificationRule) -> Result<(), DbError>`** — UPDATE pattern+category by id, error if not found. Note: UNIQUE constraint on pattern may reject duplicates — surface error naturally.
3. **`query_rules(query: &RuleQuery) -> Result<RuleQueryResult, DbError>`** — dynamic WHERE (pattern LIKE for search, category = for filter), LIMIT/OFFSET. After fetching the page, load all expense titles and compute match counts via regex in Rust (bounded to ~50 rules per page).

Add tests: delete (success + not-found), update (success + not-found), query (no filters, search, category filter, pagination, match counts).

### New IPC commands — `src-tauri/src/lib.rs`
Four commands, standard pattern (lock mutex → call DB → map_err):
- `query_rules(query: RuleQuery) -> RuleQueryResult`
- `add_rule(pattern: String, category: String) -> i64` (reuses `db.insert_rule()`)
- `update_rule(id: i64, pattern: String, category: String)`
- `delete_rule(id: i64)`

Register all four in `invoke_handler!`.

---

## Frontend

### New components — `src/lib/rules/`

**`RulesFilterBar.svelte`** — search input (debounced 300ms, searches pattern text) + category select + clear button. Simplified `expense-list/SearchFilterBar.svelte`.

**`RulesTable.svelte`** — table with inline click-to-edit (same `editingId` pattern as `ExpenseTable.svelte`). Category input uses datalist for autocomplete. Edit/delete actions visible on hover.

**`RuleDeleteModal.svelte`** — confirmation modal (same structure as `expense-list/DeleteConfirmModal.svelte`), shows pattern and category.

### Rules page — `src/lib/Rules.svelte`
Parent component (same structure as `ExpenseList.svelte`): state for rules, totalCount, loading, filters, pagination, deleteModal, add-form. Fetches via `invoke("query_rules", { query })`. Add-rule: collapsible inline form with pattern + category inputs. Reuses `expense-list/PaginationBar.svelte`.

### Existing file modifications

**`src/lib/expense-list/PaginationBar.svelte`** — add optional `label` prop (default `"expense"`) to replace hardcoded text on line 17.

**`src/lib/Sidebar.svelte`** — accept `showRules` prop (default false), `$derived` items array conditionally includes `{ id: "rules", label: "Rules", icon: "⚡" }`.

**`src/lib/Settings.svelte`** — add `showRulesTab` state from config key `"show_rules_tab"`, checkbox card toggle, `onrulesvisibilitychange` callback prop.

**`src/App.svelte`** — import Rules, `showRules` state from config on mount, pass to Sidebar, add route, pass callback to Settings. If toggled off while on Rules page → navigate to dashboard.

---

## File Summary

| File | Action |
|------|--------|
| `crates/core/src/models.rs` | Modify — add 3 structs |
| `crates/core/src/db.rs` | Modify — add 3 methods + tests |
| `src-tauri/src/lib.rs` | Modify — add 4 IPC commands |
| `src/lib/rules/RulesFilterBar.svelte` | Create |
| `src/lib/rules/RulesTable.svelte` | Create |
| `src/lib/rules/RuleDeleteModal.svelte` | Create |
| `src/lib/Rules.svelte` | Create |
| `src/lib/expense-list/PaginationBar.svelte` | Modify — add `label` prop |
| `src/lib/Sidebar.svelte` | Modify — conditional rules item |
| `src/lib/Settings.svelte` | Modify — add toggle |
| `src/App.svelte` | Modify — routing + state |

## Verification
1. `cargo test -p accountant-core` — new DB method tests pass
2. `npm run tauri dev` — toggle rules in Settings, sidebar item appears/disappears
3. Rules page: filtering, pagination, inline edit, add, delete all work
4. Edit a rule's category → future classifications use new category
5. Toggle off while on Rules page → navigates to dashboard
