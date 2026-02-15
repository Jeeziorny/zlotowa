use crate::models::{ClassificationRule, ClassificationSource, Expense};
use rusqlite::{params, Connection, Result as SqlResult};
use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DbError {
    #[error("Database error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("Failed to determine data directory")]
    NoDataDir,
    #[error("Invalid data: {0}")]
    InvalidData(String),
}

pub struct Database {
    conn: Connection,
}

impl Database {
    /// Open (or create) the database at the default location.
    pub fn open_default() -> Result<Self, DbError> {
        let path = Self::default_path()?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).ok();
        }
        Self::open(&path)
    }

    /// Open (or create) the database at a specific path.
    pub fn open(path: &std::path::Path) -> Result<Self, DbError> {
        let conn = Connection::open(path)?;
        let db = Self { conn };
        db.migrate()?;
        Ok(db)
    }

    /// Open an in-memory database (for testing).
    pub fn open_memory() -> Result<Self, DbError> {
        let conn = Connection::open_in_memory()?;
        let db = Self { conn };
        db.migrate()?;
        Ok(db)
    }

    fn default_path() -> Result<PathBuf, DbError> {
        let data_dir = dirs::data_dir().ok_or(DbError::NoDataDir)?;
        Ok(data_dir.join("4ccountant").join("4ccountant.db"))
    }

    fn migrate(&self) -> Result<(), DbError> {
        self.conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS expenses (
                id              INTEGER PRIMARY KEY AUTOINCREMENT,
                title           TEXT NOT NULL,
                amount          REAL NOT NULL,
                date            TEXT NOT NULL,
                category        TEXT,
                classification_source TEXT
            );

            CREATE INDEX IF NOT EXISTS idx_expenses_date ON expenses(date);
            CREATE INDEX IF NOT EXISTS idx_expenses_dup ON expenses(title, amount, date);

            CREATE TABLE IF NOT EXISTS classification_rules (
                id       INTEGER PRIMARY KEY AUTOINCREMENT,
                pattern  TEXT NOT NULL UNIQUE,
                category TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS config (
                key   TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );
            ",
        )?;
        Ok(())
    }

    // ── Expenses ──

    pub fn insert_expense(&self, expense: &Expense) -> Result<i64, DbError> {
        if !expense.amount.is_finite() {
            return Err(DbError::InvalidData(format!(
                "Amount is not a valid number: {}",
                expense.amount
            )));
        }
        self.conn.execute(
            "INSERT INTO expenses (title, amount, date, category, classification_source)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                expense.title,
                expense.amount,
                expense.date.to_string(),
                expense.category,
                expense.classification_source.as_ref().map(|s| s.as_db_str()),
            ],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    /// Insert multiple expenses atomically. Either all succeed or none are saved.
    pub fn insert_expenses_bulk(&self, expenses: &[Expense]) -> Result<usize, DbError> {
        let tx = self.conn.unchecked_transaction()?;
        let mut count = 0;

        for expense in expenses {
            if !expense.amount.is_finite() {
                return Err(DbError::InvalidData(format!(
                    "Amount is not a valid number for '{}': {}",
                    expense.title, expense.amount
                )));
            }
            tx.execute(
                "INSERT INTO expenses (title, amount, date, category, classification_source)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![
                    expense.title,
                    expense.amount,
                    expense.date.to_string(),
                    expense.category,
                    expense.classification_source.as_ref().map(|s| s.as_db_str()),
                ],
            )?;
            count += 1;
        }

        tx.commit()?;
        Ok(count)
    }

    pub fn get_all_expenses(&self) -> Result<Vec<Expense>, DbError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, title, amount, date, category, classification_source FROM expenses ORDER BY date DESC",
        )?;
        let rows = stmt.query_map([], |row| {
            let source_str: Option<String> = row.get(5)?;
            let source = source_str
                .as_deref()
                .and_then(ClassificationSource::from_str_opt);
            let date_str: String = row.get(3)?;
            let date = chrono::NaiveDate::parse_from_str(&date_str, "%Y-%m-%d")
                .map_err(|e| {
                    rusqlite::Error::FromSqlConversionFailure(
                        3,
                        rusqlite::types::Type::Text,
                        Box::new(e),
                    )
                })?;
            Ok(Expense {
                id: Some(row.get(0)?),
                title: row.get(1)?,
                amount: row.get(2)?,
                date,
                category: row.get(4)?,
                classification_source: source,
            })
        })?;
        rows.collect::<SqlResult<Vec<_>>>().map_err(DbError::from)
    }

    /// Check if an expense is a duplicate (same title, amount, and date).
    pub fn is_duplicate(&self, title: &str, amount: f64, date: &chrono::NaiveDate) -> Result<bool, DbError> {
        let count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM expenses WHERE title = ?1 AND amount = ?2 AND date = ?3",
            params![title, amount, date.to_string()],
            |row| row.get(0),
        )?;
        Ok(count > 0)
    }

    // ── Classification Rules ──

    pub fn insert_rule(&self, rule: &ClassificationRule) -> Result<i64, DbError> {
        self.conn.execute(
            "INSERT OR REPLACE INTO classification_rules (pattern, category) VALUES (?1, ?2)",
            params![rule.pattern, rule.category],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    /// Insert multiple classification rules atomically.
    pub fn insert_rules_bulk(&self, rules: &[ClassificationRule]) -> Result<usize, DbError> {
        let tx = self.conn.unchecked_transaction()?;
        let mut count = 0;

        for rule in rules {
            tx.execute(
                "INSERT OR REPLACE INTO classification_rules (pattern, category) VALUES (?1, ?2)",
                params![rule.pattern, rule.category],
            )?;
            count += 1;
        }

        tx.commit()?;
        Ok(count)
    }

    pub fn get_all_rules(&self) -> Result<Vec<ClassificationRule>, DbError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, pattern, category FROM classification_rules ORDER BY id",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(ClassificationRule {
                id: Some(row.get(0)?),
                pattern: row.get(1)?,
                category: row.get(2)?,
            })
        })?;
        rows.collect::<SqlResult<Vec<_>>>().map_err(DbError::from)
    }

    pub fn get_all_categories(&self) -> Result<Vec<String>, DbError> {
        let mut stmt = self.conn.prepare(
            "SELECT DISTINCT category FROM classification_rules ORDER BY category",
        )?;
        let rows = stmt.query_map([], |row| row.get(0))?;
        rows.collect::<SqlResult<Vec<_>>>().map_err(DbError::from)
    }

    // ── Config ──

    pub fn get_config(&self, key: &str) -> Result<Option<String>, DbError> {
        let result = self.conn.query_row(
            "SELECT value FROM config WHERE key = ?1",
            params![key],
            |row| row.get(0),
        );
        match result {
            Ok(val) => Ok(Some(val)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(DbError::from(e)),
        }
    }

    pub fn set_config(&self, key: &str, value: &str) -> Result<(), DbError> {
        self.conn.execute(
            "INSERT OR REPLACE INTO config (key, value) VALUES (?1, ?2)",
            params![key, value],
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{ClassificationSource, Expense};
    use chrono::NaiveDate;

    fn test_db() -> Database {
        Database::open_memory().expect("Failed to create in-memory DB")
    }

    fn make_expense(title: &str, amount: f64, date: &str) -> Expense {
        Expense {
            id: None,
            title: title.to_string(),
            amount,
            date: NaiveDate::parse_from_str(date, "%Y-%m-%d").unwrap(),
            category: None,
            classification_source: None,
        }
    }

    // ── Migration & setup ──

    #[test]
    fn open_memory_creates_tables() {
        let db = test_db();
        // Should be able to query all tables without error
        db.get_all_expenses().unwrap();
        db.get_all_rules().unwrap();
        db.get_all_categories().unwrap();
    }

    // ── Single expense CRUD ──

    #[test]
    fn insert_and_retrieve_expense() {
        let db = test_db();
        let expense = make_expense("Coffee", 4.50, "2025-01-15");
        let id = db.insert_expense(&expense).unwrap();
        assert!(id > 0);

        let all = db.get_all_expenses().unwrap();
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].title, "Coffee");
        assert_eq!(all[0].amount, 4.50);
        assert_eq!(all[0].date.to_string(), "2025-01-15");
    }

    #[test]
    fn insert_expense_with_category_and_source() {
        let db = test_db();
        let expense = Expense {
            id: None,
            title: "Groceries".to_string(),
            amount: 52.30,
            date: NaiveDate::from_ymd_opt(2025, 3, 10).unwrap(),
            category: Some("Food".to_string()),
            classification_source: Some(ClassificationSource::Database),
        };
        db.insert_expense(&expense).unwrap();

        let all = db.get_all_expenses().unwrap();
        assert_eq!(all[0].category.as_deref(), Some("Food"));
        assert_eq!(all[0].classification_source, Some(ClassificationSource::Database));
    }

    #[test]
    fn insert_expense_rejects_nan() {
        let db = test_db();
        let expense = make_expense("Bad", f64::NAN, "2025-01-01");
        let result = db.insert_expense(&expense);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not a valid number"));
    }

    #[test]
    fn insert_expense_rejects_infinity() {
        let db = test_db();
        let expense = make_expense("Bad", f64::INFINITY, "2025-01-01");
        assert!(db.insert_expense(&expense).is_err());

        let expense = make_expense("Bad", f64::NEG_INFINITY, "2025-01-01");
        assert!(db.insert_expense(&expense).is_err());
    }

    #[test]
    fn expenses_ordered_by_date_desc() {
        let db = test_db();
        db.insert_expense(&make_expense("Old", 10.0, "2024-01-01")).unwrap();
        db.insert_expense(&make_expense("New", 20.0, "2025-06-15")).unwrap();
        db.insert_expense(&make_expense("Mid", 15.0, "2024-07-01")).unwrap();

        let all = db.get_all_expenses().unwrap();
        assert_eq!(all[0].title, "New");
        assert_eq!(all[1].title, "Mid");
        assert_eq!(all[2].title, "Old");
    }

    // ── Duplicate detection ──

    #[test]
    fn is_duplicate_returns_false_when_empty() {
        let db = test_db();
        let date = NaiveDate::from_ymd_opt(2025, 1, 1).unwrap();
        assert!(!db.is_duplicate("Coffee", 4.50, &date).unwrap());
    }

    #[test]
    fn is_duplicate_returns_true_for_exact_match() {
        let db = test_db();
        db.insert_expense(&make_expense("Coffee", 4.50, "2025-01-15")).unwrap();
        let date = NaiveDate::from_ymd_opt(2025, 1, 15).unwrap();
        assert!(db.is_duplicate("Coffee", 4.50, &date).unwrap());
    }

    #[test]
    fn is_duplicate_returns_false_for_different_fields() {
        let db = test_db();
        db.insert_expense(&make_expense("Coffee", 4.50, "2025-01-15")).unwrap();

        let date = NaiveDate::from_ymd_opt(2025, 1, 15).unwrap();
        // Different title
        assert!(!db.is_duplicate("Tea", 4.50, &date).unwrap());
        // Different amount
        assert!(!db.is_duplicate("Coffee", 5.00, &date).unwrap());
        // Different date
        let other_date = NaiveDate::from_ymd_opt(2025, 1, 16).unwrap();
        assert!(!db.is_duplicate("Coffee", 4.50, &other_date).unwrap());
    }

    // ── Bulk insert ──

    #[test]
    fn bulk_insert_expenses() {
        let db = test_db();
        let expenses = vec![
            make_expense("A", 1.0, "2025-01-01"),
            make_expense("B", 2.0, "2025-01-02"),
            make_expense("C", 3.0, "2025-01-03"),
        ];
        let count = db.insert_expenses_bulk(&expenses).unwrap();
        assert_eq!(count, 3);
        assert_eq!(db.get_all_expenses().unwrap().len(), 3);
    }

    #[test]
    fn bulk_insert_rolls_back_on_nan() {
        let db = test_db();
        let expenses = vec![
            make_expense("Good", 10.0, "2025-01-01"),
            make_expense("Bad", f64::NAN, "2025-01-02"),
        ];
        assert!(db.insert_expenses_bulk(&expenses).is_err());
        // Transaction rolled back — nothing inserted
        assert_eq!(db.get_all_expenses().unwrap().len(), 0);
    }

    // ── Classification rules ──

    #[test]
    fn insert_and_retrieve_rules() {
        let db = test_db();
        let rule = ClassificationRule {
            id: None,
            pattern: "(?i)grocery".to_string(),
            category: "Food".to_string(),
        };
        db.insert_rule(&rule).unwrap();

        let rules = db.get_all_rules().unwrap();
        assert_eq!(rules.len(), 1);
        assert_eq!(rules[0].pattern, "(?i)grocery");
        assert_eq!(rules[0].category, "Food");
    }

    #[test]
    fn insert_rule_replaces_on_duplicate_pattern() {
        let db = test_db();
        let rule1 = ClassificationRule { id: None, pattern: "coffee".to_string(), category: "Drinks".to_string() };
        let rule2 = ClassificationRule { id: None, pattern: "coffee".to_string(), category: "Cafe".to_string() };
        db.insert_rule(&rule1).unwrap();
        db.insert_rule(&rule2).unwrap();

        let rules = db.get_all_rules().unwrap();
        assert_eq!(rules.len(), 1);
        assert_eq!(rules[0].category, "Cafe");
    }

    #[test]
    fn bulk_insert_rules() {
        let db = test_db();
        let rules = vec![
            ClassificationRule { id: None, pattern: "a".to_string(), category: "A".to_string() },
            ClassificationRule { id: None, pattern: "b".to_string(), category: "B".to_string() },
        ];
        let count = db.insert_rules_bulk(&rules).unwrap();
        assert_eq!(count, 2);
        assert_eq!(db.get_all_rules().unwrap().len(), 2);
    }

    #[test]
    fn get_all_categories() {
        let db = test_db();
        let rules = vec![
            ClassificationRule { id: None, pattern: "a".to_string(), category: "Food".to_string() },
            ClassificationRule { id: None, pattern: "b".to_string(), category: "Transport".to_string() },
            ClassificationRule { id: None, pattern: "c".to_string(), category: "Food".to_string() },
        ];
        db.insert_rules_bulk(&rules).unwrap();

        let categories = db.get_all_categories().unwrap();
        assert_eq!(categories, vec!["Food", "Transport"]);
    }

    // ── Config ──

    #[test]
    fn config_get_missing_key_returns_none() {
        let db = test_db();
        assert_eq!(db.get_config("nonexistent").unwrap(), None);
    }

    #[test]
    fn config_set_and_get() {
        let db = test_db();
        db.set_config("llm_provider", "openai").unwrap();
        assert_eq!(db.get_config("llm_provider").unwrap(), Some("openai".to_string()));
    }

    #[test]
    fn config_set_overwrites() {
        let db = test_db();
        db.set_config("key", "old").unwrap();
        db.set_config("key", "new").unwrap();
        assert_eq!(db.get_config("key").unwrap(), Some("new".to_string()));
    }
}
