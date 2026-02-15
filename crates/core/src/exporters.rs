use crate::models::Expense;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ExportError {
    #[error("Export failed: {0}")]
    Failed(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Configuration for which columns to include in the export.
#[derive(Debug, Clone)]
pub struct ExportColumns {
    pub date: bool,
    pub title: bool,
    pub amount: bool,
    pub category: bool,
    pub classification_source: bool,
}

impl Default for ExportColumns {
    fn default() -> Self {
        Self {
            date: true,
            title: true,
            amount: true,
            category: true,
            classification_source: false,
        }
    }
}

/// An exporter that can write expenses to a specific format.
pub trait Exporter: Send + Sync {
    /// Human-readable name (e.g. "CSV", "JSON").
    fn name(&self) -> &str;

    /// File extension for the output (e.g. "csv", "json").
    fn extension(&self) -> &str;

    /// Export expenses to bytes.
    fn export(&self, expenses: &[Expense], columns: &ExportColumns) -> Result<Vec<u8>, ExportError>;
}

/// Escape a field per RFC 4180.
fn csv_escape(field: &str) -> String {
    if field.contains(',') || field.contains('"') || field.contains('\n') {
        format!("\"{}\"", field.replace('"', "\"\""))
    } else {
        field.to_string()
    }
}

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
        fields.push(
            expense.classification_source
                .as_ref()
                .map(|s| s.to_string())
                .unwrap_or_default()
        );
    }
    fields
}

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{ClassificationSource, Expense};
    use chrono::NaiveDate;

    fn sample_expenses() -> Vec<Expense> {
        vec![
            Expense {
                id: Some(1),
                title: "Coffee".to_string(),
                amount: 4.50,
                date: NaiveDate::from_ymd_opt(2025, 1, 15).unwrap(),
                category: Some("Drinks".to_string()),
                classification_source: Some(ClassificationSource::Database),
            },
            Expense {
                id: Some(2),
                title: "UBER TRIP".to_string(),
                amount: 12.99,
                date: NaiveDate::from_ymd_opt(2025, 1, 16).unwrap(),
                category: Some("Transport".to_string()),
                classification_source: Some(ClassificationSource::Llm),
            },
            Expense {
                id: Some(3),
                title: "Mystery shop".to_string(),
                amount: 100.00,
                date: NaiveDate::from_ymd_opt(2025, 1, 17).unwrap(),
                category: None,
                classification_source: None,
            },
        ]
    }

    fn export_to_string(expenses: &[Expense], columns: &ExportColumns) -> String {
        let exporter = CsvExporter;
        let bytes = exporter.export(expenses, columns).unwrap();
        String::from_utf8(bytes).unwrap()
    }

    #[test]
    fn test_export_all_default_columns() {
        let csv = export_to_string(&sample_expenses(), &ExportColumns::default());
        let lines: Vec<&str> = csv.lines().collect();
        assert_eq!(lines[0], "date,title,amount,category");
        assert_eq!(lines.len(), 4); // header + 3 rows
        assert!(lines[1].contains("Coffee"));
        assert!(lines[1].contains("4.50"));
        assert!(lines[1].contains("Drinks"));
    }

    #[test]
    fn test_export_subset_columns() {
        let columns = ExportColumns {
            date: false,
            title: true,
            amount: true,
            category: false,
            classification_source: false,
        };
        let csv = export_to_string(&sample_expenses(), &columns);
        let lines: Vec<&str> = csv.lines().collect();
        assert_eq!(lines[0], "title,amount");
        // Each row should have 2 fields
        assert_eq!(lines[1].split(',').count(), 2);
    }

    #[test]
    fn test_export_with_classification_source() {
        let columns = ExportColumns {
            date: true,
            title: true,
            amount: true,
            category: true,
            classification_source: true,
        };
        let csv = export_to_string(&sample_expenses(), &columns);
        let lines: Vec<&str> = csv.lines().collect();
        assert_eq!(lines[0], "date,title,amount,category,source");
        assert!(lines[1].contains("Database"));
        assert!(lines[2].contains("Llm"));
        // Row 3 has no source — should be empty
        let fields: Vec<&str> = lines[3].split(',').collect();
        assert_eq!(fields.last().unwrap(), &"");
    }

    #[test]
    fn test_csv_escape_comma_in_title() {
        let expenses = vec![Expense {
            id: None,
            title: "Coffee, large".to_string(),
            amount: 5.00,
            date: NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
            category: None,
            classification_source: None,
        }];
        let csv = export_to_string(&expenses, &ExportColumns::default());
        assert!(csv.contains("\"Coffee, large\""));
    }

    #[test]
    fn test_csv_escape_quotes_in_title() {
        let expenses = vec![Expense {
            id: None,
            title: r#"She said "hello""#.to_string(),
            amount: 5.00,
            date: NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
            category: None,
            classification_source: None,
        }];
        let csv = export_to_string(&expenses, &ExportColumns::default());
        assert!(csv.contains(r#""She said ""hello""""#));
    }

    #[test]
    fn test_csv_escape_newline_in_title() {
        let expenses = vec![Expense {
            id: None,
            title: "Line1\nLine2".to_string(),
            amount: 5.00,
            date: NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
            category: None,
            classification_source: None,
        }];
        let csv = export_to_string(&expenses, &ExportColumns::default());
        assert!(csv.contains("\"Line1\nLine2\""));
    }

    #[test]
    fn test_csv_escape_plain_title() {
        let escaped = csv_escape("Coffee");
        assert_eq!(escaped, "Coffee");
    }

    #[test]
    fn test_export_empty_expenses() {
        let csv = export_to_string(&[], &ExportColumns::default());
        let lines: Vec<&str> = csv.lines().collect();
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0], "date,title,amount,category");
    }

    #[test]
    fn test_export_no_columns_selected() {
        let columns = ExportColumns {
            date: false,
            title: false,
            amount: false,
            category: false,
            classification_source: false,
        };
        let csv = export_to_string(&sample_expenses(), &columns);
        let lines: Vec<&str> = csv.lines().collect();
        // Header is empty, each data line is also empty
        assert_eq!(lines[0], "");
    }

    #[test]
    fn test_export_missing_category() {
        let columns = ExportColumns {
            date: false,
            title: true,
            amount: false,
            category: true,
            classification_source: false,
        };
        let expenses = vec![Expense {
            id: None,
            title: "Unknown".to_string(),
            amount: 10.0,
            date: NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
            category: None,
            classification_source: None,
        }];
        let csv = export_to_string(&expenses, &columns);
        let lines: Vec<&str> = csv.lines().collect();
        assert_eq!(lines[1], "Unknown,");
    }

    #[test]
    fn test_export_amount_formatting() {
        let expenses = vec![Expense {
            id: None,
            title: "Item".to_string(),
            amount: 4.5,
            date: NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
            category: None,
            classification_source: None,
        }];
        let csv = export_to_string(&expenses, &ExportColumns::default());
        assert!(csv.contains("4.50"));
    }

    #[test]
    fn test_export_amount_formatting_whole_number() {
        let expenses = vec![Expense {
            id: None,
            title: "Item".to_string(),
            amount: 100.0,
            date: NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
            category: None,
            classification_source: None,
        }];
        let csv = export_to_string(&expenses, &ExportColumns::default());
        assert!(csv.contains("100.00"));
    }

    #[test]
    fn test_exporter_name_and_extension() {
        let exporter = CsvExporter;
        assert_eq!(exporter.name(), "CSV");
        assert_eq!(exporter.extension(), "csv");
    }
}
