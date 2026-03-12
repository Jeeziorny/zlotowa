# Task 104: E2E Playwright Tests

**Track:** Testing — E2E test infrastructure + test suite (frontend)
**Blocked by:** nothing
**Blocks:** nothing

## Problem

The app has zero frontend tests. All 37 IPC commands, 7 pages, and every interactive flow (add expense, bulk upload wizard, categories CRUD, budget planning, dashboard customization, settings, keyboard shortcuts) are untested from the user's perspective. Rust unit/integration tests cover the core logic, but nothing verifies the Svelte UI works end-to-end.

There's a stale `playwright-report/` directory suggesting Playwright was tried before, but no config, no test files, and no test dependencies exist in the project.

## Current State

### No test infrastructure

`package.json` (lines 13–18) has no test script and no test dependencies:
```json
"scripts": {
  "dev": "vite",
  "build": "vite build",
  "preview": "vite preview",
  "tauri": "tauri"
}
```

No `playwright.config.js`, no `tests/` directory, no mock for `@tauri-apps/api/core`.

### Frontend architecture

**Routing** — `src/App.svelte` lines 17–18: string-based SPA with `currentPage` and `expenseSubView` state variables. Pages: dashboard, expenses (list/add/bulk sub-views), categories, budget, rules, settings.

**IPC** — Every component imports `invoke` from `@tauri-apps/api/core` and calls Tauri commands directly. Example from `AddExpense.svelte` line 2:
```javascript
import { invoke } from "@tauri-apps/api/core";
```

**State** — Svelte 5 runes: `$state()`, `$derived()`, `$props()`. No global store — each component fetches its own data via `invoke()` in `onMount`.

### Key files per page

| Page | Component | Key invoke calls |
|------|-----------|-----------------|
| Dashboard | `src/lib/Dashboard.svelte` | `get_expenses`, `get_active_widgets`, `save_active_widgets` |
| Add Expense | `src/lib/AddExpense.svelte` | `add_expense`, `suggest_category`, `get_categories` |
| Expense List | `src/lib/ExpenseList.svelte` | `query_expenses`, `update_expense`, `delete_expense`, `delete_expenses` |
| Bulk Upload | `src/lib/BulkUpload.svelte` + `src/lib/bulk-upload/*.svelte` | `preview_csv`, `parse_and_classify`, `bulk_save_expenses`, `bulk_save_rules` |
| Categories | `src/lib/Categories.svelte` | `get_category_stats`, `create_category`, `rename_category`, `delete_category`, `merge_categories` |
| Budget | `src/lib/budget/BudgetPlanning.svelte` | `list_budgets`, `create_budget`, `get_budget_summary`, `delete_budget`, `get_category_averages` |
| Settings | `src/lib/Settings.svelte` + `src/lib/settings/*.svelte` | `get_config`, `save_config`, `get/save/validate/clear_llm_config` |
| Rules | `src/lib/Rules.svelte` | `query_rules` |

## Scope

### Sub-task 1: Install Playwright and create config

- `npm install -D @playwright/test`
- Create `playwright.config.js` — use `webServer` to start `npm run dev` on port 1420, single Chromium browser
- Add scripts to `package.json`: `"test:e2e": "playwright test"`, `"test:e2e:ui": "playwright test --ui"`

### Sub-task 2: Create Tauri invoke mock

Since we're testing against the Vite dev server (not the full Tauri app), `@tauri-apps/api/core` won't be available. We need a mock layer.

**Option A — Vite alias + mock module:** Create `tests/mocks/tauri.js` that exports a mock `invoke()` function. Configure `vite.config.js` to alias `@tauri-apps/api/core` to the mock in test/dev mode. The mock holds an in-memory state (expenses array, categories array, config map, etc.) and handles each command name with simple JS logic.

**Option B — Playwright `page.addInitScript` + `window.__TAURI__` injection:** Before each test, inject a script that stubs `window.__TAURI_INTERNALS__` so that `invoke()` resolves with test data. This avoids touching Vite config but is more fragile.

Recommend **Option A** — it's more maintainable and lets `npm run dev` work without the Tauri backend too (useful for pure frontend development).

