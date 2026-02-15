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
}

impl Default for ExportColumns {
    fn default() -> Self {
        Self {
            date: true,
            title: true,
            amount: true,
            category: true,
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
