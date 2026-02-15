# Task 12: Editable Rule Pattern for Classification Rules

**Track:** B — Classification Pipeline (frontend + backend)
**Blocked by:** nothing
**Blocks:** nothing

## Problem

Auto-generated classification rules are useless for real bank data. When a user categorizes an expense, `make_classification_rule` (`src-tauri/src/lib.rs:68`) escapes the **full raw title** into a regex. Polish bank titles contain variable data that makes every transaction unique:

**Card payments** embed the specific amount:
```
DOP. MC 557519******2036 PLATNOSC KARTA 94.88 PLN LIDL SWOJCZYCKA Wroclaw
DOP. MC 557519******2036 PLATNOSC KARTA 109.70 PLN LIDL SWOJCZYCKA Wroclaw  <- different amount, no match
```

**BLIK payments** embed a unique reference number:
```
Zakup BLIK allegro.pl WIERZBIECICE 1b ref:93363518508
Zakup BLIK allegro.pl WIERZBIECICE 1b ref:93348201689  <- different ref, no match
```

The escaped full-title regex will almost never match a future transaction from the same merchant. The "learning loop" (user categorizes -> rule saved -> future auto-classification) is broken. Every import falls through to the LLM every time, and `suggest_category` autocomplete never fires for previously-seen merchants.

## Solution

Add an editable "Match keyword" field wherever users assign categories. Instead of silently generating a rule from the full title, let the user specify what the rule should match. Pre-populate with the full title so existing behavior is preserved, but allow trimming to just `LIDL` or `allegro`.

## Current State

**Rule creation (`src-tauri/src/lib.rs:68-83`):**
- `make_classification_rule(title, category)` does `regex::escape(title)` + wraps in `(?i)...`
- `save_rule_if_categorized(db, title, category)` calls it whenever a category is non-empty
- Used in `add_expense` (line 108) and `bulk_save_expenses` (line 365)

**Bulk save IPC struct (`src-tauri/src/lib.rs:53-58`):**
```rust
pub struct BulkSaveExpense {
    pub title: String,
    pub amount: f64,
    pub date: String,
    pub category: Option<String>,
    pub source: Option<String>,
}
```
No field for a custom rule pattern — rule is always derived from `title`.

**Bulk Upload review UI (`src/lib/BulkUpload.svelte:179-229`):**
- `expenseTable` snippet renders each row with: Date, Title, Amount, editable Category input, optional Source badge
- `editCategory(index, newCategory)` (line 142) updates `classifiedRows[index].category`
- `saveApproved()` (line 147) maps rows to `{ title, amount, date, category, source }` and calls `bulk_save_expenses`

**AddExpense UI (`src/lib/AddExpense.svelte`):**
- Single-expense form with Title, Amount, Date, Category fields
- `submit()` (line 46) calls `add_expense` IPC with `{ title, amount, date, category }`
- Category suggestion via `suggest_category` IPC on title input (debounced, line 31)

**`add_expense` IPC (`src-tauri/src/lib.rs:94-110`):**
```rust
struct ExpenseInput {
    pub title: String,
    pub amount: f64,
    pub date: String,
    pub category: Option<String>,
}
```
After inserting, calls `save_rule_if_categorized(&db, &input.title, &input.category)` — always uses `title`.

## Scope

### 1. Add `rule_pattern` field to IPC structs

**`src-tauri/src/lib.rs`:**

Add `rule_pattern: Option<String>` to both `ExpenseInput` (line 27) and `BulkSaveExpense` (line 53):

```rust
pub struct ExpenseInput {
    pub title: String,
    pub amount: f64,
    pub date: String,
    pub category: Option<String>,
    pub rule_pattern: Option<String>,  // NEW
}

pub struct BulkSaveExpense {
    pub title: String,
    pub amount: f64,
    pub date: String,
    pub category: Option<String>,
    pub source: Option<String>,
    pub rule_pattern: Option<String>,  // NEW
}
```

### 2. Update rule creation to use custom pattern when provided

**`src-tauri/src/lib.rs`:**

Change `save_rule_if_categorized` to accept an optional custom pattern. When provided, use it instead of the full title:

```rust
fn save_rule_if_categorized(db: &Database, title: &str, category: &Option<String>, rule_pattern: &Option<String>) {
    if let Some(cat) = category {
        if !cat.is_empty() {
            let pattern_source = rule_pattern.as_deref()
                .filter(|p| !p.is_empty())
                .unwrap_or(title);
            let _ = db.insert_rule(&make_classification_rule(pattern_source, cat));
        }
    }
}
```

Update the two call sites:
- `add_expense` (line 108): pass `&input.rule_pattern`
- `bulk_save_expenses` (line 363-367): use `e.rule_pattern` instead of `&e.title` when building rules

### 3. Add "Match keyword" column to bulk review table

**`src/lib/BulkUpload.svelte`:**

In the `expenseTable` snippet (line 179), add a "Match keyword" column between Title and Category. Each row gets an editable input pre-populated with the full title:

- Add `rule_pattern` field to each row in `classifiedRows` (initialized to `row.title` in the `goToReview` mapping at line 134)
- New column with a text input, styled like the existing category input but slightly narrower
- Placeholder text: "e.g. LIDL" to hint at the expected usage
- `saveApproved()` (line 149) passes `rule_pattern` in the mapped object sent to `bulk_save_expenses`

