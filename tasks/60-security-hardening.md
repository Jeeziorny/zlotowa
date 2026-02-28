# 60 — Security Hardening

## Problem

Several defense-in-depth gaps around secret storage and data leakage.

### Findings

1. **Plaintext API keys** — `crates/core/src/db.rs:929-934`: LLM API keys stored as plain TEXT in the SQLite `config` table via `set_config("llm_api_key", ...)`. The DB file at `~/Library/Application Support/4ccountant/` is readable by any process running as the user.

2. **LLM data leakage** — `crates/core/src/llm.rs:47-77` (`build_classification_prompt`): Expense titles and amounts are sent to external LLM APIs. Users are not warned that financial data leaves the device when LLM is enabled.

3. **`.expect()` panics in production code** — `crates/core/src/db.rs:1230,1246,1287,1289`: `restore_backup_data` uses `.expect("date already validated")` on date parsing. If backup data is corrupted between validation and use, these panic and crash the app. Replace with `map_err` returning `DbError`.

4. **Silent migration errors** — `crates/core/src/db.rs:127-129`: `ALTER TABLE expenses ADD COLUMN batch_id` error is swallowed with `let _ = ...`. If it fails for any reason other than "column exists" (disk full, corruption), the error is lost. Same at line 193-195 for the title_cleanup_rules unique index.

## Scope

- Replace `.expect()` calls in `restore_backup_data` with proper `map_err` error propagation
- For migration `let _ =` patterns: check if column/index already exists before ALTER, or at least log the error
- Add a user-facing notice in Settings when LLM is enabled: "Expense titles and amounts are sent to [provider] for classification"
- Evaluate using macOS Keychain (via `keyring` crate or similar) for API key storage instead of SQLite — or at minimum document the tradeoff
