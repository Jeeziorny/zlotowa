Update the user-facing documentation in `docs/src/` to match the current state of the application. Do NOT invent features — only document what actually exists in the code.

## Process

1. Read all Svelte components in `src/lib/` and `src/lib/widgets/` to understand current UI features
2. Read `src-tauri/src/lib.rs` to understand available IPC commands
3. Read `crates/core/src/models.rs` for data structures
4. Read `crates/core/src/parsers/` for supported import formats
5. Read `crates/core/src/classifiers.rs` for classification behavior
6. Read all existing doc pages in `docs/src/`
7. Compare what the code does vs what the docs say

## What to update

For each doc page, check:
- **Accuracy** — does the documented behavior match the code? Fix any drift.
- **Completeness** — are there features in the code not mentioned in docs? Add them.
- **Removed features** — are there documented features no longer in the code? Remove them.
- **Widget list** — does `docs/src/features/dashboard.md` match `src/lib/widgets/registry.js`?
- **Supported formats** — does `docs/src/features/bulk-import.md` match the actual parsers?
- **LLM providers** — does `docs/src/features/llm-config.md` match the Settings UI?
- **SUMMARY.md** — does the table of contents reflect all existing pages? Add entries for new pages, remove entries for deleted ones.

## Rules

- Write for end users, not developers. No code snippets unless showing CLI usage.
- Keep language simple and direct. Short sentences. No marketing fluff.
- Use tables for structured data (formats, providers, widgets).
- Each page should be self-contained — don't require reading other pages to understand.
- Match the existing tone and structure of current docs.

## After updating

Run `mdbook build docs` via Bash to verify the book compiles without errors.

Report a summary of what changed: pages added, pages updated, pages removed.