The column should appear **only in the "Needs your input" and "Classified by AI" sections** where users are likely to edit. For "Classified by rules" section, the rule already exists and works, so no pattern field needed.

### 4. Add "Match keyword" field to AddExpense form

**`src/lib/AddExpense.svelte`:**

Add an optional "Match keyword" field below the Title field. It should:
- Be collapsed by default (small "Advanced" or "Set match keyword" toggle link)
- When expanded, show a text input pre-populated with the current title value
- Pass `rule_pattern` in the `submit()` call to `add_expense`
- This field is less important than in bulk upload (manual entries tend to have clean titles already), so keep it unobtrusive

### 5. Smart auto-fill for repeated merchants in bulk review

**`src/lib/BulkUpload.svelte`:**

When a user edits the `rule_pattern` for one row, check if other rows' titles contain the same keyword. If so, offer to apply the same pattern + category to matching rows:

- After a user sets `rule_pattern` to e.g. `LIDL` on one row and tabs/clicks away, scan other rows
- If other rows' titles contain `LIDL` (case-insensitive) and they have no category yet (or are unclassified), auto-fill their `rule_pattern` and `category` to match
- Show a brief toast or inline note: "Applied to N other matching rows"

This is the key UX payoff — categorize one LIDL transaction and all 8 get filled in.

Implementation: add an `onchange` handler on the rule_pattern input. When it fires and the value differs from the title, run the scan:

```js
function onRulePatternChange(index, newPattern) {
    classifiedRows[index].rule_pattern = newPattern;
    if (!newPattern.trim()) return;

    const category = classifiedRows[index].category;
    if (!category) return;

    let applied = 0;
    for (let i = 0; i < classifiedRows.length; i++) {
        if (i === index || classifiedRows[i].is_duplicate) continue;
        if (classifiedRows[i].category) continue;  // don't overwrite existing
        if (classifiedRows[i].title.toLowerCase().includes(newPattern.toLowerCase())) {
            classifiedRows[i].rule_pattern = newPattern;
            classifiedRows[i].category = category;
            classifiedRows[i].source = "Manual";
            applied++;
        }
    }
    // optionally show applied count
}
```

## Files to Change

| File | Change |
|---|---|
| `src-tauri/src/lib.rs` | Add `rule_pattern` to `ExpenseInput` and `BulkSaveExpense`, update `save_rule_if_categorized` to accept and use custom pattern, update `add_expense` and `bulk_save_expenses` call sites |
| `src/lib/BulkUpload.svelte` | Add `rule_pattern` field to `classifiedRows`, add "Match keyword" column to review table (AI + unclassified sections), pass `rule_pattern` in `saveApproved()`, add auto-fill logic for repeated merchants |
| `src/lib/AddExpense.svelte` | Add collapsible "Match keyword" field, pass `rule_pattern` in `submit()` |

## Test Scenarios

### Backend (Rust unit tests)

1. **`rule_pattern` used when provided** — call `make_classification_rule("LIDL", "Groceries")` with a title of "DOP. MC ... LIDL Wroclaw" but rule_pattern "LIDL"; saved rule pattern should be `(?i)LIDL`, not the escaped full title
2. **Falls back to title when `rule_pattern` is None or empty** — existing behavior preserved

### Frontend (manual UI tests)

1. **Bulk upload: Match keyword column visible** — upload a CSV, classify; in review step, "Needs your input" and "Classified by AI" sections show a "Match keyword" column
2. **Bulk upload: pre-populated with title** — each row's match keyword field shows the full title by default
3. **Bulk upload: editable** — user can clear the field and type "LIDL"; the value persists
4. **Bulk upload: auto-fill** — user sets match keyword to "LIDL" and category to "Groceries" on one row; other rows with "LIDL" in the title that have no category get auto-filled
5. **Bulk upload: auto-fill doesn't overwrite** — rows that already have a category are not touched by auto-fill
6. **Bulk upload: saved to backend** — after saving, check `classification_rules` table; rules should use the custom pattern, not the full title
7. **Bulk upload: DB-classified rows don't show pattern field** — "Classified by rules" section has no Match keyword column (those rules already work)
8. **AddExpense: match keyword hidden by default** — the field is collapsed/hidden on initial load
9. **AddExpense: match keyword expandable** — clicking toggle reveals the field, pre-populated with current title
10. **AddExpense: rule uses custom pattern** — add an expense with title "DOP. MC ... LIDL" and match keyword "LIDL"; check DB rule is `(?i)LIDL`
11. **Backward compatibility** — if match keyword is left at the default (full title), behavior is identical to before

## Acceptance Criteria

- Users can edit what text the auto-generated classification rule matches, instead of it always being the full title
- Match keyword field is pre-populated with the full title (backward-compatible default)
- In bulk upload, editing one row's pattern + category auto-fills other rows whose titles contain the same keyword
- "Classified by rules" rows don't show the pattern field (their rules already work)
- AddExpense form has an unobtrusive optional field for the pattern
- No new DB tables or migrations needed — same `classification_rules` schema, just smarter pattern content
- Follows dark theme conventions (gray-950/900/800, emerald accents)
