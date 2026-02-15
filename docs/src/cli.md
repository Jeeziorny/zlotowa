# CLI

4ccountant includes a terminal interface with the same core features as the desktop app.

## Installation

```bash
cargo build --release -p accountant-cli
```

The binary is at `target/release/4ccountant`.

## Commands

| Command | Description |
|---|---|
| `4ccountant llm-conf` | Configure LLM provider and API key |
| `4ccountant insert` | Add a single expense (interactive or with flags) |
| `4ccountant bulk-insert <file>` | Import expenses from a CSV file |
| `4ccountant export [grammar]` | Export expenses to CSV |
| `4ccountant dashboard` | Launch the GUI app |

### llm-conf

Interactive setup: choose a provider, enter your API key, and validate the connection.

```bash
4ccountant llm-conf
```

### insert

Add a single expense. Run without flags for interactive mode, or pass values directly:

```bash
# Interactive
4ccountant insert

# Non-interactive
4ccountant insert --date 2025-01-15 --title "Coffee" --amount 4.50 --category "Drinks"
```

### bulk-insert

Import expenses from a CSV file. The app auto-detects the format, shows a column mapping for confirmation, classifies expenses using database rules (and LLM if configured), and displays a review table before saving.

```bash
4ccountant bulk-insert bank-export.csv
```

### export

Export all expenses to CSV. Pass a grammar file (one column name per line: `date`, `title`, `amount`, `category`, `source`) or select columns interactively.

```bash
# Interactive column selection
4ccountant export

# Grammar file
4ccountant export columns.txt

# Pipe to file
4ccountant export > expenses.csv
```

### dashboard

Attempts to launch the GUI binary.

```bash
4ccountant dashboard
```
