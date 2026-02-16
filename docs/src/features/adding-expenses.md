# Adding Expenses

## Manual entry

Go to **Add Expense** in the sidebar. Fill in:

- **Date** — defaults to today. Click the field to open a calendar picker for quick date selection, or type a date in `YYYY-MM-DD` format directly
- **Title** — the expense description (e.g., "Grocery Store")
- **Match keyword** (optional) — click "Set match keyword..." to customize what the auto-classification rule matches. Defaults to the full title, but you can trim it to just the key part (e.g., "LIDL" instead of "DOP. MC 557519 LIDL Wroclaw"). See [Classification](./classification.md) for details
- **Amount** — the expense amount
- **Category** (optional) — start typing to see suggestions from your existing categories. If you provide a category, the app creates a classification rule so future expenses with the match keyword get categorized automatically

The app also suggests a category based on the title you enter. If an existing rule matches the title, the suggested category appears automatically.

## What happens when you set a category

When you save an expense with a category, the app creates a pattern-matching rule:

> Title: "LIDL Grocery" + Category: "Groceries" → Rule: match any expense containing "LIDL Grocery" → assign "Groceries"

If you set a match keyword (e.g., "LIDL"), the rule matches that keyword instead of the full title. This is useful for bank transactions that include variable data like amounts or reference numbers in the title.

This rule is case-insensitive and applies automatically during future bulk imports.
