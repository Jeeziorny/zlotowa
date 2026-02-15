pub mod csv_parser;

use crate::models::ParsedExpense;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("Input format not recognized")]
    UnrecognizedFormat,
    #[error("Failed to parse input: {0}")]
    ParseFailed(String),
    #[error("Too few expenses parsed (got {got}, minimum {min})")]
    TooFewExpenses { got: usize, min: usize },
}

/// Column mapping that the user confirms after previewing parsed data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnMapping {
    pub title_index: usize,
    pub amount_index: usize,
    pub date_index: usize,
    pub date_format: String,
}

/// A parser that can handle a specific bank's data format.
pub trait Parser: Send + Sync {
    /// Human-readable name of this parser (e.g. "CSV").
    fn name(&self) -> &str;

    /// Try to detect if this parser can handle the given input.
    /// Returns a confidence score between 0.0 and 1.0.
    fn detect(&self, input: &str) -> f64;

    /// Parse the input into expenses using a confirmed column mapping.
    fn parse(&self, input: &str, mapping: &ColumnMapping) -> Result<Vec<ParsedExpense>, ParseError>;

    /// Preview: return raw rows so the user can confirm column mapping.
    fn preview_rows(&self, input: &str) -> Result<Vec<Vec<String>>, ParseError>;
}

/// Find the best matching parser for the given input.
pub fn detect_parser<'a>(input: &str, parsers: &'a [Box<dyn Parser>]) -> Option<&'a dyn Parser> {
    parsers
        .iter()
        .map(|p| (p.as_ref(), p.detect(input)))
        .filter(|(_, score)| *score > 0.3)
        .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
        .map(|(p, _)| p)
}

/// Get all built-in parsers.
pub fn builtin_parsers() -> Vec<Box<dyn Parser>> {
    vec![Box::new(csv_parser::CsvParser)]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detect_parser_finds_csv() {
        let parsers = builtin_parsers();
        let input = "date,title,amount\n2025-01-01,Coffee,4.50\n2025-01-02,Lunch,12.00";
        let parser = detect_parser(input, &parsers);
        assert!(parser.is_some());
        assert_eq!(parser.unwrap().name(), "CSV");
    }

    #[test]
    fn detect_parser_returns_none_for_garbage() {
        let parsers = builtin_parsers();
        let parser = detect_parser("not csv at all", &parsers);
        assert!(parser.is_none());
    }

    #[test]
    fn builtin_parsers_contains_csv() {
        let parsers = builtin_parsers();
        assert_eq!(parsers.len(), 1);
        assert_eq!(parsers[0].name(), "CSV");
    }
}
