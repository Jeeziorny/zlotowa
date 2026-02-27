use super::{ColumnMapping, ParseError, Parser};
use crate::models::ParsedExpense;
use chrono::NaiveDate;
use log::{debug, info, warn};

pub struct CsvParser;

impl CsvParser {
    fn detect_delimiter(input: &str) -> char {
        let first_lines: Vec<&str> = input.lines().take(5).collect();
        let delimiters = [',', ';', '\t', '|'];

        let result = delimiters
            .iter()
            .copied()
            .max_by_key(|&d| {
                // Pick the delimiter that gives the most consistent column count
                let counts: Vec<usize> = first_lines
                    .iter()
                    .map(|line| line.split(d).count())
                    .collect();
                if counts.is_empty() {
                    return 0;
                }
                let first = counts[0];
                if first <= 1 {
                    return 0;
                }
                // Score = column count if consistent, 0 otherwise
                let score = if counts.iter().all(|&c| c == first) {
                    first
                } else {
                    0
                };
                debug!("CSV delimiter '{d}': score={score}");
                score
            })
            .unwrap_or(',');
        result
    }

    fn split_csv_line(line: &str, delimiter: char) -> Vec<String> {
        let mut fields = Vec::new();
        let mut current = String::new();
        let mut in_quotes = false;

        for ch in line.chars() {
            if ch == '"' {
                in_quotes = !in_quotes;
            } else if ch == delimiter && !in_quotes {
                fields.push(current.trim().trim_matches('"').to_string());
                current = String::new();
            } else {
                current.push(ch);
            }
        }
        fields.push(current.trim().trim_matches('"').to_string());
        fields
    }
}

impl Parser for CsvParser {
    fn name(&self) -> &str {
        "CSV"
    }

    fn detect(&self, input: &str) -> f64 {
        let lines: Vec<&str> = input.lines().filter(|l| !l.trim().is_empty()).collect();
        if lines.len() < 2 {
            return 0.0;
        }

        let delim = Self::detect_delimiter(input);
        let col_counts: Vec<usize> = lines.iter().map(|l| l.split(delim).count()).collect();

        if col_counts[0] < 2 {
            return 0.0;
        }

        // Check consistency of column counts
        let consistent = col_counts.iter().filter(|&&c| c == col_counts[0]).count();
        let ratio = consistent as f64 / col_counts.len() as f64;

        if ratio > 0.8 {
            0.8
        } else if ratio > 0.5 {
            0.5
        } else {
            0.1
        }
    }

    fn preview_rows(&self, input: &str) -> Result<Vec<Vec<String>>, ParseError> {
        let delim = Self::detect_delimiter(input);
        let rows: Vec<Vec<String>> = input
            .lines()
            .filter(|l| !l.trim().is_empty())
            .map(|line| Self::split_csv_line(line, delim))
            .collect();

        if rows.len() < 2 {
            return Err(ParseError::ParseFailed(
                "Need at least a header row and one data row".to_string(),
            ));
        }

        Ok(rows)
    }

    fn parse(
        &self,
        input: &str,
        mapping: &ColumnMapping,
    ) -> Result<Vec<ParsedExpense>, ParseError> {
        let delim = Self::detect_delimiter(input);
        let lines: Vec<&str> = input
            .lines()
            .filter(|l| !l.trim().is_empty())
            .collect();

        if lines.len() < 2 {
            return Err(ParseError::ParseFailed("Not enough rows".to_string()));
        }

        // Skip header row
        let mut expenses = Vec::new();
        for (i, line) in lines.iter().skip(1).enumerate() {
            let fields = Self::split_csv_line(line, delim);

            let title = fields
                .get(mapping.title_index)
                .ok_or_else(|| {
                    ParseError::ParseFailed(format!("Row {}: missing title column", i + 2))
                })?
                .clone();

            let amount_str = fields
                .get(mapping.amount_index)
                .ok_or_else(|| {
                    ParseError::ParseFailed(format!("Row {}: missing amount column", i + 2))
                })?;

            // Handle various number formats: "1,234.56", "1.234,56", "-123.45"
            let amount = parse_amount(amount_str).map_err(|_| {
                let msg = format!("Row {}: can't parse amount '{}'", i + 2, amount_str);
                warn!("CSV parse error: {msg}");
                ParseError::ParseFailed(msg)
            })?;

            let date_str = fields
                .get(mapping.date_index)
                .ok_or_else(|| {
                    ParseError::ParseFailed(format!("Row {}: missing date column", i + 2))
                })?;

            let date = NaiveDate::parse_from_str(date_str, &mapping.date_format).map_err(|_| {
                let msg = format!(
                    "Row {}: can't parse date '{}' with format '{}'",
                    i + 2, date_str, mapping.date_format
                );
                warn!("CSV parse error: {msg}");
                ParseError::ParseFailed(msg)
            })?;

            expenses.push(ParsedExpense {
                title,
                amount,
                date,
            });
        }

        info!("CSV parsed {} rows", expenses.len());
        Ok(expenses)
    }
}

