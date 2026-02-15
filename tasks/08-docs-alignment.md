# Task 8: Align Documentation with Implementation Reality

**Track:** E — Documentation (standalone)
**Blocked by:** nothing
**Blocks:** nothing

## Problem

The docs (`docs/src/`) present the app as feature-complete, but several described features
don't actually work yet. Most critically, LLM classification is documented as functional
when it's only a trait stub. Meanwhile, implemented features like the step wizard and
export are underdocumented or missing entirely.

## Current State

**Documentation structure** (`docs/src/SUMMARY.md`):
```
[Introduction](./introduction.md)
# Getting Started
- [Installation](./getting-started/installation.md)
- [First Launch](./getting-started/first-launch.md)
# Features
- [Adding Expenses](./features/adding-expenses.md)
- [Bulk Import](./features/bulk-import.md)
- [Categories & Auto-Classification](./features/classification.md)
- [Dashboard & Widgets](./features/dashboard.md)
- [LLM Configuration](./features/llm-config.md)
```

## Issues to Fix

### 1. `classification.md` — LLM described as working

**Current (lines 9-11):**
> 2. **LLM** (if configured) — unmatched expenses are sent to an AI provider for classification (shown as "LLM")

**Problem:** LLM provider code is a trait stub. No HTTP calls. No expenses are sent anywhere.

**Fix:** Mark as planned:
> 2. **LLM** (if configured) — *coming soon.* Once configured, unmatched expenses will be sent to an AI provider for classification.

Or restructure to separate "current" from "planned" behavior.

### 2. `llm-config.md` — implies end-to-end LLM flow

**Current:** The entire page describes setup as if it integrates with classification.
The "Supported providers" table lists key formats and notes.

**Problem:** Settings UI saves config, but nothing reads it for classification.

**Fix:** Add a note at the top:
> **Note:** LLM configuration is saved locally but classification integration is not yet active. Currently, all classification is done via learned rules or manual input.

### 3. `introduction.md` — feature list overclaims

**Current (line 8):**
> - **LLM support** (optional) — connect an AI provider to classify expenses that don't match any existing rules

**Fix:** Add "(coming soon)" or "(planned)" suffix.

### 4. Missing: Export feature documentation

No page exists for export. After Task 4+5, there should be one.

**Action:** Create `docs/src/features/export.md` with a "Coming Soon" stub:
```markdown
# Export

> This feature is under development.

Export your expense database to CSV. You'll be able to:
- Choose which columns to include (date, title, amount, category, classification source)
- Download directly from the Expenses page
```

Add to `SUMMARY.md`.

### 5. Missing: CLI documentation

No mention of CLI anywhere in docs. The CLI is a first-class feature in the initial prompt.

**Action:** Create `docs/src/cli.md` with a "Coming Soon" stub listing the planned commands:
```markdown
# CLI

> The CLI is under development.

4ccountant can also be used from the terminal. Planned commands:

| Command | Description |
|---|---|
| `4ccountant llm-conf` | Configure LLM API key |
| `4ccountant bulk-insert <file>` | Import expenses from CSV |
| `4ccountant insert` | Add a single expense |
| `4ccountant export` | Export expenses to CSV |
| `4ccountant dashboard` | Open the GUI |
```

Add to `SUMMARY.md`.

### 6. Missing: Roadmap section in introduction

The docs have no visibility into what's planned vs done. Add a section to `introduction.md`:

```markdown
## Roadmap

Features currently under development:

- **LLM classification** — AI-powered expense classification using OpenAI, Anthropic, or Ollama
- **CSV export** — export your expenses with customizable columns
- **CLI** — terminal interface for all core features
- **Category suggestions** — auto-suggest categories when adding expenses manually
- **Receipt parsing** — import expenses from photos of receipts (future)
- **Budget planning** — set and track spending budgets (future)
```

### 7. Undersold: BulkUpload step wizard

`bulk-import.md` describes the steps but doesn't mention the visual step indicator
(progress bar) at the top of the page. This is a nice UX detail worth highlighting.

**Fix:** Add a note:
> A step indicator at the top of the page shows your progress through the import process.

### 8. `first-launch.md` — navigation list missing Settings

**Current (lines 11-15):**
Lists Dashboard, Add Expense, Bulk Upload, Expenses, Settings — this is actually correct.
No fix needed.

## Files to Change

| File | Change |
|---|---|
| `docs/src/features/classification.md` | Mark LLM step as planned |
| `docs/src/features/llm-config.md` | Add "not yet active" note |
| `docs/src/introduction.md` | Mark LLM as planned, add Roadmap section |
| `docs/src/features/bulk-import.md` | Mention step indicator |
| `docs/src/features/export.md` | **New file** — coming soon stub |
| `docs/src/cli.md` | **New file** — coming soon stub |
| `docs/src/SUMMARY.md` | Add Export and CLI entries |

## Test Scenarios

1. **`mdbook build docs`** — builds without errors after all changes
2. **`mdbook serve docs`** — all pages render, no broken links
3. **SUMMARY.md links** — every entry in SUMMARY.md points to an existing file
4. **No overclaiming** — read through each page and verify no feature is described as working when it isn't
5. **Tone consistency** — "coming soon" notes use consistent phrasing across pages
6. **Roadmap completeness** — all gaps from the audit review are listed in the roadmap

## Acceptance Criteria

- `mdbook build docs` succeeds
- LLM is clearly marked as planned/coming soon wherever mentioned
- Export and CLI have stub pages
- Roadmap section provides visibility into what's next
- No false claims about functionality
- Existing accurate documentation is preserved
