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
   - **Classification Source** (how each expense was classified: DB Rule, LLM, or Manual)
4. Click **Download CSV**

The file is saved as `4ccountant-export-YYYY-MM-DD.csv`.

## CLI export

You can also export from the terminal:

```bash
# Interactive column selection
4ccountant export

# Using a grammar file (one column name per line)
4ccountant export columns.txt
```

The CLI writes CSV to stdout, so you can pipe it:

```bash
4ccountant export > expenses.csv
```