The mock should support:
- `get_expenses` / `query_expenses` / `add_expense` / `update_expense` / `delete_expense` / `delete_expenses` — in-memory array CRUD
- `get_categories` / `get_category_stats` / `create_category` / `rename_category` / `delete_category` / `merge_categories` — in-memory categories
- `get_config` / `save_config` — in-memory key-value map
- `suggest_category` — return empty string or fixed value
- `preview_csv` / `parse_and_classify` / `bulk_save_expenses` / `bulk_save_rules` — return canned responses
- `list_budgets` / `create_budget` / `get_budget_summary` / `delete_budget` / `check_budget_overlap` / `get_category_averages` / `save_budget_categories` — basic budget logic
- `get_active_widgets` / `save_active_widgets` — in-memory widget state
- `get_llm_config` / `save_llm_config` / `validate_llm_config` / `clear_llm_config` — in-memory LLM config
- `backup_database` / `restore_database` — no-op or canned
- `get_upload_batches` / `delete_batch` — in-memory batches
- `query_rules` — return empty or canned rules

Provide a `resetMockState()` to clear between tests and a `seedData(overrides)` to set initial state.

### Sub-task 3: Test — Add Expense

File: `tests/add-expense.spec.js`

```
- Navigate to Add Expense (sidebar or Cmd+N)
- Fill title, amount, date → submit → success toast appears → form resets
- Submit with empty title → error shown
- Submit with zero amount → error shown
- Type title → category suggestion appears (mock suggest_category to return "Food") → click applies it
- Add expense → navigate to expense list → new expense visible
```

### Sub-task 4: Test — Expense List

File: `tests/expense-list.spec.js`

```
- Seed 10+ expenses → page loads → all visible (respecting pagination)
- Type in search → list filters by title
- Select category filter → list narrows
- Set date range → correct subset shown
- Set amount min/max → correct subset shown
- Click "Clear all" → full list restored
- Change page size → pagination updates
- Navigate pages → correct items shown
```

### Sub-task 5: Test — Inline Edit & Delete

File: `tests/expense-crud.spec.js`

```
- Click edit on expense → fields become editable → change title → save → list updates
- Click edit → cancel → no changes
- Select one expense → delete → confirm modal → expense removed
- Select 3 expenses → bulk delete → confirm → all 3 removed
- Select all → bulk delete → confirm → list empty
```

### Sub-task 6: Test — Bulk Upload Wizard

File: `tests/bulk-upload.spec.js`

```
- Navigate to bulk upload
- Upload a CSV file (use Playwright file chooser mock) → preview shown
- Map columns → next
- Title cleanup: add find/replace → apply → titles updated → next
- Review classified: mock returns mix of classified/unclassified → manually assign category → save
- Review rules step: see auto-generated rules → save
- Done step: success message shown
- Navigate to expense list → imported expenses visible
```

### Sub-task 7: Test — Categories

File: `tests/categories.spec.js`

```
- Create new category → appears in table
- Rename category → name updates
- Delete category → pick replacement → category removed, expenses reassigned
- Select 2 categories → merge → target category has combined count
- Search → filters category list
```

### Sub-task 8: Test — Budget Planning

File: `tests/budget.spec.js`

```
- Create budget with name, dates, category limits → appears in overview
- Navigate between budgets with arrows
- Delete budget → removed from list
- Create overlapping budget → error shown
```

### Sub-task 9: Test — Dashboard Widgets

File: `tests/dashboard.spec.js`

```
- Dashboard loads with default widgets
- Enter edit mode → "Add Widget" button visible
- Add a widget → appears on dashboard
- Remove a widget → disappears
- Reorder with arrow buttons → order changes
- Exit edit mode → save → reload page → order persisted
```

### Sub-task 10: Test — Settings

File: `tests/settings.spec.js`

```
- Toggle "Show Rules tab" → sidebar updates (rules tab appears/disappears)
- LLM tab: enter API key + model → save → reload → values persisted (masked)
- LLM tab: clear config → fields empty
```

### Sub-task 11: Test — Keyboard Shortcuts

File: `tests/keyboard-shortcuts.spec.js`

```
- Cmd+N → navigates to Add Expense
- Cmd+U → navigates to Bulk Upload
- Cmd+K → navigates to Expense List, search focused
- Cmd+1 → Dashboard
- Cmd+2 → Expenses
- Cmd+3 → Categories
- Cmd+4 → Budget
- Escape from sub-view → goes back
```

