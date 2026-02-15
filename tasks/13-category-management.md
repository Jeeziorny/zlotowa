# Task 13: Category Management Page

**Track:** Full-stack — New feature (frontend + backend)
**Blocked by:** nothing
**Blocks:** nothing

## Problem

Users have no way to see, create, rename, merge, or delete their categories. Categories currently exist implicitly — they are just strings stored in `classification_rules.category` and `expenses.category`. There is no dedicated UI to manage them. A client demo surfaced this as a needed feature.

The `get_categories` IPC command (`src-tauri/src/lib.rs:113`) returns distinct category names from `classification_rules`, but there are no commands for creating standalone categories, renaming across tables, merging, or deleting with reassignment.

## Current State

### Database (`crates/core/src/db.rs`)

Categories are **not a first-class entity** — they're just strings in two tables:

- `classification_rules` table (line 66-70): `pattern TEXT NOT NULL UNIQUE, category TEXT NOT NULL`
- `expenses` table (line 54-61): `category TEXT` (nullable)

Relevant methods:
- `get_all_categories()` (line 215-221): `SELECT DISTINCT category FROM classification_rules ORDER BY category` — only looks at rules, not expenses
- `get_all_rules()` (line 201-213): returns all `ClassificationRule` rows
- `get_all_expenses()` (line 134-162): returns all `Expense` rows
- `insert_rule()` (line 176-182): `INSERT OR REPLACE` by pattern uniqueness

No methods exist for: updating category names across tables, deleting rules by category, counting expenses per category, or counting rules per category.

### Tauri IPC (`src-tauri/src/lib.rs`)

- `get_categories` (line 113-116): wraps `db.get_all_categories()`, returns `Vec<String>`
- No commands for category CRUD operations

### Frontend

- `App.svelte` (line 9): page routing via `currentPage` string state — pages: `dashboard`, `add`, `bulk`, `expenses`, `settings`
- `Sidebar.svelte` (line 4-9): navigation items array, Settings rendered separately at bottom (line 32-43)
- `AddExpense.svelte` (line 24): calls `get_categories` for autocomplete dropdown

### Models (`crates/core/src/models.rs`)

- `ClassificationRule` (line 64-69): `{ id: Option<i64>, pattern: String, category: String }`
- `Expense` (line 6-14): `{ id, title, amount, date, category: Option<String>, classification_source }`

## Scope

### 1. Add category stats queries to Database (`crates/core/src/db.rs`)

Add these methods to `impl Database`:

```rust
/// Returns (category_name, expense_count, rule_count) for all known categories.
/// Union of categories from both expenses and classification_rules tables.
pub fn get_category_stats(&self) -> Result<Vec<CategoryStats>, DbError>

/// Rename a category across both expenses and classification_rules tables.
/// Uses a transaction for atomicity.
pub fn rename_category(&self, old_name: &str, new_name: &str) -> Result<(), DbError>

/// Delete a category: reassign its expenses and rules to `replacement`.
/// Uses a transaction for atomicity.
pub fn delete_category(&self, category: &str, replacement: &str) -> Result<(), DbError>

/// Merge multiple source categories into a target category.
/// Moves all expenses and rules, then removes the source category names.
/// If `target` doesn't exist yet among the sources, creates it by renaming.
pub fn merge_categories(&self, sources: &[String], target: &str) -> Result<(), DbError>

/// Check if a category name already exists (in either rules or expenses).
pub fn category_exists(&self, name: &str) -> Result<bool, DbError>

/// Create a standalone category by inserting a placeholder rule or config entry.
pub fn create_category(&self, name: &str) -> Result<(), DbError>
```

Add a `CategoryStats` struct to `crates/core/src/models.rs`:
```rust
pub struct CategoryStats {
    pub name: String,
    pub expense_count: i64,
    pub rule_count: i64,
}
```

Key implementation notes:
- `get_category_stats` should union categories from both `expenses` and `classification_rules` to catch categories that exist in expenses but have no rules (e.g., manually typed ones)
- `rename_category` runs two UPDATEs in a transaction: `UPDATE expenses SET category = ?2 WHERE category = ?1` and `UPDATE classification_rules SET category = ?2 WHERE category = ?1`
- `delete_category` is essentially rename-to-replacement + nothing extra (the old category ceases to exist once all references are updated)
- `merge_categories` updates all source categories to target in both tables, in a single transaction
- `create_category`: insert a row into `classification_rules` with a placeholder pattern (e.g., `__category_placeholder__{name}`) and the category name — or use config table. This ensures `get_all_categories` picks it up. Consider a cleaner approach: add a `categories` config key storing a JSON array, or just accept that standalone categories without rules may not be common.

### 2. Add Tauri IPC commands (`src-tauri/src/lib.rs`)

```rust
#[tauri::command]
fn get_category_stats(state: State<AppState>) -> Result<Vec<CategoryStats>, String>

#[tauri::command]
fn create_category(state: State<AppState>, name: String) -> Result<(), String>

#[tauri::command]
fn rename_category(state: State<AppState>, old_name: String, new_name: String) -> Result<(), String>

#[tauri::command]
fn delete_category(state: State<AppState>, category: String, replacement: String) -> Result<(), String>

#[tauri::command]
fn merge_categories(state: State<AppState>, sources: Vec<String>, target: String) -> Result<(), String>
```

Register all five in the `invoke_handler!` macro (line 414-429).

