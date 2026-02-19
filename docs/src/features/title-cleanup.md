# Title Cleanup

Bank transaction titles are often full of noise: card numbers, payment processor names, reference codes, and station IDs. The **Title Cleanup** feature lets you define find-and-replace rules that strip or transform these fragments across your expenses.

## Creating Rules

Navigate to **Title Cleanup** in the sidebar. Click **+ Add Rule** to open the rule form.

Each rule has three fields:

- **Pattern** -- the text to find in expense titles
- **Replacement** -- what to replace it with (leave empty to delete the matched portion)
- **Regex toggle** -- whether the pattern is a literal string match or a regular expression

### Literal vs Regex

- **Literal** (default): the pattern is matched exactly as typed. Special characters like `.`, `(`, `)` are treated as plain text.
- **Regex**: the pattern is interpreted as a regular expression. Use this for flexible matching like `\d{6}\*{6}\d{4}` to match masked card numbers.

## Previewing Changes

Before applying a rule, click **Preview** on any rule in the list. This shows a table of all expenses that would be affected:

- **Original** title (shown in red)
- **Cleaned** title after the rule is applied (shown in green)

No changes are made to the database during preview.

## Applying Rules

From the preview table:

1. Review the before/after for each expense
2. Use the checkboxes to select which expenses to update (all are selected by default)
3. Click **Apply to N selected** to commit the changes

Applied changes are permanent -- the expense titles are updated in the database. The preview refreshes automatically after applying, showing any remaining matches.

## Managing Rules

Rules are saved for reuse. After importing new bank data, you can re-preview and apply existing rules to clean up the new titles.

- **Edit** a rule by clicking the pencil icon
- **Delete** a rule by clicking the X icon (with confirmation)

## Bulk Import

Title cleanup rules **do not run automatically** during bulk import. When you upload a CSV through Bulk Upload, the expenses are parsed, classified, and saved with their original bank titles — no cleanup rules are applied.

After importing new bank data, revisit the **Title Cleanup** page and re-preview your existing rules. Any new expenses matching a rule will appear in the preview, ready to be cleaned.

## Tips

- Start with literal rules for common noise patterns like `PLATNOSC KARTA` or `Zakup BLIK`
- Use regex rules when the noise contains variable parts (card numbers, reference codes, amounts)
- After replacement, multiple consecutive spaces are automatically collapsed and leading/trailing whitespace is trimmed
- Invalid regex patterns will show an error message without crashing
