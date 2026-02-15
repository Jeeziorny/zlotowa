# Categories & Auto-Classification

## How rules work

Every time you assign a category to an expense — whether manually or during bulk import — the app saves a **classification rule**. Rules are regex patterns matched against expense titles (case-insensitive).

For example, categorizing an expense titled "SHELL STATION #42" as "Fuel" creates a rule that matches any future expense containing "SHELL STATION #42".

## Classification pipeline

When expenses are imported, each one is checked against rules in this order:

1. **Database rules** — if the title matches a stored regex pattern, the category is applied automatically (shown as "DB Rule")
2. **LLM** (if configured) — unmatched expenses are sent to an AI provider for classification (shown as "LLM")
3. **Unclassified** — if nothing matches, you assign a category manually (shown as "Manual" or "Unclassified")

The first match wins — if a database rule matches, the LLM is not consulted.

## Building up your rules

The more you use the app, the more rules accumulate. After a few bulk imports, most common expenses (recurring subscriptions, regular stores) will be classified automatically.

Rules are stored in the local database and persist across sessions.
