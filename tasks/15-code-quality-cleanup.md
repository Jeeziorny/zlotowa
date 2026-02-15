# Task 15: Code Quality Cleanup — Fix Audit Smells

**Track:** Engineering hygiene (backend + frontend)
**Blocked by:** Tasks 9–14 (do this after all feature work is done)
**Blocks:** nothing

## Problem

A codebase audit identified several code smells that don't block feature development but accumulate maintenance risk over time. The issues fall into five categories:

1. **Panicking code in production paths** — `.expect()` in the Tauri entry point and direct array indexing (`rows[i]`) in `parse_and_classify` will crash the app instead of returning errors.

2. **N+1 duplicate detection** — bulk upload checks each expense for duplicates with an individual `SELECT COUNT(*)`, turning a 100-expense import into 100+ queries.

3. **Silent error swallowing** — bare `catch {}` blocks in the frontend and `.ok()` on directory creation hide failures from users and developers.

4. **Hardcoded string where enum exists** — `"Llm".to_string()` used instead of `ClassificationSource::Llm.to_string()`.

5. **Missing DB indices** — `category` and `classification_source` columns are queried/grouped but not indexed.

6. **Duplicated helper function** — `make_classification_rule()` / `make_rule()` is copy-pasted between `src-tauri/src/lib.rs` and `crates/cli/src/main.rs`.

**Note:** Line numbers below reflect the codebase as of the audit. Tasks 9–14 will modify some of these files. Use function names and patterns (not line numbers) to locate the code when implementing. Each sub-task describes what to search for.

## Current State

### Panicking code (`src-tauri/src/lib.rs`)

The `run()` function uses `.expect()` twice:
```rust
let db = Database::open_default().expect("Failed to open database");
// ...
.run(tauri::generate_context!())
    .expect("error while running tauri application");
```

The `parse_and_classify` function uses direct array indexing on `rows`:
```rust
title: rows[i].title.clone(),
amount: rows[i].amount,
date: chrono::NaiveDate::parse_from_str(&rows[i].date, "%Y-%m-%d")
    .unwrap_or_else(|_| chrono::Local::now().date_naive()),
// ...
rows[*idx].category = Some(cat);
rows[*idx].source = Some("Llm".to_string());
```

The indices come from `unclassified_indices` which are valid by construction, but `.get()` would be defensive.

### N+1 queries (`src-tauri/src/lib.rs`, `crates/cli/src/main.rs`)

In `parse_and_classify`, duplicate checking happens inside a loop:
```rust
for (expense, class_result) in &classified {
    let is_dup = db
        .is_duplicate(&expense.title, expense.amount, &expense.date)
        .map_err(|e| e.to_string())?;
    // ...
}
```

The CLI `cmd_bulk_insert()` has the same pattern. Each call to `is_duplicate()` runs:
```sql
SELECT COUNT(*) FROM expenses WHERE title = ?1 AND amount = ?2 AND date = ?3
```

### Silent error swallowing

**`crates/core/src/db.rs`** — `Database::open_default()`:
```rust
std::fs::create_dir_all(parent).ok();  // silently ignores permission/disk errors
```

**`src/lib/AddExpense.svelte`** — two bare catches:
```js
} catch { suggestedCategory = ""; }   // in onTitleInput debounce
// ...
} catch {}                             // after refreshing categories
```

### Hardcoded string (`src-tauri/src/lib.rs`)

In the LLM fallback section of `parse_and_classify`:
```rust
rows[*idx].source = Some("Llm".to_string());
```
Should use `ClassificationSource::Llm.to_string()`.

### Missing DB indices (`crates/core/src/db.rs`)

The `migrate()` function creates `idx_expenses_date` and `idx_expenses_dup` but no index on `category` or `classification_source`. Dashboard widgets group/filter by category; future queries will benefit from an index.

### Duplicated helper

`make_classification_rule` in `src-tauri/src/lib.rs` and `make_rule` in `crates/cli/src/main.rs` are identical logic. If Task 12 (editable rule pattern) changes the rule creation logic, both must be updated independently.

