# Categories & Auto-Classification

## How rules work

Every time you assign a category to an expense — whether manually or during bulk import — the app saves a **classification rule**. Rules are regex patterns matched against expense titles (case-insensitive).

For example, categorizing an expense titled "SHELL STATION #42" as "Fuel" creates a rule that matches any future expense containing "SHELL STATION #42".

### Match keywords

Bank transaction titles often contain variable data (amounts, reference numbers, dates) that make the full title unique to each transaction. For example:

- `DOP. MC 557519 PLATNOSC KARTA 94.88 PLN LIDL Wroclaw`
- `DOP. MC 557519 PLATNOSC KARTA 109.70 PLN LIDL Wroclaw`

A rule matching the full first title would never match the second. To handle this, you can set a **match keyword** when categorizing an expense. Instead of the full title, the rule matches just the keyword (e.g., "LIDL"). This is available in both the Add Expense form and the Bulk Upload review step.

If you don't set a match keyword, the full title is used (the default behavior).

## Classification pipeline

When expenses are imported, each one is checked against rules in this order:

1. **Database rules** — if the title matches a stored regex pattern, the category is applied automatically (shown as "DB")
2. **LLM** (if configured) — unmatched expenses are sent to an AI provider in a batch for classification (shown as "LLM"). Each result includes a confidence level: **High**, **Medium**, or **Low**
3. **Unclassified** — if nothing matches, you assign a category manually (shown as "Manual")

The first match wins — if a database rule matches, the LLM is not consulted.

## LLM confidence

When the LLM classifies an expense, it reports a confidence level:

| Level | Meaning |
|---|---|
| **High** | The expense clearly belongs to the suggested category |
| **Medium** | Reasonable guess, but the title is somewhat ambiguous |
| **Low** | The title is vague or cryptic (e.g., reference numbers, generic descriptions) |

Confidence badges are shown during bulk import review so you can prioritize which AI classifications to double-check.

## Building up your rules

The more you use the app, the more rules accumulate. After a few bulk imports, most common expenses (recurring subscriptions, regular stores) will be classified automatically.

Rules are stored in the local database and persist across sessions.
