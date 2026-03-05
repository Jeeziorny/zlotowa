use crate::db::{Database, DbError};
use chrono::NaiveDate;
use log::info;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum BackupError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Unsupported backup version: {0}")]
    UnsupportedVersion(u32),
    #[error("Database error: {0}")]
    Db(#[from] DbError),
    #[error("Invalid data: {0}")]
    InvalidData(String),
}

// ── Backup data structures ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupData {
    pub version: u32,
    pub app: String,
    pub exported_at: String,
    pub expenses: Vec<BackupExpense>,
    pub classification_rules: Vec<BackupClassificationRule>,
    /// Ignored — title cleanup rules were removed in v2. Kept for backwards compatibility with v1 backups.
    #[serde(default, skip_serializing)]
    pub title_cleanup_rules: Vec<serde_json::Value>,
    pub budgets: Vec<BackupBudget>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupExpense {
    pub title: String,
    pub amount: f64,
    pub date: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub classification_source: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupClassificationRule {
    pub pattern: String,
    pub category: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupBudget {
    pub start_date: String,
    pub end_date: String,
    pub categories: Vec<BackupBudgetCategory>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupBudgetCategory {
    pub category: String,
    pub amount: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupPreview {
    pub expense_count: usize,
    pub rule_count: usize,
    pub category_count: usize,
    pub budget_count: usize,
    pub created_at: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RestoreSummary {
    pub expenses_inserted: usize,
    pub expenses_skipped: usize,
    pub rules_upserted: usize,
    pub budgets_inserted: usize,
    pub budgets_skipped: usize,
}

const CURRENT_VERSION: u32 = 2;

// ── Backup ──

pub fn create_backup(db: &Database) -> Result<BackupData, BackupError> {
    let expenses = db.get_all_expenses()?;
    let rules = db.get_all_rules()?;
    let budgets = db.get_all_budgets()?;

    let mut backup_budgets = Vec::new();
    for budget in &budgets {
        let budget_id = budget.id.ok_or_else(|| {
            BackupError::InvalidData("Budget missing id".into())
        })?;
        let cats = db.get_budget_categories(budget_id)?;
        backup_budgets.push(BackupBudget {
            start_date: budget.start_date.to_string(),
            end_date: budget.end_date.to_string(),
            categories: cats
                .into_iter()
                .map(|c| BackupBudgetCategory {
                    category: c.category,
                    amount: c.amount,
                })
                .collect(),
        });
    }

    let backup = BackupData {
        version: CURRENT_VERSION,
        app: "zlotowa".to_string(),
        exported_at: chrono::Utc::now().to_rfc3339(),
        expenses: expenses
            .into_iter()
            .map(|e| BackupExpense {
                title: e.title,
                amount: e.amount,
                date: e.date.to_string(),
                category: e.category,
                classification_source: e.classification_source.map(|s| s.as_db_str().to_string()),
            })
            .collect(),
        classification_rules: rules
            .into_iter()
            .map(|r| BackupClassificationRule {
                pattern: r.pattern,
                category: r.category,
            })
            .collect(),
        title_cleanup_rules: Vec::new(),
        budgets: backup_budgets,
    };

    info!(
        "Backup created: {} expenses, {} rules, {} budgets",
        backup.expenses.len(),
        backup.classification_rules.len(),
        backup.budgets.len()
    );

    Ok(backup)
}

// ── Preview ──

pub fn preview_backup(data: &BackupData) -> Result<BackupPreview, BackupError> {
    if data.version > CURRENT_VERSION {
        return Err(BackupError::UnsupportedVersion(data.version));
    }

    let mut categories = std::collections::HashSet::new();
    for e in &data.expenses {
        if let Some(cat) = &e.category {
            categories.insert(cat.as_str());
        }
    }
    for r in &data.classification_rules {
        categories.insert(&r.category);
    }
    for b in &data.budgets {
        for bc in &b.categories {
            categories.insert(&bc.category);
        }
    }

    Ok(BackupPreview {
        expense_count: data.expenses.len(),
        rule_count: data.classification_rules.len(),
        category_count: categories.len(),
        budget_count: data.budgets.len(),
        created_at: Some(data.exported_at.clone()),
    })
}

// ── Restore ──

pub fn restore_backup(db: &Database, data: &BackupData) -> Result<RestoreSummary, BackupError> {
    if data.version > CURRENT_VERSION {
        return Err(BackupError::UnsupportedVersion(data.version));
    }

    // Validate all dates before touching the DB
    for (i, e) in data.expenses.iter().enumerate() {
        NaiveDate::parse_from_str(&e.date, "%Y-%m-%d").map_err(|_| {
            BackupError::InvalidData(format!(
                "Expense {}: invalid date '{}'",
                i, e.date
            ))
        })?;
        if !e.amount.is_finite() {
            return Err(BackupError::InvalidData(format!(
                "Expense {}: invalid amount '{}'",
                i, e.amount
            )));
        }
    }
    for (i, b) in data.budgets.iter().enumerate() {
        NaiveDate::parse_from_str(&b.start_date, "%Y-%m-%d").map_err(|_| {
            BackupError::InvalidData(format!(
                "Budget {}: invalid start_date '{}'",
                i, b.start_date
            ))
        })?;
        NaiveDate::parse_from_str(&b.end_date, "%Y-%m-%d").map_err(|_| {
            BackupError::InvalidData(format!(
                "Budget {}: invalid end_date '{}'",
                i, b.end_date
            ))
        })?;
    }

    let summary = db.restore_backup_data(data)?;

    info!(
        "Restore complete: {} expenses inserted ({} skipped), {} rules, {} budgets ({} skipped)",
        summary.expenses_inserted,
        summary.expenses_skipped,
        summary.rules_upserted,
        summary.budgets_inserted,
        summary.budgets_skipped,
    );

    Ok(summary)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Database;
    use crate::models::ClassificationRule;
    use chrono::NaiveDate;

    fn test_db() -> Database {
        Database::open_memory().unwrap()
    }

    fn sample_backup() -> BackupData {
        BackupData {
            version: 2,
            app: "zlotowa".to_string(),
            exported_at: "2026-02-27T12:00:00Z".to_string(),
            expenses: vec![
                BackupExpense {
                    title: "Coffee".to_string(),
                    amount: 4.50,
                    date: "2025-01-15".to_string(),
                    category: Some("Drinks".to_string()),
                    classification_source: Some("database".to_string()),
                },
                BackupExpense {
                    title: "Uber Trip".to_string(),
                    amount: 12.99,
                    date: "2025-01-16".to_string(),
                    category: Some("Transport".to_string()),
                    classification_source: Some("llm".to_string()),
                },
            ],
            classification_rules: vec![
                BackupClassificationRule {
                    pattern: "(?i)coffee".to_string(),
                    category: "Drinks".to_string(),
                },
                BackupClassificationRule {
                    pattern: "(?i)uber".to_string(),
                    category: "Transport".to_string(),
                },
            ],
            title_cleanup_rules: Vec::new(),
            budgets: vec![BackupBudget {
                start_date: "2025-01-01".to_string(),
                end_date: "2025-02-01".to_string(),
                categories: vec![
                    BackupBudgetCategory {
                        category: "Drinks".to_string(),
                        amount: 50.0,
                    },
                    BackupBudgetCategory {
                        category: "Transport".to_string(),
                        amount: 200.0,
                    },
                ],
            }],
        }
    }

    #[test]
    fn backup_roundtrip() {
        let db = test_db();
        let backup = sample_backup();

        // Restore into empty DB
        let summary = restore_backup(&db, &backup).unwrap();
        assert_eq!(summary.expenses_inserted, 2);
        assert_eq!(summary.expenses_skipped, 0);
        assert_eq!(summary.rules_upserted, 2);
        assert_eq!(summary.budgets_inserted, 1);
        assert_eq!(summary.budgets_skipped, 0);

        // Create a new backup from the restored data
        let backup2 = create_backup(&db).unwrap();
        assert_eq!(backup2.version, 2);
        assert_eq!(backup2.expenses.len(), 2);
        assert_eq!(backup2.classification_rules.len(), 2);
        assert_eq!(backup2.budgets.len(), 1);
        assert_eq!(backup2.budgets[0].categories.len(), 2);

        // Verify expense data survived roundtrip
        let uber = backup2.expenses.iter().find(|e| e.title == "Uber Trip").unwrap();
        assert_eq!(uber.category.as_deref(), Some("Transport"));
        assert_eq!(uber.classification_source.as_deref(), Some("llm"));
    }

    #[test]
    fn restore_skips_duplicate_expenses() {
        let db = test_db();
        let backup = sample_backup();

        // First restore
        let s1 = restore_backup(&db, &backup).unwrap();
        assert_eq!(s1.expenses_inserted, 2);
        assert_eq!(s1.expenses_skipped, 0);

        // Second restore — same expenses should be skipped
        let s2 = restore_backup(&db, &backup).unwrap();
        assert_eq!(s2.expenses_inserted, 0);
        assert_eq!(s2.expenses_skipped, 2);
    }

    #[test]
    fn restore_upserts_rules() {
        let db = test_db();

        // Insert a rule that conflicts with the backup
        db.insert_rule(&ClassificationRule {
            id: None,
            pattern: "(?i)coffee".to_string(),
            category: "Food".to_string(), // different category
        })
        .unwrap();

        let backup = sample_backup();
        let summary = restore_backup(&db, &backup).unwrap();
        assert_eq!(summary.rules_upserted, 2);

        // Verify the rule was overwritten
        let rules = db.get_all_rules().unwrap();
        let coffee_rule = rules.iter().find(|r| r.pattern == "(?i)coffee").unwrap();
        assert_eq!(coffee_rule.category, "Drinks"); // backup wins
    }

    #[test]
    fn restore_skips_overlapping_budgets() {
        let db = test_db();

        // Create an existing budget that overlaps
        db.create_budget(
            NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2025, 2, 1).unwrap(),
        )
        .unwrap();

        let backup = sample_backup();
        let summary = restore_backup(&db, &backup).unwrap();
        assert_eq!(summary.budgets_inserted, 0);
        assert_eq!(summary.budgets_skipped, 1);
    }

    #[test]
    fn restore_is_atomic() {
        let db = test_db();

        // Create a backup with one valid and one invalid expense
        let mut backup = sample_backup();
        backup.expenses.push(BackupExpense {
            title: "Bad".to_string(),
            amount: f64::INFINITY, // invalid
            date: "2025-01-17".to_string(),
            category: None,
            classification_source: None,
        });

        // Should fail validation before any DB writes
        let result = restore_backup(&db, &backup);
        assert!(result.is_err());

        // DB should be untouched
        let expenses = db.get_all_expenses().unwrap();
        assert!(expenses.is_empty());
    }

    #[test]
    fn restore_rejects_unsupported_version() {
        let db = test_db();
        let mut backup = sample_backup();
        backup.version = 999;

        let result = restore_backup(&db, &backup);
        assert!(result.is_err());
        match result.unwrap_err() {
            BackupError::UnsupportedVersion(999) => {}
            other => panic!("Expected UnsupportedVersion, got: {other}"),
        }
    }

    #[test]
    fn backup_empty_database() {
        let db = test_db();
        let backup = create_backup(&db).unwrap();

        assert_eq!(backup.version, 2);
        assert_eq!(backup.app, "zlotowa");
        assert!(backup.expenses.is_empty());
        assert!(backup.classification_rules.is_empty());
        assert!(backup.budgets.is_empty());

        // Should serialize to valid JSON
        let json = serde_json::to_string_pretty(&backup).unwrap();
        let parsed: BackupData = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.version, 2);
    }

    #[test]
    fn preview_backup_counts() {
        let backup = sample_backup();
        let preview = preview_backup(&backup).unwrap();
        assert_eq!(preview.expense_count, 2);
        assert_eq!(preview.rule_count, 2);
        assert_eq!(preview.category_count, 2); // Drinks + Transport
        assert_eq!(preview.budget_count, 1);
        assert_eq!(preview.created_at.as_deref(), Some("2026-02-27T12:00:00Z"));
    }

    #[test]
    fn preview_rejects_unsupported_version() {
        let mut backup = sample_backup();
        backup.version = 999;
        let result = preview_backup(&backup);
        assert!(result.is_err());
    }

    #[test]
    fn preview_deduplicates_categories() {
        let mut backup = sample_backup();
        // Add an expense with a category that already exists in rules
        backup.expenses.push(BackupExpense {
            title: "Latte".to_string(),
            amount: 5.0,
            date: "2025-01-17".to_string(),
            category: Some("Drinks".to_string()), // same as existing
            classification_source: None,
        });
        let preview = preview_backup(&backup).unwrap();
        assert_eq!(preview.category_count, 2); // still 2, not 3
    }

    #[test]
    fn restore_v1_backup_with_cleanup_rules() {
        let db = test_db();
        // Simulate a v1 backup JSON that includes title_cleanup_rules
        let json = r#"{
            "version": 1,
            "app": "4ccountant",
            "exported_at": "2026-01-01T00:00:00Z",
            "expenses": [],
            "classification_rules": [],
            "title_cleanup_rules": [
                {"pattern": "NOISE", "replacement": "", "is_regex": false}
            ],
            "budgets": []
        }"#;
        let backup: BackupData = serde_json::from_str(json).unwrap();
        let summary = restore_backup(&db, &backup).unwrap();
        assert_eq!(summary.expenses_inserted, 0);
        assert_eq!(summary.rules_upserted, 0);
        assert_eq!(summary.budgets_inserted, 0);
    }
}