**Note:** Task 12 modifies `save_rule_if_categorized` to accept an optional custom pattern. After Task 12, the deduplication should preserve that new signature — move the core logic to `accountant-core` and have both callers use it.

## Scope

### 1. Add batch duplicate checking to Database

**File:** `crates/core/src/db.rs`

Add a new method that checks multiple expenses for duplicates in one query:

```rust
/// Check which (title, amount, date) tuples already exist in the database.
/// Returns a Vec<bool> aligned with the input slice — true means duplicate.
pub fn check_duplicates_batch(
    &self,
    expenses: &[(String, f64, chrono::NaiveDate)],
) -> Result<Vec<bool>, DbError>
```

Implementation approach: create a temporary table, bulk-insert the candidates, JOIN against `expenses`, collect results. Alternatively, for small batches (<500), build a single query with `OR`-chained conditions. The temp table approach scales better.

Update `parse_and_classify` in `src-tauri/src/lib.rs` and `cmd_bulk_insert` in `crates/cli/src/main.rs` to call `check_duplicates_batch` instead of looping with `is_duplicate`.

Keep `is_duplicate()` as-is for single-expense use (e.g., `add_expense`).

### 2. Replace panicking code in `run()`

**File:** `src-tauri/src/lib.rs`

Replace the two `.expect()` calls with `unwrap_or_else` that prints a user-readable error and exits:

```rust
let db = Database::open_default().unwrap_or_else(|e| {
    eprintln!("Fatal: failed to open database: {e}");
    std::process::exit(1);
});
```

For the `.run()` call, same pattern — exit with a message instead of panicking.

This is functionally similar but avoids a raw panic backtrace that's meaningless to users.

### 3. Replace direct array indexing with `.get()`

**File:** `src-tauri/src/lib.rs`

In `parse_and_classify`, replace `rows[i]` and `rows[*idx]` with `.get(i)` / `.get_mut(*idx)` and handle the `None` case by skipping or returning an error. Since the indices are derived from the same `rows` vec in the same function, `None` should be unreachable — a `continue` or early return with a logged warning is sufficient.

### 4. Use `ClassificationSource` enum instead of hardcoded string

**File:** `src-tauri/src/lib.rs`

Replace:
```rust
rows[*idx].source = Some("Llm".to_string());
```
With:
```rust
rows[*idx].source = Some(ClassificationSource::Llm.to_string());
```

### 5. Propagate directory creation errors

**File:** `crates/core/src/db.rs`

In `open_default()`, replace `.ok()` with proper error propagation:

```rust
if let Some(parent) = path.parent() {
    std::fs::create_dir_all(parent).map_err(|e| {
        DbError::InvalidData(format!("Cannot create data directory {}: {}", parent.display(), e))
    })?;
}
```

### 6. Add missing DB indices

**File:** `crates/core/src/db.rs`

Add to the `migrate()` function:

```sql
CREATE INDEX IF NOT EXISTS idx_expenses_category ON expenses(category);
CREATE INDEX IF NOT EXISTS idx_expenses_source ON expenses(classification_source);
```

These are `IF NOT EXISTS` so they're safe to add to an existing migration block.

### 7. Add `console.warn` to bare catch blocks

**File:** `src/lib/AddExpense.svelte`

Replace the two bare catches:

```js
// In onTitleInput:
} catch (err) { console.warn("Category suggestion failed:", err); suggestedCategory = ""; }

// After refreshing categories:
} catch (err) { console.warn("Failed to refresh categories:", err); }
```

### 8. Move `make_classification_rule` to `accountant-core`

**File:** `crates/core/src/models.rs` (or a new `crates/core/src/rules.rs` — prefer models.rs to avoid a new file)

Move the rule-creation logic to the core crate so both `src-tauri` and `crates/cli` can import it:

```rust
impl ClassificationRule {
    /// Create a case-insensitive regex rule from a pattern string.
    pub fn from_pattern(pattern_source: &str, category: &str) -> Self {
        let escaped = regex::escape(pattern_source);
        Self {
            id: None,
            pattern: format!("(?i){}", escaped),
            category: category.to_string(),
        }
    }
}
```