### Sub-task 12: Test — Navigation Guards

File: `tests/navigation-guards.spec.js`

```
- Start bulk upload, reach step 2 → click sidebar nav → unsaved changes modal appears
- Click "Leave" → navigates away
- Click "Cancel" → stays on bulk upload
```

## Files to Change

| File | Change |
|------|--------|
| `package.json` | Add `@playwright/test` devDependency, add `test:e2e` and `test:e2e:ui` scripts |
| `playwright.config.js` | New — Playwright configuration (webServer, browser, baseURL) |
| `tests/mocks/tauri.js` | New — Mock `invoke()` with in-memory state for all 37 commands |
| `vite.config.js` | Add conditional alias: `@tauri-apps/api/core` → mock module when `MOCK_TAURI=1` |
| `tests/add-expense.spec.js` | New — Add Expense tests (6 cases) |
| `tests/expense-list.spec.js` | New — Expense List search/filter/pagination tests (8 cases) |
| `tests/expense-crud.spec.js` | New — Inline edit + single/bulk delete tests (5 cases) |
| `tests/bulk-upload.spec.js` | New — Full 6-step wizard test (8 cases) |
| `tests/categories.spec.js` | New — Category CRUD + merge tests (5 cases) |
| `tests/budget.spec.js` | New — Budget create/navigate/delete/overlap tests (4 cases) |
| `tests/dashboard.spec.js` | New — Widget add/remove/reorder/persist tests (6 cases) |
| `tests/settings.spec.js` | New — Settings toggle/LLM config tests (3 cases) |
| `tests/keyboard-shortcuts.spec.js` | New — Global shortcut tests (7 cases) |
| `tests/navigation-guards.spec.js` | New — Unsaved changes modal tests (3 cases) |
| `.gitignore` | Add `playwright-report/`, `test-results/` |

## Test Scenarios

All tests are E2E Playwright tests running against the Vite dev server with mocked Tauri IPC.

### Critical path (must pass before merge)

1. **Add expense happy path:** Fill form → submit → toast → form clears → expense in list
2. **Add expense validation:** Empty title → error; zero amount → error
3. **Expense list loads:** Seed data → page shows expenses with correct count
4. **Search filters:** Type "grocery" → only matching expenses shown
5. **Inline edit:** Edit title → save → new title in list
6. **Single delete:** Delete → confirm → removed
7. **Bulk delete:** Select 3 → delete → confirm → all gone
8. **Bulk upload full flow:** CSV → columns → cleanup → classify → rules → done → expenses saved
9. **Create category:** Name → submit → in list
10. **Delete category with reassignment:** Delete → pick target → expenses moved
11. **Merge categories:** Select 2 → merge → single category remains
12. **Create budget:** Name + dates + limits → appears in overview
13. **Dashboard widget add/remove:** Edit mode → add → visible; remove → gone

### Important (should pass)

14. **Category filter in expense list:** Select category → matching expenses only
15. **Date/amount filters:** Set range → correct subset
16. **Pagination:** Change page size, navigate → correct slices
17. **Keyboard shortcuts:** Each Cmd+key → correct page
18. **Navigation guard:** Dirty bulk upload → sidebar click → modal shown
19. **Settings rules toggle:** Toggle on → rules tab in sidebar
20. **Budget overlap rejection:** Create overlapping → error

### Nice-to-have

21. **Dashboard widget reorder persistence:** Reorder → reload → same order
22. **Bulk upload duplicate detection:** Same CSV twice → duplicates flagged
23. **Category autocomplete keyboard navigation:** Arrow keys + Enter
24. **LLM config save/clear cycle:** Save → reload → masked key visible → clear → empty

## Acceptance Criteria

- `npm run test:e2e` runs all tests and exits cleanly
- All critical-path tests (1–13) pass
- Tests run against Vite dev server on port 1420 with mocked Tauri IPC (no Rust backend needed)
- Mock covers all 37 IPC commands with at minimum stub responses
- Each test is independent — `resetMockState()` between tests, no ordering dependencies
- Tests complete in under 60 seconds total (single Chromium browser)
- `playwright-report/` and `test-results/` are gitignored
- No changes to production source code required (mock is injected via Vite alias, gated behind env var)
