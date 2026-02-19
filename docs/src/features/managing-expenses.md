# Managing Expenses

The **Expenses** page shows your expenses in a paginated table sorted by date (newest first). From here you can search, filter, edit, delete, and batch-delete expenses.

## Search and filters

A **search bar** at the top lets you find expenses by title. Typing filters the list as you type (with a short debounce). Below the search bar you'll find filter controls:

- **Category** — dropdown to show only a specific category, or "Uncategorized" for expenses without a category
- **From / To** — date range inputs to narrow results by date
- **Min / Max amount** — number inputs to filter by amount range

Filters combine with AND logic — only expenses matching all active filters are shown. Click **"Clear filters"** to reset everything.

## Pagination

The expense list paginates at 50 rows by default. At the bottom of the table you'll see:

- **"Showing X-Y of Z expenses"** — current page range and total matching count
- **Rows: 25 / 50 / 100** — click to change the page size
- **Previous / Next** arrows to navigate between pages

Changing filters or search resets to page 1.

## Editing an expense

Hover over any row to reveal the edit (pencil) and delete (trash) icons on the right side.

Click the pencil icon to enter inline edit mode. The row transforms into editable fields:

- **Date** — date picker input
- **Title** — text input
- **Amount** — number input
- **Category** — text input with autocomplete from your existing categories

Click the **checkmark** to save your changes, or the **X** to cancel and restore the original values. Only one row can be edited at a time.

When you save an edit with a category, the app creates a classification rule for the title, just like when adding a new expense. The classification source is updated to "Manual".

## Deleting a single expense

Click the trash icon on any row. The icons change to a confirmation prompt (checkmark to confirm, X to cancel). Confirm to permanently delete the expense.

## Batch delete

Each row has a checkbox on the left. Use these to select multiple expenses, or click the checkbox in the header to select all.

When one or more expenses are selected, a **"Delete N selected"** button appears in the top-right corner. Click it to see a confirmation bar, then confirm to delete all selected expenses at once.

## Export

The **Export CSV** button (visible when you have expenses) lets you choose which columns to include and save a CSV file. See [Export](./export.md) for details.
