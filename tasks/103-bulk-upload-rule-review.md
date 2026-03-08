# Task 103: Bulk Upload Rule Review Step

## Problem

When expenses are categorized (manually, by LLM, or via bulk upload), a classification rule is auto-generated using the **full escaped title** as a regex pattern — e.g. `(?i)5000\.00 PLN XTB S\.A\. Warsaw`. This makes rules overly specific because amounts, card fragments, reference numbers, and other noise are baked into the pattern. The rule only matches that exact title variation instead of all future transactions from the same merchant.

There is no opportunity for the user to decide what part of the title actually matters for future matching.

## Solution

Add a **Rule Review step** (step 6) to the bulk upload wizard, shown after expenses are saved and before the Done screen. This step lets users edit, trim, or remove auto-generated rule patterns before they are committed to the database.

Additionally, add the same inline pattern editing to the **Add Expense** and **Expense List inline edit** flows.

## Design

### Bulk Upload: New Step 6 — "Review Rules"

**Flow change:**
```
Step 1: Input → Step 2: Columns → Step 3: Cleanup → Step 4: Review → Step 5: Save
                                                                          ↓
                                                                    Step 6: Review Rules
                                                                          ↓
                                                                    Step 7: Done
```

**What to show:** Only rules for expenses where classification source is NOT `"Database"` (i.e., expenses that were classified by LLM, manually assigned, or unclassified-then-categorized by the user). Expenses already matched by existing DB rules don't need new rules.

**Display:** Rules grouped by category. Each row shows the expense title with an editable text input pre-filled with the full title. The user can trim it down to just the meaningful part (e.g., `5000.00 PLN XTB S.A. Warsaw` → `XTB S.A.`).

**Each row has:**
- The original expense title (read-only, for reference)
- eddit button that unlocks an editable text input for the pattern text (pre-filled with the full title),
- An `✕` button to remove that rule (don't create it at all)

**Show all unique titles** — no automatic deduplication/grouping (too error-prone). Many exact-duplicate titles will naturally collapse since they produce the same pattern.

**Bottom actions:**
- "Save Rules" — creates rules from the edited patterns
- "Skip All" — no rules created, proceed to Done

**Pattern generation from edited text:** The user edits plain text (no regex knowledge needed). The app generates the pattern by escaping the text with `regex::escape()` and wrapping in `(?i)`. Since `Regex::is_match()` does substring matching, `(?i)XTB S\.A\.` will match any title containing "XTB S.A." — effectively a case-insensitive "contains" rule. 

### Navigation guard

If the user navigates away (clicks sidebar, other tab) while on step 6, show a warning dialog:
- "What do you want to do with the classification rules?"
- **"Skip"** — no rules created, navigate away
- **"Save as-is"** — create rules using `(?i)<escaped_full_title>` (exact match, same as today's behavior), then navigate away

This ensures no data loss if the user doesn't want to bother with rule editing.

### Backend changes

**Decouple rule creation from expense saving.** Currently `bulk_save_expenses` in `src-tauri/src/lib.rs` creates rules as part of the same transaction that saves expenses. This needs to be split:

1. `bulk_save_expenses` — saves expenses only, does NOT create rules. Returns the list of expenses that need rules (those with `source != "Database"`).
2. New IPC command `bulk_save_rules` — accepts a list of `{ pattern_text: String, category: String }` pairs, generates `ClassificationRule::from_pattern()` for each, and inserts them.

This separation allows step 5 (save expenses) and step 6 (save rules) to be independent operations.

### Add Expense & Inline Edit

When a user assigns a category in the **Add Expense** form or **Expense List inline edit**, show an additional inline field below the category input:

```
Match future transactions containing: [editable text field pre-filled with title]
```

The user can trim it before saving. If left unchanged, the full title is used (same as today). Behind the scenes, the edited text is escaped and wrapped in `(?i)`.

This is a smaller change — just an extra input field in `AddExpense.svelte` and `ExpenseTable.svelte`, and passing the custom pattern text to the existing `add_expense` / `update_expense` IPC commands.

**IPC change:** `add_expense` and `update_expense` accept an optional `rule_pattern` field. If provided, use it instead of the title for `ClassificationRule::from_pattern()`. If absent, fall back to using the full title (backwards compatible).

## Files to modify

### Backend (Rust)
- `src-tauri/src/lib.rs` — split rule creation out of `bulk_save_expenses`, add `bulk_save_rules` command, add optional `rule_pattern` to `add_expense`/`update_expense`
- `crates/core/src/db.rs` — possible new method for bulk rule insert (or reuse `insert_rules_bulk`)
- `crates/core/src/models.rs` — no changes needed (`from_pattern` already works, just called with different input)

### Frontend (Svelte)
- `src/lib/BulkUpload.svelte` — add step 6 to the wizard flow, handle navigation guard for rule step
- New: `src/lib/bulk-upload/ReviewRules.svelte` — the rule review UI component
- `src/lib/AddExpense.svelte` — add inline pattern editor field
- `src/lib/expense-list/ExpenseTable.svelte` — add inline pattern editor field in edit mode
- data presented in sixth step of bulk upload should be visually consisted with other tables in the application. Reuse components we already have.

## Testing

### Rust tests
- `bulk_save_expenses` without rules (verify expenses saved, no rules created)
- `bulk_save_rules` with edited patterns
- `add_expense` with custom `rule_pattern`
- `update_expense` with custom `rule_pattern`

### Manual testing
- Bulk upload: verify step 6 appears with correct expenses (not DB-matched ones)
- Edit a pattern, save rules, verify the trimmed pattern works on next upload
- Remove a rule with ✕, verify it's not created
- "Skip All" — verify no rules created
- Navigate away during step 6 — verify warning appears with correct options
- Add Expense with custom pattern — verify rule uses edited text
- Inline edit with custom pattern — verify rule uses edited text
- 10th upload scenario — verify step 6 shows only new/unmatched expenses

### Rules tab
Rules tab should contain the the information >?<, so that people can hover over it and see that those rules are using CONTAIN criteria, not exact matches.