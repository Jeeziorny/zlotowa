use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use std::fmt;

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
        match s {
            "database" | "Database" => Some(Self::Database),
            "llm" | "Llm" => Some(Self::Llm),
            "manual" | "Manual" => Some(Self::Manual),
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

/// Result of a bulk classification attempt.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassifiedExpense {
    pub parsed: ParsedExpense,
    pub category: Option<String>,
    pub source: Option<ClassificationSource>,
    pub is_duplicate: bool,
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
        assert_eq!(ClassificationSource::from_str_opt("DATABASE"), None);
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
