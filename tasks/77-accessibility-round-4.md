# Task 77 — Accessibility Round 4: Color, Labels, Announcements

Merges items: #11 (colorblind-safe indicators), #12 (aria-labels on icon buttons), #14 (sort announcements).

## Problem

Three related accessibility gaps:
1. Budget progress bars and confidence labels use color alone (red/amber/green) — no text/icon fallback for colorblind users.
2. Several icon-only buttons (edit, delete, move, close) lack `aria-label` — screen readers announce them as empty.
3. Table sort toggles update `aria-sort` but don't announce the new state to screen readers.

## Solution

### 1. Colorblind-safe status indicators

**Budget progress (BudgetOverview.svelte):**
Add a text label to the right of each category progress bar:
- Over budget: `"Over"` in `text-red-400 text-xs font-medium`
- ≥80% spent: `"80%+"` in `text-amber-400 text-xs font-medium`
- Under budget: `"OK"` in `text-emerald-400 text-xs font-medium`

These labels sit inline after the bar, providing a shape/text signal alongside color.

**Confidence labels (ReviewClassified.svelte):**
Prefix each confidence badge with a character icon:
- High (≥0.8): `"✓ High"` (checkmark)
- Medium (≥0.5): `"~ Medium"` (tilde)
- Low (<0.5): `"! Low"` (exclamation)

The icon provides a non-color differentiator.

**Budget summary (BudgetOverview.svelte):**
Ensure the remaining/over amount always shows a sign prefix (`+$50.00` / `−$30.00`) alongside the color.

### 2. Aria labels on icon buttons — full sweep

Audit and add `aria-label` to every icon-only button across all components:

| Component | Button | aria-label |
|-----------|--------|------------|
| ExpenseTable | edit pencil | `"Edit expense"` |
| ExpenseTable | delete trash | `"Delete expense"` |
| ExpenseTable | save check | `"Save changes"` |
| ExpenseTable | cancel X | `"Cancel editing"` |
| ExpenseTable | select checkbox | `"Select expense"` |
| ExpenseTable | select-all checkbox | `"Select all expenses"` |
| RulesTable | edit pencil | `"Edit rule"` |
| RulesTable | delete trash | `"Delete rule"` |
| Dashboard | move left | `"Move widget left"` |
| Dashboard | move right | `"Move widget right"` |
| Dashboard | remove widget | `"Remove widget"` |
| Dashboard | edit config | `"Configure widget"` |
| Dashboard | close picker | `"Close"` |
| Categories | delete button | `"Delete category"` |
| SearchFilterBar | clear filters | `"Clear all filters"` |
| DatePicker | prev month | `"Previous month"` |
| DatePicker | next month | `"Next month"` |
| BudgetOverview | delete budget | `"Delete budget"` |
| BudgetPlanning | prev budget | `"Previous budget"` |
| BudgetPlanning | next budget | `"Next budget"` |
| ReviewClassified | remove category | `"Remove category"` |

Note: Task 62 (Accessibility Round 3) already added some aria-labels. Only add missing ones — check each component first to avoid duplicates.

### 3. Screen reader sort announcements

Add a visually-hidden live region near each sortable table:

```html
<div class="sr-only" role="status" aria-live="polite" aria-atomic="true">
  {sortAnnouncement}
</div>
```

When sort state changes, set `sortAnnouncement` to: `"Sorted by {column}, {ascending/descending}"`.

Add a `sr-only` utility class in `app.css`:
```css
.sr-only {
  position: absolute;
  width: 1px;
  height: 1px;
  padding: 0;
  margin: -1px;
  overflow: hidden;
  clip: rect(0, 0, 0, 0);
  white-space: nowrap;
  border-width: 0;
}
```

Apply to: `ExpenseTable`, `RulesTable`, `Categories` table headers.

Also add `aria-label="Sort by {column}"` to each sortable header button.

## Files

| File | Action |
|------|--------|
| `src/lib/budget/BudgetOverview.svelte` | Modify — status labels, sign prefix |
| `src/lib/bulk-upload/ReviewClassified.svelte` | Modify — confidence icons |
| `src/lib/expense-list/ExpenseTable.svelte` | Modify — aria-labels, sort announcement |
| `src/lib/rules/RulesTable.svelte` | Modify — aria-labels, sort announcement |
| `src/lib/Categories.svelte` | Modify — aria-labels, sort announcement |
| `src/lib/Dashboard.svelte` | Modify — aria-labels on widget controls |
| `src/lib/expense-list/SearchFilterBar.svelte` | Modify — aria-label on clear |
| `src/lib/DatePicker.svelte` | Modify — aria-labels on nav buttons |
| `src/lib/BudgetPlanning.svelte` | Modify — aria-labels on nav |
| `src/app.css` | Modify — add .sr-only utility class |

## Verification
1. Budget bars show text labels (OK/80%+/Over) alongside color
2. Confidence labels show icon prefix (✓/~/!)
3. All icon-only buttons have meaningful aria-label attributes
4. Sort a table column → screen reader announces "Sorted by Name, ascending"
5. Test with VoiceOver: navigate tables, modals, icon buttons
