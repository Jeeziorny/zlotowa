# Adding Expenses

## Manual entry

Go to **Add Expense** in the sidebar. Fill in:

- **Date** — defaults to today
- **Title** — the expense description (e.g., "Grocery Store")
- **Amount** — the expense amount
- **Category** (optional) — start typing to see suggestions from your existing categories. If you provide a category, the app creates a classification rule so future expenses with the same title get categorized automatically

The app also suggests a category based on the title you enter. If an existing rule matches the title, the suggested category appears automatically.

## What happens when you set a category

When you save an expense with a category, the app creates a pattern-matching rule:

> Title: "LIDL Grocery" + Category: "Groceries" → Rule: match any expense containing "LIDL Grocery" → assign "Groceries"

This rule is case-insensitive and applies automatically during future bulk imports.
