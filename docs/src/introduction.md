# 4ccountant

4ccountant is a desktop application for tracking and categorizing personal expenses. It runs locally on your machine — your financial data never leaves your computer.

## What it does

- **Track expenses** — add them manually or import in bulk from CSV bank exports
- **Auto-classify** — the app learns categories from your past decisions and applies them to new expenses automatically using pattern matching
- **LLM support** (optional) — connect an AI provider (OpenAI, Anthropic, or Ollama) to classify expenses that don't match any existing rules
- **Export** — export your expenses to CSV with customizable column selection
- **Category management** — rename, merge, and delete categories across all expenses and rules
- **Title cleanup** — find/replace rules to normalize noisy transaction titles
- **Budget planning** — set monthly spending limits per category, track planned expenses, import calendar events
- **Dashboard** — configurable widgets showing spending breakdowns, trends, and budget status
- **CLI** — terminal interface for all core features (`4ccountant` binary)

## How classification works

When you categorize an expense (e.g., mark "LIDL" as "Groceries"), the app creates a rule that automatically applies "Groceries" to any future expense matching "LIDL". Over time, most of your imports get classified automatically.

The classification pipeline checks rules in order:
1. **Database rules** — regex patterns learned from your previous categorizations
2. **LLM** (if configured) — asks an AI model to suggest a category
3. **Unclassified** — you assign a category manually, which creates a new rule for next time

## Roadmap

Features under consideration for future releases:

- **Receipt parsing** — import expenses from photos of receipts
- **Additional export formats** — JSON, PDF reports
