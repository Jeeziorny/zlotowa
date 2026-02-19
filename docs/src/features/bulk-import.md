# Bulk Import

Bulk import lets you load many expenses at once from a CSV file exported by your bank.

## Steps

### 1. Input

Paste CSV text directly into the text area, or drag-and-drop / browse for a `.csv` or `.txt` file.

### 2. Column mapping

The app auto-detects the CSV format (delimiter, columns) and shows a compact preview with 1 data row. Click column headers to assign them as **Title**, **Amount**, or **Date**. A small popover appears with the available roles. Unassigned columns show a "Click to assign" hint. The header text changes color to indicate its assignment (green for Title, blue for Amount, purple for Date).

The app tries to auto-detect column assignments by looking at header names (supports English and Polish headers like "tytuł", "kwota", "data").

The preview also shows inline validation:

- **Amount column** — parsed values are shown next to the raw text (e.g., `1.234,56 (1234.56)`). A red `?` appears for unparseable values
- **Date column** — a red `?` appears if the value doesn't match the selected date format

If no LLM API key is configured, a dismissible amber info bar warns that unmatched expenses will need manual categorization. Click the × button to dismiss it.

Supported date formats:

| Format | Example |
|---|---|
| YYYY-MM-DD | 2024-01-15 |
| DD-MM-YYYY | 15-01-2024 |
| MM-DD-YYYY | 01-15-2024 |
| DD/MM/YYYY | 15/01/2024 |
| MM/DD/YYYY | 01/15/2024 |
| YYYY/MM/DD | 2024/01/15 |
| DD.MM.YYYY | 15.01.2024 |

The date format is auto-detected from the first few data rows. You can override it with the date format selector below the preview.

### 3. Review

A step indicator at the top of the page shows your progress through the import process.

When you click "Next: Classify & Review", a full-screen overlay with a spinner shows classification progress while the AI processes your expenses.

Each expense is displayed as a card (not a table row) with:

- **Top row** — date, title, amount (read-only), plus a confidence badge for AI-classified items (**High** green, **Medium** yellow, **Low** red)
- **Bottom row** — category chip input and match keyword input

Expenses are grouped into sections:

- **Classified by rules** — matched by existing database rules (blue dot)
- **Classified by AI** — classified by the LLM provider (purple dot, only if LLM is configured)
- **Needs your input** — no match found (yellow dot)
- **Duplicates** — expenses that already exist in the database (matched by title + amount + date), shown separately and skipped on save

**Category chip input:** Click the category field to see an autocomplete dropdown of existing categories from your database. Type to filter suggestions. Press Enter or click a suggestion to select it — the category appears as a chip with a × button to remove it. You can also type a new category name that doesn't exist yet.

The AI-classified and unclassified sections include a **Match keyword** field. This lets you edit what the auto-generated classification rule will match. By default it's the full title, but you can trim it to just the merchant name (e.g., "LIDL" instead of the full bank transaction string). When you edit a match keyword and the row has a category, the app automatically applies the same keyword and category to other unclassified rows whose titles contain the keyword.

You can edit any category before saving. Categories you assign here become new rules for future imports.

### 4. Save

Click save to store all non-duplicate expenses. The count of saved items is shown on the confirmation screen. Each bulk upload is tracked as a batch with filename and timestamp.

## Undo an upload

Every bulk upload is recorded in **Settings > Upload History**. Each entry shows the filename (or "Pasted data"), upload date, and expense count. Click **Undo** to delete all expenses from that upload. A confirmation step prevents accidental deletion. Classification rules created during the upload are kept — only the expenses are removed.

## Supported formats

Currently only CSV is supported. The parser auto-detects delimiters: comma, semicolon, tab, and pipe (`|`).

Amount parsing handles both US format (1,234.56) and European format (1.234,56), and strips currency symbols.
