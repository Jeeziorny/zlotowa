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
        self.conn.execute(
            "INSERT INTO expenses (title, amount, date, category, classification_source)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                expense.title,
                expense.amount,
                expense.date.to_string(),
                expense.category,
                expense.classification_source.as_ref().map(|s| match s {
                    ClassificationSource::Database => "database",
                    ClassificationSource::Llm => "llm",
                    ClassificationSource::Manual => "manual",
                }),
            ],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn get_all_expenses(&self) -> Result<Vec<Expense>, DbError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, title, amount, date, category, classification_source FROM expenses ORDER BY date DESC",
        )?;
        let rows = stmt.query_map([], |row| {
            let source_str: Option<String> = row.get(5)?;
            let source = source_str.map(|s| match s.as_str() {
                "database" => ClassificationSource::Database,
                "llm" => ClassificationSource::Llm,
                _ => ClassificationSource::Manual,
            });
            let date_str: String = row.get(3)?;
            Ok(Expense {
                id: Some(row.get(0)?),
                title: row.get(1)?,
                amount: row.get(2)?,
                date: chrono::NaiveDate::parse_from_str(&date_str, "%Y-%m-%d")
                    .unwrap_or_default(),
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
