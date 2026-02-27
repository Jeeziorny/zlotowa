# Task 57 — Structured Logging

## Goal

Add structured logging to the Rust backend so log files are available for remote debugging when sharing the app with friends. Currently the app has zero structured logging — just 2 `eprintln!` calls at startup and ~14 frontend `console.*` calls.

## Dependencies

### Rust crates

- `log = "0.4"` in `crates/core/Cargo.toml` (dependencies)
- `log = "0.4"` in `src-tauri/Cargo.toml` (dependencies)
- `tauri-plugin-log = "2"` in `src-tauri/Cargo.toml` (dependencies)

### Tauri capability

Add `"log:default"` to the `permissions` array in `src-tauri/capabilities/default.json`.

## Plugin Configuration

In `src-tauri/src/lib.rs`, register the log plugin in `run()`:

```rust
use tauri_plugin_log::{Target, TargetKind, TimezoneStrategy, RotationStrategy};

.plugin(
    tauri_plugin_log::Builder::new()
        .targets([
            Target::new(TargetKind::Stdout),
            Target::new(TargetKind::LogDir { file_name: None }),
        ])
        .timezone_strategy(TimezoneStrategy::UseLocal)
        .max_file_size(5_000_000) // 5 MB
        .rotation_strategy(RotationStrategy::KeepAll)
        .level(log::LevelFilter::Debug)
        .build(),
)
```

Log files land in `~/Library/Logs/4ccountant/` on macOS.

## Where to Add Logs

### 1. `src-tauri/src/lib.rs`

- **Plugin registration:** `info!` in `run()` confirming log plugin loaded
- **All 39 IPC commands:**
  - `info!` for mutations (add, update, delete, save, create, merge, rename, apply, clear, parse_and_classify, bulk_save)
  - `debug!` for reads (get, list, query, preview, suggest, export, check)
- **All `.map_err()` paths:** `warn!` or `error!` with the error message
- **`parse_and_classify`:** detailed phase logging — preview, parse, classify, LLM fallback, result summary

### 2. `crates/core/src/db.rs`

- `info!` — DB open path, migration complete
- `info!` — bulk insert count + filename
- `error!` — migration failures, transaction errors

### 3. `crates/core/src/llm.rs`

- `info!` — HTTP call entry (provider name + expense count, **NOT** api key)
- `info!` — response status code
- `debug!` — parsed classification results
- `warn!` — validation failures, HTTP errors

### 4. `crates/core/src/classifiers.rs`

- `info!` — pipeline entry with expense count
- `info!` — pipeline exit with classified/unclassified counts

### 5. `crates/core/src/parsers/mod.rs`

- `info!` — parser detection result + confidence score

### 6. `crates/core/src/parsers/csv_parser.rs`

- `debug!` — delimiter detection scores
- `info!` — final row count after parse
- `warn!` — row-level parse errors

### 7. `crates/core/src/ical.rs`

- `info!` — total event count parsed
- `debug!` — skipped events (bad format, missing fields)
- `info!` — date range filter result count

### 8. `crates/core/src/exporters.rs`

- `info!` — export row count and selected columns

## Log Level Guide

| Level | Use for | Example |
|-------|---------|---------|
| `error!` | Should not happen | Mutex poison, migration failure |
| `warn!` | Recoverable problems | LLM fallback failed, parse row error, auth rejected |
| `info!` | User-visible operations | "parsed 42 expenses", "bulk saved 35" |
| `debug!` | Developer details | Per-row parse, delimiter scores, regex compile counts |

## Security

- **NEVER** log API keys, `Authorization` headers, or `x-api-key` values
- Don't `info!` inside hot loops (per-row in large CSV) — use `debug!` sparingly

## Testing

- `cargo test` — all existing tests must pass (log macros are no-ops without a subscriber)
- Manual: `npm run tauri dev` shows timestamped logs on stdout
- Manual: log file appears at `~/Library/Logs/4ccountant/`
- Manual: bulk upload shows parser → classify → save pipeline in logs

## Frontend (Optional, Low Priority)

- Add `@tauri-apps/plugin-log` npm package
- Wire frontend logs to the same log file via the plugin
- Existing `console.*` calls are sufficient for now — this is stretch goal only

## Files to Modify

1. `crates/core/Cargo.toml` — add `log` dependency
2. `src-tauri/Cargo.toml` — add `log` + `tauri-plugin-log` dependencies
3. `src-tauri/capabilities/default.json` — add `"log:default"` permission
4. `src-tauri/src/lib.rs` — plugin setup + IPC command logging
5. `crates/core/src/db.rs` — DB operation logging
6. `crates/core/src/llm.rs` — LLM provider logging
7. `crates/core/src/classifiers.rs` — pipeline logging
8. `crates/core/src/parsers/mod.rs` — parser detection logging
9. `crates/core/src/parsers/csv_parser.rs` — CSV parse logging
10. `crates/core/src/ical.rs` — calendar parse logging
11. `crates/core/src/exporters.rs` — export logging
