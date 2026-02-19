use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Sentinel value for filtering expenses with no category.
pub const UNCATEGORIZED: &str = "uncategorized";

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

impl fmt::Display for ClassificationSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Database => write!(f, "Database"),
            Self::Llm => write!(f, "Llm"),
            Self::Manual => write!(f, "Manual"),
        }
    }
}

impl ClassificationSource {
    pub fn from_str_opt(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "database" => Some(Self::Database),
            "llm" => Some(Self::Llm),
            "manual" => Some(Self::Manual),
            _ => None,
        }
    }

    pub fn as_db_str(&self) -> &'static str {
        match self {
            Self::Database => "database",
            Self::Llm => "llm",
            Self::Manual => "manual",
        }
    }
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

impl ClassificationRule {
    /// Create a case-insensitive regex rule from a pattern string.
    pub fn from_pattern(pattern_source: &str, category: &str) -> Self {
        let escaped = regex::escape(pattern_source);
        Self {
            id: None,
            pattern: format!("(?i){}", escaped),
            category: category.to_string(),
        }
    }
}

/// Category with usage statistics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryStats {
    pub name: String,
    pub expense_count: i64,
    pub rule_count: i64,
}

/// Metadata for a bulk upload batch (used for undo/revert).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadBatch {
    pub id: i64,
    pub filename: Option<String>,
    pub uploaded_at: String,
    pub expense_count: i64,
}

/// A find/replace rule for cleaning up noisy bank transaction titles.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TitleCleanupRule {
    pub id: Option<i64>,
    pub pattern: String,
    pub replacement: String,
    pub is_regex: bool,
}

// ── Budget Models ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Budget {
    pub id: Option<i64>,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetCategory {
    pub id: Option<i64>,
    pub budget_id: i64,
    pub category: String,
    pub amount: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlannedExpense {
    pub id: Option<i64>,
    pub budget_id: i64,
    pub title: String,
    pub amount: f64,
    pub date: NaiveDate,
    pub category: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalendarEvent {
    pub id: Option<i64>,
    pub budget_id: i64,
    pub summary: String,
    pub description: Option<String>,
    pub location: Option<String>,
    pub start_date: NaiveDate,
    pub end_date: Option<NaiveDate>,
    pub all_day: bool,
    pub amount: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum BudgetStatus {
    #[serde(rename = "over")]
    Over,
    #[serde(rename = "approaching")]
    Approaching,
    #[serde(rename = "under")]
    Under,
}

impl fmt::Display for BudgetStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Over => write!(f, "over"),
            Self::Approaching => write!(f, "approaching"),
            Self::Under => write!(f, "under"),
        }
    }
}

impl BudgetStatus {
    pub fn from_ratio(ratio: f64) -> Self {
        if ratio > 1.0 {
            Self::Over
        } else if ratio >= 0.8 {
            Self::Approaching
        } else {
            Self::Under
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetCategoryStatus {
    pub category: String,
    pub budgeted: f64,
    pub spent: f64,
    pub status: BudgetStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryAverage {
    pub category: String,
    pub average: f64,
    pub months_with_data: u32,
}

// ── Expense Query ──

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ExpenseQuery {
    pub search: Option<String>,
    pub category: Option<String>,
    pub date_from: Option<NaiveDate>,
    pub date_to: Option<NaiveDate>,
    pub amount_min: Option<f64>,
    pub amount_max: Option<f64>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpenseQueryResult {
    pub expenses: Vec<Expense>,
    pub total_count: i64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classification_source_display() {
        assert_eq!(ClassificationSource::Database.to_string(), "Database");
        assert_eq!(ClassificationSource::Llm.to_string(), "Llm");
        assert_eq!(ClassificationSource::Manual.to_string(), "Manual");
    }

    #[test]
    fn classification_source_from_str_opt_valid() {
        assert_eq!(ClassificationSource::from_str_opt("database"), Some(ClassificationSource::Database));
        assert_eq!(ClassificationSource::from_str_opt("Database"), Some(ClassificationSource::Database));
        assert_eq!(ClassificationSource::from_str_opt("llm"), Some(ClassificationSource::Llm));
        assert_eq!(ClassificationSource::from_str_opt("Llm"), Some(ClassificationSource::Llm));
        assert_eq!(ClassificationSource::from_str_opt("manual"), Some(ClassificationSource::Manual));
        assert_eq!(ClassificationSource::from_str_opt("Manual"), Some(ClassificationSource::Manual));
    }

    #[test]
    fn classification_source_from_str_opt_invalid() {
        assert_eq!(ClassificationSource::from_str_opt(""), None);
        assert_eq!(ClassificationSource::from_str_opt("unknown"), None);
    }

    #[test]
    fn classification_source_from_str_opt_case_insensitive() {
        assert_eq!(ClassificationSource::from_str_opt("DATABASE"), Some(ClassificationSource::Database));
        assert_eq!(ClassificationSource::from_str_opt("LLM"), Some(ClassificationSource::Llm));
        assert_eq!(ClassificationSource::from_str_opt("MANUAL"), Some(ClassificationSource::Manual));
    }

    #[test]
    fn classification_source_as_db_str() {
        assert_eq!(ClassificationSource::Database.as_db_str(), "database");
        assert_eq!(ClassificationSource::Llm.as_db_str(), "llm");
        assert_eq!(ClassificationSource::Manual.as_db_str(), "manual");
    }

    #[test]
    fn classification_source_roundtrip() {
        for source in [ClassificationSource::Database, ClassificationSource::Llm, ClassificationSource::Manual] {
            let db_str = source.as_db_str();
            let restored = ClassificationSource::from_str_opt(db_str).unwrap();
            assert_eq!(source, restored);
        }
    }
}
