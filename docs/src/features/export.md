# Export

Export your expenses to CSV from the **Expenses** page.

## How to export

1. Go to **Expenses** in the sidebar
2. Click **Export CSV** (visible when you have at least one expense)
3. Select which columns to include:
   - **Date**
   - **Title**
   - **Amount**
   - **Category**
   - **Classification Source** (how each expense was classified: Database, Llm, or Manual)
4. Click **Download CSV** — a native file save dialog opens with a default name of `zlotowa-export-YYYY-MM-DD.csv`. Choose where to save the file.

## CLI export

You can also export from the terminal:

```bash
# Interactive column selection
zlotowa export

# Using a grammar file (one column name per line)
zlotowa export columns.txt
```

The CLI writes CSV to stdout, so you can pipe it:

```bash
zlotowa export > expenses.csv
```
