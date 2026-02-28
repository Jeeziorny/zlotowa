# 63 — Docs Sync Round 3

## Problem

CLAUDE.md describes features and code that no longer exist.

### Findings

1. **Phantom Exporter** — `CLAUDE.md` documents an `Exporter` trait in `crates/core/src/exporters.rs` with `name()`, `extension()`, `export()` methods and a `CsvExporter`. Also lists `export_expenses` as an IPC command. Neither the file nor the command exist — the exporter was removed but docs weren't updated.

2. **CLI crate status** — `CLAUDE.md` documents `accountant-cli` with 5 commands (`llm-conf`, `insert`, `bulk-insert`, `export`, `dashboard`). Verify `crates/cli` still exists and is functional, or remove from docs.

3. **Inconsistent widget data pattern** — `src/lib/widgets/BudgetStatus.svelte:25-32` fetches its own data via `invoke()` in `onMount` while all other widgets receive data via the `expenses` prop. This isn't a bug but should be documented or made consistent.

4. **Command count** — CLAUDE.md says "39 commands grouped by domain" — verify this matches the actual count after exporter removal.

## Scope

- Remove Exporter references from CLAUDE.md (trait description, extending section, IPC command list)
- Verify CLI crate status and update docs accordingly
- Update IPC command count
- Note BudgetStatus widget's independent data fetching in the widget section or make it consistent