Validation in IPC layer:
- `create_category`: check `db.category_exists(&name)` first, return error `"Category '{}' already exists"` if true
- `rename_category`: check new name doesn't already exist (unless it's a case-only change)
- `merge_categories`: validate `sources` is non-empty and `target` is non-empty

### 3. Add "Categories" to sidebar navigation (`src/lib/Sidebar.svelte`)

Add to the `items` array (line 4-9):
```js
{ id: "categories", label: "Categories", icon: "▤" }
```

### 4. Add route in App.svelte (`src/App.svelte`)

Add import for the new `Categories.svelte` component and a routing case:
```svelte
{:else if currentPage === "categories"}
  <Categories />
```

### 5. Create Categories page (`src/lib/Categories.svelte`)

New Svelte 5 component with these features:

**Layout:**
- Page title "Categories"
- Search bar at top for filtering by name
- "+" button to open create form
- Table with sortable columns: Name, Expenses, Rules
- Each row has a checkbox (for multi-select merge) and action buttons

**Category list table:**
- Fetches data via `invoke("get_category_stats")` on mount
- Columns: checkbox, Name (editable inline), Expense count, Rule count
- Click column headers to sort (toggle asc/desc)
- Search bar filters rows by name (client-side)

**Inline rename:**
- Click category name to turn it into a text input
- Press Enter or blur to save via `invoke("rename_category", { oldName, newName })`
- Press Escape to cancel
- Auto-cascades — no confirmation needed

**Create category:**
- "+" button opens a small modal/form with a text input
- On submit, calls `invoke("create_category", { name })`
- If category already exists, show error message (backend returns error string)
- On success, refresh the list

**Delete:**
- Delete button per row (e.g., trash icon)
- Opens a modal: "Select a replacement category for N expenses and M rules"
- Dropdown of remaining categories (exclude the one being deleted)
- Confirm button calls `invoke("delete_category", { category, replacement })`
- Refresh list on success

**Merge:**
- Checkboxes on each row for multi-select
- "Merge" button appears when 2+ categories are selected
- Opens a modal: "Merge N categories"
- User can pick one of the selected categories as survivor, OR type a new name
- Confirm calls `invoke("merge_categories", { sources, target })`
- Refresh list and clear selection on success

## Files to Change

| File | Change |
|---|---|
| `crates/core/src/models.rs` | Add `CategoryStats` struct |
| `crates/core/src/db.rs` | Add `get_category_stats`, `rename_category`, `delete_category`, `merge_categories`, `category_exists`, `create_category` methods |
| `src-tauri/src/lib.rs` | Add 5 new IPC commands, register in `invoke_handler!` |
| `src/lib/Sidebar.svelte` | Add "Categories" nav item |
| `src/App.svelte` | Add import and route for Categories page |
| `src/lib/Categories.svelte` | **New file** — full category management UI |

## Test Scenarios

### Backend (Rust unit tests in `crates/core/src/db.rs`)

1. `get_category_stats` returns correct counts from both expenses and rules tables
2. `get_category_stats` unions categories — a category only in expenses (no rule) still appears
3. `rename_category` updates both `expenses.category` and `classification_rules.category`
4. `rename_category` is atomic — if one UPDATE fails, neither applies
5. `delete_category` reassigns expenses and rules to replacement category
6. `delete_category` removes the old category from `get_category_stats` results
7. `merge_categories` moves all expenses and rules from sources to target
8. `merge_categories` with a new target name (not in sources) works correctly
9. `category_exists` returns true for categories in rules, true for categories only in expenses, false for nonexistent
10. `create_category` makes the category appear in `get_all_categories`
11. `create_category` for duplicate name returns an error

### Frontend (manual UI tests)

1. Navigate to Categories page — verify all existing categories appear with correct expense and rule counts
2. Click column header "Name" — rows sort alphabetically, click again for reverse
3. Click column header "Expenses" — rows sort by expense count descending
4. Type in search bar — list filters to matching categories (case-insensitive)
5. Click "+" → type "NewCategory" → submit — category appears in list with 0 expenses, 0 rules
6. Click "+" → type an existing category name → submit — error message "Category already exists" appears
7. Click a category name → edit to "RenamedCategory" → press Enter — name updates, counts stay same
8. Click a category name → press Escape — edit cancelled, original name restored
9. Click delete on a category with expenses → modal shows replacement dropdown → select replacement → confirm — category removed, counts on replacement increase
10. Select 3 categories via checkboxes → click Merge → pick one as survivor → confirm — 2 categories disappear, survivor has combined counts
11. Select 2 categories → click Merge → type new name → confirm — both old categories disappear, new one appears with combined counts
12. After any operation (create/rename/delete/merge), go to Add Expense — verify the autocomplete dropdown reflects the changes

## Acceptance Criteria

- Categories page is accessible from sidebar navigation
- Category list shows name, expense count, and rule count for every category (from both expenses and rules tables)
- List is searchable and sortable by all three columns
- New categories can be created; duplicate names are blocked with a visible error
- Categories can be renamed inline; rename cascades to all expenses and rules
- Categories can be deleted only after selecting a replacement; expenses and rules move to replacement
- Multiple categories can be merged into a survivor or a new name; all expenses and rules consolidate
- All database mutations (rename, delete, merge) are atomic (transactional)
- UI follows existing dark theme conventions (gray-950/900/800, emerald accents)
- Svelte 5 syntax used throughout (`$state`, `$derived`, `$props`, `onclick`)
