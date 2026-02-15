# Task 4: Implement CSV Exporter in Core Crate

**Track:** B — Export Feature (1 of 2)
**Blocked by:** nothing
**Blocks:** Task 5

## Problem

The initial prompt (`initial_prompt.md` lines 77-78) requires CSV export with user-selectable
columns. The trait and config struct exist in `crates/core/src/exporters.rs` but there
are zero implementations. The docs don't mention export at all (tracked in Task 8).

## Current State

**`crates/core/src/exporters.rs`** defines:
```rust
pub struct ExportColumns {
    pub date: bool,
    pub title: bool,
    pub amount: bool,
    pub category: bool,
}

impl Default for ExportColumns {
    fn default() -> Self {
        Self { date: true, title: true, amount: true, category: true }
    }
}

pub trait Exporter: Send + Sync {
    fn name(&self) -> &str;
    fn extension(&self) -> &str;
    fn export(&self, expenses: &[Expense], columns: &ExportColumns) -> Result<Vec<u8>, ExportError>;
}
```

**`crates/core/src/models.rs`** — `Expense` struct:
```rust
pub struct Expense {
    pub id: Option<i64>,
    pub title: String,
    pub amount: f64,
    pub date: NaiveDate,
    pub category: Option<String>,
    pub classification_source: Option<ClassificationSource>,
}
```

Note: `ExportColumns` has 4 fields but `Expense` also has `classification_source`.
Consider adding `classification_source: bool` to `ExportColumns`.

## Scope

### 1. Add `classification_source` to `ExportColumns`

```rust
pub struct ExportColumns {
    pub date: bool,
    pub title: bool,
    pub amount: bool,
    pub category: bool,
    pub classification_source: bool,
}
```

Update `Default` to include `classification_source: false` (opt-in, not everyone cares).

### 2. Implement `CsvExporter`

```rust
pub struct CsvExporter;

impl Exporter for CsvExporter {
    fn name(&self) -> &str { "CSV" }
    fn extension(&self) -> &str { "csv" }

    fn export(&self, expenses: &[Expense], columns: &ExportColumns) -> Result<Vec<u8>, ExportError> {
        let mut output = Vec::new();

        // Header row
        let headers = build_headers(columns);
        output.extend_from_slice(headers.join(",").as_bytes());
        output.push(b'\n');

        // Data rows
        for expense in expenses {
            let fields = build_fields(expense, columns);
            let line = fields.iter().map(|f| csv_escape(f)).collect::<Vec<_>>().join(",");
            output.extend_from_slice(line.as_bytes());
            output.push(b'\n');
        }

        Ok(output)
    }
}
```

### 3. CSV escaping

Implement proper CSV escaping per RFC 4180:
- Fields containing commas, double quotes, or newlines must be quoted
- Double quotes inside a field are escaped as `""`
- Other fields are written as-is

```rust
fn csv_escape(field: &str) -> String {
    if field.contains(',') || field.contains('"') || field.contains('\n') {
        format!("\"{}\"", field.replace('"', "\"\""))
    } else {
        field.to_string()
    }
}
```

### 4. Helper functions

```rust
fn build_headers(columns: &ExportColumns) -> Vec<&str> {
    let mut headers = Vec::new();
    if columns.date { headers.push("date"); }
    if columns.title { headers.push("title"); }
    if columns.amount { headers.push("amount"); }
    if columns.category { headers.push("category"); }
    if columns.classification_source { headers.push("source"); }
    headers
}

fn build_fields(expense: &Expense, columns: &ExportColumns) -> Vec<String> {
    let mut fields = Vec::new();
    if columns.date { fields.push(expense.date.to_string()); }
    if columns.title { fields.push(expense.title.clone()); }
    if columns.amount { fields.push(format!("{:.2}", expense.amount)); }
    if columns.category { fields.push(expense.category.clone().unwrap_or_default()); }
    if columns.classification_source {
        fields.push(expense.classification_source.as_ref().map(|s| s.to_string()).unwrap_or_default());
    }
    headers
}
```

## Files to Change

| File | Change |
|---|---|
| `crates/core/src/exporters.rs` | Add `CsvExporter`, `csv_escape`, helper fns, add `classification_source` to `ExportColumns` |

## Test Scenarios

```rust
#[cfg(test)]
mod tests {
    // Test data helper
    fn sample_expenses() -> Vec<Expense> { /* 3 expenses with varied data */ }
}
```

1. **`test_export_all_columns`** — export with default columns (all true), verify header is `date,title,amount,category` and each row has 4 fields
2. **`test_export_subset_columns`** — export with only `title` and `amount` enabled, verify header is `title,amount` and rows have 2 fields
3. **`test_export_with_classification_source`** — enable all 5 columns, verify `source` column contains "Database"/"Llm"/"Manual"/""
4. **`test_csv_escape_comma_in_title`** — expense with title `"Coffee, large"` exports as `"Coffee, large"` (quoted)
5. **`test_csv_escape_quotes_in_title`** — expense with title `She said "hello"` exports as `"She said ""hello"""` (double-escaped)
6. **`test_csv_escape_newline_in_title`** — expense with newline in title gets quoted
7. **`test_csv_escape_plain_title`** — expense with title `Coffee` exports unquoted
8. **`test_export_empty_expenses`** — exporting an empty slice returns just the header row
9. **`test_export_no_columns_selected`** — all columns false, returns empty lines (or just newlines) — edge case
10. **`test_export_missing_category`** — expense with `category: None` exports as empty string in that column
11. **`test_export_amount_formatting`** — amount `4.5` exports as `4.50` (2 decimal places)
12. **`test_roundtrip_parse_export`** — export expenses to CSV, then parse the CSV back with `CsvParser`, verify data matches (integration test)

## Acceptance Criteria

- `cargo test -p accountant-core` passes with all new tests
- `CsvExporter` produces valid RFC 4180 CSV
- Column selection works correctly
- No changes to other crates
- Amount is formatted to 2 decimal places consistently
