# CLI

złotówa includes a terminal interface with the same core features as the desktop app.

## Installation

```bash
cargo build --release -p accountant-cli
```

The binary is at `target/release/zlotowa`.

## Commands

| Command | Description |
|---|---|
| `zlotowa llm-conf` | Configure LLM provider and API key |
| `zlotowa insert` | Add a single expense (interactive or with flags) |
| `zlotowa bulk-insert <file>` | Import expenses from a CSV file |
| `zlotowa export [grammar]` | Export expenses to CSV |
| `zlotowa dashboard` | Launch the GUI app |

### llm-conf

Interactive setup: choose a provider, enter your API key, and validate the connection.

```bash
zlotowa llm-conf
```

### insert

Add a single expense. Run without flags for interactive mode, or pass values directly:

```bash
# Interactive
zlotowa insert

# Non-interactive
zlotowa insert --date 2025-01-15 --title "Coffee" --amount 4.50 --category "Drinks"
```

### bulk-insert

Import expenses from a CSV file. The app auto-detects the format, shows a column mapping for confirmation, classifies expenses using database rules (and LLM if configured), and displays a review table before saving.

```bash
zlotowa bulk-insert bank-export.csv
```

### export

Export all expenses to CSV. Pass a grammar file (one column name per line: `date`, `title`, `amount`, `category`, `source`) or select columns interactively.

```bash
# Interactive column selection
zlotowa export

# Grammar file
zlotowa export columns.txt

# Pipe to file
zlotowa export > expenses.csv
```

### dashboard

Attempts to launch the GUI binary.

```bash
zlotowa dashboard
```
