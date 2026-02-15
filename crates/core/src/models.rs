use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Expense {
    pub id: Option<i64>,
    pub title: String,
    pub amount: f64,
    pub date: NaiveDate,
    pub category: Option<String>,
    /// How this expense was classified
    pub classification_source: Option<ClassificationSource>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ClassificationSource {
    /// Matched by regex rule in the classification database
    Database,
    /// Classified by LLM
    Llm,
    /// Manually assigned by user
    Manual,
}

/// A row parsed from bank data before classification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedExpense {
    pub title: String,
    pub amount: f64,
    pub date: NaiveDate,
}

/// A regex-to-category mapping stored in the classification database.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassificationRule {
    pub id: Option<i64>,
    pub pattern: String,
    pub category: String,
}

/// Result of a bulk classification attempt.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassifiedExpense {
    pub parsed: ParsedExpense,
    pub category: Option<String>,
    pub source: Option<ClassificationSource>,
    pub is_duplicate: bool,
}
