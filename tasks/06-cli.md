# Task 6: Implement CLI Commands

**Track:** C — CLI (standalone)
**Blocked by:** nothing (uses existing core crate APIs; LLM fallback is best-effort)
**Blocks:** nothing

## Problem

The initial prompt (`initial_prompt.md` lines 81-86) defines 5 CLI commands. All are
stubbed in `crates/cli/src/main.rs` — every command prints "not yet implemented".
The CLI binary is named `4ccountant` and uses `clap` for argument parsing.

## Current State

**`crates/cli/src/main.rs`:**
```rust
enum Commands {
    LlmConf,
    BulkInsert { path: PathBuf },
    Insert,
    Export { grammar: Option<PathBuf> },
    Dashboard,
}
```
All branches: `println!("... not yet implemented")`.

**`crates/cli/Cargo.toml`:**
```toml
[dependencies]
accountant-core = { path = "../core" }
clap = { version = "4", features = ["derive"] }
tokio = { version = "1", features = ["full"] }
```

**Core crate APIs available:**
- `Database::open_default()` — opens the shared SQLite DB
- `Database::insert_expense()`, `insert_expenses_bulk()`
- `Database::get_all_expenses()`, `get_all_rules()`, `get_all_categories()`
- `Database::set_config()`, `get_config()`
- `parsers::builtin_parsers()`, `detect_parser()`, `Parser::parse()`
- `classifiers::RegexClassifier`, `classify_pipeline()`
- `exporters::CsvExporter` (after Task 4), `Exporter::export()`
- `llm::create_provider()`, `LlmProvider::validate()` (after Task 1)

## Scope

### 1. Add interactive dependencies

Add to `crates/cli/Cargo.toml`:
```toml
dialoguer = "0.11"      # Interactive prompts (select, input, confirm)
comfy-table = "7"        # Terminal table display
colored = "2"            # Colored output
```

### 2. `llm-conf` command

Interactive LLM API key configuration.

Flow:
1. Show current config (if any) from DB
2. Prompt: select provider (OpenAI / Anthropic / Ollama) using `dialoguer::Select`
3. Prompt: input API key (or endpoint URL for Ollama) using `dialoguer::Password` (masked)
4. Validate: call `provider.validate()` — show spinner/message
5. On success: save to DB via `set_config()`, print confirmation
6. On failure: show error, ask if they want to retry

```rust
Commands::LlmConf => {
    let db = Database::open_default().map_err(|e| eprintln!("{}", e)).unwrap();
    // Show current config
    if let Ok(Some(provider)) = db.get_config("llm_provider") {
        println!("Current provider: {}", provider);
    }
    // Select provider
    let providers = ["openai", "anthropic", "ollama"];
    let selection = Select::new().with_prompt("Provider").items(&providers).interact()?;
    // Input key
    let key = if providers[selection] == "ollama" {
        Input::new().with_prompt("Endpoint URL").default("http://localhost:11434").interact_text()?
    } else {
        Password::new().with_prompt("API Key").interact()?
    };
    // Validate & save
    ...
}
```

### 3. `bulk-insert <PATH>` command

Bulk import from CSV file.

Flow:
1. Read file contents from `path`
2. Detect parser, preview first 5 rows in a table
3. Auto-detect columns (same heuristics as frontend) or prompt user to pick
4. Parse with confirmed mapping
5. Classify: DB rules first, then LLM if configured (warn if not)
6. Display classified expenses in a table using `comfy-table`:
   - Group: DB-classified, LLM-classified, Unclassified, Duplicates
   - Show source badge as colored text
7. Prompt user: "Save N expenses? (duplicates will be skipped)" using `dialoguer::Confirm`
8. Allow editing: optionally let user edit categories for unclassified items
   - Simpler approach: save to a temp file, let user edit, re-read
   - Even simpler: prompt for each unclassified expense inline
9. Save to DB, print summary

```rust
Commands::BulkInsert { path } => {
    let content = std::fs::read_to_string(&path)?;
    let db = Database::open_default()?;
    let parsers = builtin_parsers();
    let parser = detect_parser(&content, &parsers).ok_or("Unsupported format")?;
    let preview = parser.preview_rows(&content)?;
    // Display preview table...
    // Prompt for column mapping...
    // Parse and classify...
    // Display results table...
    // Confirm and save...
}
```

**Decision: column mapping interaction.** The frontend has dropdowns; CLI needs
either auto-detection (use same header heuristics) or numbered prompts:
```
Columns detected: [date, description, amount, balance]
  Title column [2 - description]:
  Amount column [3 - amount]:
  Date column [1 - date]:
  Date format [%Y-%m-%d]:
```

### 4. `insert` command

Single expense entry.

Flow:
1. Prompt: date (default today) using `dialoguer::Input` with default
2. Prompt: title
3. Prompt: amount (validate as f64)
4. Prompt: category (optional, show existing categories as suggestions)
5. Save expense + auto-create rule if category provided
6. Print confirmation

### 5. `export` command

Export expenses to CSV.

Flow:
1. If `grammar` file provided: read it as a column spec (simple format, one column name per line)
2. If no grammar: prompt user to select columns using `dialoguer::MultiSelect`
3. Fetch all expenses from DB
4. Export via `CsvExporter`
5. Write to stdout (pipeable) or to a file if redirected

Grammar file format (simple):
```
date
title
amount
category
```

### 6. `dashboard` command

Open the GUI app.

Implementation:
- Use `std::process::Command` to launch the Tauri binary
- Find binary path: same directory as CLI binary, or system PATH
- If binary not found, print helpful error

## Files to Change

| File | Change |
|---|---|
| `crates/cli/Cargo.toml` | Add `dialoguer`, `comfy-table`, `colored` |
| `crates/cli/src/main.rs` | Implement all 5 commands |

Optionally split into modules:
```
crates/cli/src/
  main.rs
  llm_conf.rs
  bulk_insert.rs
  insert.rs
  export.rs
```

## Test Scenarios

CLI tests are harder to automate due to interactive prompts. Focus on:

### Unit-testable logic (extract into functions)

1. **`test_auto_detect_columns`** — given headers `["date", "description", "amount"]`, auto-detect returns correct indices
2. **`test_auto_detect_polish_headers`** — given `["data", "tytuł", "kwota"]`, detect works
3. **`test_grammar_file_parsing`** — parse a grammar file with `date\ntitle\namount` into `ExportColumns`
4. **`test_grammar_file_unknown_column`** — unknown column name is ignored or errors

### Integration tests (using `assert_cmd` crate, optional)

5. **`test_insert_noninteractive`** — if we add `--date`, `--title`, `--amount`, `--category` flags to `insert` for scripting, test the non-interactive path
6. **`test_export_stdout`** — run `4ccountant export` and capture stdout, verify CSV output
7. **`test_bulk_insert_file_not_found`** — run with nonexistent path, expect error message

### Manual tests

8. **LLM conf flow** — run `4ccountant llm-conf`, select provider, enter key, verify saved
9. **Bulk insert flow** — run with a real CSV, walk through prompts, verify expenses saved
10. **Insert flow** — add a single expense, verify in DB
11. **Export flow** — export with column selection, verify CSV content
12. **Dashboard** — verify it attempts to launch the GUI binary

## Acceptance Criteria

- `cargo build -p accountant-cli` compiles
- All 5 commands produce meaningful output (no "not yet implemented")
- CLI shares the same SQLite DB as the desktop app
- LLM warning: if LLM not configured during bulk-insert, print a clear warning but continue
- Error messages are user-friendly (no raw Rust error dumps)
- Tables are formatted nicely with `comfy-table`