Then remove the duplicate functions from both `src-tauri/src/lib.rs` and `crates/cli/src/main.rs`, replacing calls with `ClassificationRule::from_pattern(...)`.

**Important:** If Task 12 has already been completed, `save_rule_if_categorized` will accept an optional `rule_pattern`. The core method should still just take a `pattern_source: &str` — the caller decides whether to pass the custom pattern or the title.

## Files to Change

| File | Change |
|---|---|
| `crates/core/src/db.rs` | Add `check_duplicates_batch()` method; propagate `create_dir_all` error in `open_default()`; add `idx_expenses_category` and `idx_expenses_source` indices to `migrate()` |
| `crates/core/src/models.rs` | Add `ClassificationRule::from_pattern()` constructor |
| `src-tauri/src/lib.rs` | Replace `.expect()` with `unwrap_or_else` + exit; replace `rows[i]` with `.get(i)`; use `ClassificationSource::Llm.to_string()`; replace `make_classification_rule()` with `ClassificationRule::from_pattern()`; use `check_duplicates_batch()` in `parse_and_classify` |
| `crates/cli/src/main.rs` | Replace `make_rule()` with `ClassificationRule::from_pattern()`; use `check_duplicates_batch()` in `cmd_bulk_insert` |
| `src/lib/AddExpense.svelte` | Add `console.warn` to both bare `catch` blocks |

## Test Scenarios

### Backend (Rust unit tests)

1. **`check_duplicates_batch` — all new** — pass 3 expenses, none in DB; returns `[false, false, false]`
2. **`check_duplicates_batch` — all duplicates** — insert 3 expenses, then check the same 3; returns `[true, true, true]`
3. **`check_duplicates_batch` — mixed** — insert 2 expenses, check 3 (2 matching + 1 new); returns `[true, false, true]` (or appropriate order)
4. **`check_duplicates_batch` — empty input** — pass empty slice; returns empty vec
5. **`ClassificationRule::from_pattern` — basic** — `from_pattern("Coffee Shop", "Cafe")` returns rule with pattern `(?i)Coffee\\ Shop` and category `Cafe`
6. **`ClassificationRule::from_pattern` — regex metacharacters** — `from_pattern("LIDL (Store)", "Groceries")` returns pattern `(?i)LIDL\\ \\(Store\\)`
7. **`open_default` error propagation** — hard to test directly, but verify `create_dir_all` error is wrapped in `DbError::InvalidData` (mock or test with invalid path)
8. **Indices exist after migration** — open an in-memory DB, query `sqlite_master` for `idx_expenses_category` and `idx_expenses_source`; both should exist

### Frontend (manual UI tests)

1. **AddExpense: type a very short title (1 char)** — no crash, no console error (suggestion skipped, `console.warn` not triggered)
2. **AddExpense: disconnect network, type a title** — `console.warn` appears in devtools instead of silent swallow
3. **Bulk upload: import 50+ expenses** — should be noticeably faster than before (1 batch query vs 50 individual queries)
4. **App startup with no data directory** — app should show a readable error message, not a panic backtrace

## Acceptance Criteria

- No `.expect()` calls in non-test code in `src-tauri/src/lib.rs`
- No direct array indexing (`rows[i]`) in `parse_and_classify` — all access via `.get()` or `.get_mut()`
- No bare `catch {}` blocks in any `.svelte` file — all catches log to `console.warn`
- `ClassificationSource::Llm.to_string()` used instead of `"Llm".to_string()` in `parse_and_classify`
- `check_duplicates_batch()` exists in `db.rs` with unit tests and is used by both `parse_and_classify` and `cmd_bulk_insert`
- `make_classification_rule()` / `make_rule()` removed from both `src-tauri/src/lib.rs` and `crates/cli/src/main.rs`; replaced by `ClassificationRule::from_pattern()` in `accountant-core`
- `idx_expenses_category` and `idx_expenses_source` indices created in `migrate()`
- `open_default()` propagates `create_dir_all` errors instead of silently ignoring them
- All existing tests pass (`cargo test`)
