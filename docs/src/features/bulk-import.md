# Bulk Import

Bulk import lets you load many expenses at once from a CSV file exported by your bank.

## Steps

### 1. Input

Paste CSV text directly into the text area, or drag-and-drop / browse for a `.csv` or `.txt` file.

### 2. Column mapping

The app auto-detects the CSV format (delimiter, columns) and shows a preview. You confirm:

- **Title column** — which column contains the expense description
- **Amount column** — which column contains the monetary value
- **Date column** — which column contains the date
- **Date format** — how dates are formatted in your file

The app tries to auto-detect column assignments by looking at header names (supports English and Polish headers like "tytuł", "kwota", "data").

Supported date formats:
- `2024-01-15` (YYYY-MM-DD)
- `15-01-2024` (DD-MM-YYYY)
- `01-15-2024` (MM-DD-YYYY)
- `15/01/2024` (DD/MM/YYYY)
- `01/15/2024` (MM/DD/YYYY)
- `2024/01/15` (YYYY/MM/DD)
- `15.01.2024` (DD.MM.YYYY)

### 3. Review

A step indicator at the top of the page shows your progress through the import process.

After parsing, each expense is run through the classification pipeline. If LLM is configured, unmatched expenses are automatically sent to the AI provider for classification. You'll see expenses grouped into three sections:

- **Classified by rules** — matched by existing database rules (blue dot)
- **Classified by AI** — classified by the LLM provider (purple dot, only if LLM is configured)
- **Needs your input** — no match found, you can type a category inline (yellow dot)
- **Duplicates** — expenses that already exist in the database (matched by title + amount + date), shown separately and skipped on save

You can edit any category before saving. Categories you assign here become new rules for future imports.

### 4. Save

Click save to store all non-duplicate expenses. The count of saved items is shown on the confirmation screen.

## Supported formats

Currently only CSV is supported. The parser auto-detects delimiters: comma, semicolon, tab, and pipe (`|`).