pub(crate) fn parse_amount(s: &str) -> Result<f64, ()> {
    let s = s.trim();

    // Remove currency symbols and whitespace
    let cleaned: String = s
        .chars()
        .filter(|c| c.is_ascii_digit() || *c == '.' || *c == ',' || *c == '-' || *c == '+')
        .collect();

    if cleaned.is_empty() {
        return Err(());
    }

    // Detect if comma is decimal separator (European format: "1.234,56" or "123,45")
    // vs thousand separator (US format: "1,234.56")
    let last_comma = cleaned.rfind(',');
    let last_dot = cleaned.rfind('.');

    let normalized = match (last_comma, last_dot) {
        (Some(c), Some(d)) if c > d => {
            // Comma after dot: "1.234,56" -> European decimal
            cleaned.replace('.', "").replace(',', ".")
        }
        (Some(_), Some(_)) => {
            // Dot after comma: "1,234.56" -> US format
            cleaned.replace(',', "")
        }
        (Some(_), None) => {
            // Only commas: could be "123,45" (decimal) or "1,234" (thousands)
            // If exactly 2 digits after last comma, treat as decimal
            let after_comma = &cleaned[last_comma.unwrap() + 1..];
            if after_comma.len() == 2 {
                cleaned.replace(',', ".")
            } else {
                cleaned.replace(',', "")
            }
        }
        _ => cleaned,
    };

    let value = normalized.parse::<f64>().map_err(|_| ())?;
    if !value.is_finite() {
        return Err(());
    }
    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parsers::{ColumnMapping, Parser};

    // ── parse_amount ──

    #[test]
    fn amount_simple_integer() {
        assert_eq!(parse_amount("100").unwrap(), 100.0);
    }

    #[test]
    fn amount_simple_decimal() {
        assert_eq!(parse_amount("12.34").unwrap(), 12.34);
    }

    #[test]
    fn amount_negative() {
        assert_eq!(parse_amount("-45.67").unwrap(), -45.67);
    }

    #[test]
    fn amount_positive_sign() {
        assert_eq!(parse_amount("+100.50").unwrap(), 100.50);
    }

    #[test]
    fn amount_us_thousands() {
        assert_eq!(parse_amount("1,234.56").unwrap(), 1234.56);
    }

    #[test]
    fn amount_european_decimal() {
        assert_eq!(parse_amount("1.234,56").unwrap(), 1234.56);
    }

    #[test]
    fn amount_european_short() {
        // "123,45" — 2 digits after comma → European decimal
        assert_eq!(parse_amount("123,45").unwrap(), 123.45);
    }

    #[test]
    fn amount_thousands_only_comma() {
        // "1,234" — 3 digits after comma → thousands separator
        assert_eq!(parse_amount("1,234").unwrap(), 1234.0);
    }

    #[test]
    fn amount_currency_symbol_dollar() {
        assert_eq!(parse_amount("$99.99").unwrap(), 99.99);
    }

    #[test]
    fn amount_currency_symbol_euro() {
        assert_eq!(parse_amount("€1.234,56").unwrap(), 1234.56);
    }

    #[test]
    fn amount_whitespace() {
        assert_eq!(parse_amount("  42.00  ").unwrap(), 42.0);
    }

    #[test]
    fn amount_empty_string() {
        assert!(parse_amount("").is_err());
    }

    #[test]
    fn amount_non_numeric() {
        assert!(parse_amount("abc").is_err());
    }

    #[test]
    fn amount_nan_rejected() {
        assert!(parse_amount("NaN").is_err());
    }

    #[test]
    fn amount_inf_rejected() {
        // "inf" gets filtered to empty by the char filter, so it's rejected
        assert!(parse_amount("inf").is_err());
        assert!(parse_amount("Infinity").is_err());
    }

    // ── detect_delimiter ──

    #[test]
    fn detect_comma_delimiter() {
        let input = "a,b,c\n1,2,3\n4,5,6";
        assert_eq!(CsvParser::detect_delimiter(input), ',');
    }

    #[test]
    fn detect_semicolon_delimiter() {
        let input = "a;b;c\n1;2;3\n4;5;6";
        assert_eq!(CsvParser::detect_delimiter(input), ';');
    }

    #[test]
    fn detect_tab_delimiter() {
        let input = "a\tb\tc\n1\t2\t3\n4\t5\t6";
        assert_eq!(CsvParser::detect_delimiter(input), '\t');
    }

    #[test]
    fn detect_pipe_delimiter() {
        let input = "a|b|c\n1|2|3\n4|5|6";
        assert_eq!(CsvParser::detect_delimiter(input), '|');
    }

    // ── split_csv_line ──

    #[test]
    fn split_simple_line() {
        let fields = CsvParser::split_csv_line("a,b,c", ',');
        assert_eq!(fields, vec!["a", "b", "c"]);
    }

    #[test]
    fn split_quoted_field_with_delimiter() {
        let fields = CsvParser::split_csv_line("\"hello, world\",b,c", ',');
        assert_eq!(fields, vec!["hello, world", "b", "c"]);
    }

    #[test]
    fn split_trims_whitespace() {
        let fields = CsvParser::split_csv_line("  a , b , c  ", ',');
        assert_eq!(fields, vec!["a", "b", "c"]);
    }

    // ── detect (confidence) ──

    #[test]
    fn detect_returns_high_for_consistent_csv() {
        let parser = CsvParser;
        let input = "date,title,amount\n2025-01-01,Coffee,4.50\n2025-01-02,Lunch,12.00";
        assert!(parser.detect(input) >= 0.8);
    }

    #[test]
    fn detect_returns_zero_for_single_line() {
        let parser = CsvParser;
        assert_eq!(parser.detect("just one line"), 0.0);
    }

    #[test]
    fn detect_returns_zero_for_single_column() {
        let parser = CsvParser;
        let input = "a\nb\nc";
        assert_eq!(parser.detect(input), 0.0);
    }

    // ── preview_rows ──

    #[test]
    fn preview_rows_returns_all_rows_including_header() {
        let parser = CsvParser;
        let input = "date,title,amount\n2025-01-01,Coffee,4.50\n2025-01-02,Lunch,12.00";
        let rows = parser.preview_rows(input).unwrap();
        assert_eq!(rows.len(), 3);
        assert_eq!(rows[0], vec!["date", "title", "amount"]);
    }

    #[test]
    fn preview_rows_errors_on_single_row() {
        let parser = CsvParser;
        assert!(parser.preview_rows("just,a,header").is_err());
    }

    // ── parse ──

    #[test]
    fn parse_basic_csv() {
        let parser = CsvParser;
        let input = "date,title,amount\n2025-01-15,Coffee,4.50\n2025-01-16,Lunch,12.00";
        let mapping = ColumnMapping {
            date_index: 0,
            title_index: 1,
            amount_index: 2,
            date_format: "%Y-%m-%d".to_string(),
        };

        let expenses = parser.parse(input, &mapping).unwrap();
        assert_eq!(expenses.len(), 2);
        assert_eq!(expenses[0].title, "Coffee");
        assert_eq!(expenses[0].amount, 4.50);
        assert_eq!(expenses[0].date.to_string(), "2025-01-15");
        assert_eq!(expenses[1].title, "Lunch");
        assert_eq!(expenses[1].amount, 12.00);
    }

    #[test]
    fn parse_semicolon_csv() {
        let parser = CsvParser;
        let input = "date;title;amount\n15.01.2025;Kaffee;4,50\n16.01.2025;Mittagessen;12,00";
        let mapping = ColumnMapping {
            date_index: 0,
            title_index: 1,
            amount_index: 2,
            date_format: "%d.%m.%Y".to_string(),
        };

        let expenses = parser.parse(input, &mapping).unwrap();
        assert_eq!(expenses.len(), 2);
        assert_eq!(expenses[0].title, "Kaffee");
        assert_eq!(expenses[0].amount, 4.50);
    }

    #[test]
    fn parse_errors_on_bad_date() {
        let parser = CsvParser;
        let input = "date,title,amount\nnot-a-date,Coffee,4.50";
        let mapping = ColumnMapping {
            date_index: 0,
            title_index: 1,
            amount_index: 2,
            date_format: "%Y-%m-%d".to_string(),
        };
        assert!(parser.parse(input, &mapping).is_err());
    }

    #[test]
    fn parse_errors_on_bad_amount() {
        let parser = CsvParser;
        let input = "date,title,amount\n2025-01-01,Coffee,xyz";
        let mapping = ColumnMapping {
            date_index: 0,
            title_index: 1,
            amount_index: 2,
            date_format: "%Y-%m-%d".to_string(),
        };
        assert!(parser.parse(input, &mapping).is_err());
    }

    #[test]
    fn parse_errors_on_missing_column() {
        let parser = CsvParser;
        let input = "a,b\n1,2";
        let mapping = ColumnMapping {
            date_index: 0,
            title_index: 1,
            amount_index: 5, // out of range
            date_format: "%Y-%m-%d".to_string(),
        };
        assert!(parser.parse(input, &mapping).is_err());
    }

    #[test]
    fn parse_errors_on_not_enough_rows() {
        let parser = CsvParser;
        assert!(parser.parse("just,a,header", &ColumnMapping {
            date_index: 0,
            title_index: 1,
            amount_index: 2,
            date_format: "%Y-%m-%d".to_string(),
        }).is_err());
    }

    // ── Edge cases: parse_amount ──

    #[test]
    fn amount_signed_zero() {
        assert_eq!(parse_amount("-0").unwrap(), 0.0);
        assert_eq!(parse_amount("+0").unwrap(), 0.0);
        assert_eq!(parse_amount("-0.00").unwrap(), 0.0);
    }

    #[test]
    fn amount_multiple_signs_rejected() {
        assert!(parse_amount("--100").is_err());
        assert!(parse_amount("+-100").is_err());
    }

    #[test]
    fn amount_very_large_number() {
        let result = parse_amount("999999999999.99");
        assert!(result.is_ok());
        assert!((result.unwrap() - 999999999999.99).abs() < 0.01);
    }

    #[test]
    fn amount_only_currency_symbol() {
        assert!(parse_amount("$").is_err());
        assert!(parse_amount("€").is_err());
    }

    #[test]
    fn amount_only_separator() {
        // Just a comma — cleaned to empty after removing non-digit chars? No, comma stays.
        // "," → cleaned = "," → no digits around it
        // Only comma, 0 digits after → treated as thousands sep → removed → empty → parse error? No...
        // Let's just test it
        assert!(parse_amount(",").is_err());
        assert!(parse_amount(".").is_err());
    }

    // ── Edge cases: split_csv_line ──

    #[test]
    fn split_unmatched_quote_treated_as_quoted() {
        // Unmatched quote — keeps parsing in "quoted" mode until end
        let fields = CsvParser::split_csv_line("\"hello,world", ',');
        // in_quotes stays true, so comma is part of the field
        assert_eq!(fields.len(), 1);
        assert_eq!(fields[0], "hello,world");
    }

    #[test]
    fn split_nested_quotes() {
        let fields = CsvParser::split_csv_line("\"he said \"\"hi\"\"\",b", ',');
        // The parser toggles in_quotes for each '"', strips outer quotes
        assert_eq!(fields.len(), 2);
    }

    #[test]
    fn split_empty_fields() {
        let fields = CsvParser::split_csv_line(",,", ',');
        assert_eq!(fields.len(), 3);
        assert!(fields.iter().all(|f| f.is_empty()));
    }

    #[test]
    fn split_single_field_no_delimiter() {
        let fields = CsvParser::split_csv_line("hello", ',');
        assert_eq!(fields.len(), 1);
        assert_eq!(fields[0], "hello");
    }
}
