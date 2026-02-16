use crate::models::{
    Budget, BudgetCategory, CalendarEvent, CategoryAverage, CategoryStats, ClassificationRule,
    ClassificationSource, Expense, PlannedExpense, TitleCleanupRule, UploadBatch,
};
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
            std::fs::create_dir_all(parent).map_err(|e| {
                DbError::InvalidData(format!(
                    "Cannot create data directory {}: {}",
                    parent.display(),
                    e
                ))
            })?;
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
            CREATE INDEX IF NOT EXISTS idx_expenses_category ON expenses(category);
            CREATE INDEX IF NOT EXISTS idx_expenses_source ON expenses(classification_source);

            CREATE TABLE IF NOT EXISTS classification_rules (
                id       INTEGER PRIMARY KEY AUTOINCREMENT,
                pattern  TEXT NOT NULL UNIQUE,
                category TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS config (
                key   TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS title_cleanup_rules (
                id          INTEGER PRIMARY KEY AUTOINCREMENT,
                pattern     TEXT NOT NULL,
                replacement TEXT NOT NULL DEFAULT '',
                is_regex    INTEGER NOT NULL DEFAULT 0
            );

            CREATE TABLE IF NOT EXISTS budgets (
                id    INTEGER PRIMARY KEY AUTOINCREMENT,
                year  INTEGER NOT NULL,
                month INTEGER NOT NULL,
                UNIQUE(year, month)
            );

            CREATE TABLE IF NOT EXISTS budget_categories (
                id        INTEGER PRIMARY KEY AUTOINCREMENT,
                budget_id INTEGER NOT NULL REFERENCES budgets(id) ON DELETE CASCADE,
                category  TEXT NOT NULL,
                amount    REAL NOT NULL,
                UNIQUE(budget_id, category)
            );

            CREATE TABLE IF NOT EXISTS planned_expenses (
                id        INTEGER PRIMARY KEY AUTOINCREMENT,
                budget_id INTEGER NOT NULL REFERENCES budgets(id) ON DELETE CASCADE,
                title     TEXT NOT NULL,
                amount    REAL NOT NULL,
                date      TEXT NOT NULL,
                category  TEXT
            );

            CREATE TABLE IF NOT EXISTS calendar_events (
                id          INTEGER PRIMARY KEY AUTOINCREMENT,
                budget_id   INTEGER NOT NULL REFERENCES budgets(id) ON DELETE CASCADE,
                summary     TEXT NOT NULL,
                description TEXT,
                location    TEXT,
                start_date  TEXT NOT NULL,
                end_date    TEXT,
                all_day     INTEGER NOT NULL DEFAULT 0
            );

            PRAGMA foreign_keys = ON;
            ",
        )?;

        // Batch tracking migration (idempotent)
        self.conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS upload_batches (
                id             INTEGER PRIMARY KEY AUTOINCREMENT,
                filename       TEXT,
                uploaded_at    TEXT NOT NULL,
                expense_count  INTEGER NOT NULL
            );",
        )?;
        // ALTER TABLE is not idempotent in SQLite — ignore error if column already exists
        let _ = self.conn.execute_batch(
            "ALTER TABLE expenses ADD COLUMN batch_id INTEGER REFERENCES upload_batches(id);",
        );
        self.conn.execute_batch(
            "CREATE INDEX IF NOT EXISTS idx_expenses_batch_id ON expenses(batch_id);",
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
    /// When `batch_filename` is `Some`, creates an upload batch record and links expenses to it.
    pub fn insert_expenses_bulk(
        &self,
        expenses: &[Expense],
        batch_filename: Option<&str>,
    ) -> Result<usize, DbError> {
        let tx = self.conn.unchecked_transaction()?;

        // Create batch record if filename provided
        let batch_id: Option<i64> = if let Some(filename) = batch_filename {
            let now = chrono::Utc::now().to_rfc3339();
            tx.execute(
                "INSERT INTO upload_batches (filename, uploaded_at, expense_count) VALUES (?1, ?2, ?3)",
                params![filename, now, expenses.len() as i64],
            )?;
            Some(tx.last_insert_rowid())
        } else {
            None
        };

        let mut count = 0;
        for expense in expenses {
            if !expense.amount.is_finite() {
                return Err(DbError::InvalidData(format!(
                    "Amount is not a valid number for '{}': {}",
                    expense.title, expense.amount
                )));
            }
            tx.execute(
                "INSERT INTO expenses (title, amount, date, category, classification_source, batch_id)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![
                    expense.title,
                    expense.amount,
                    expense.date.to_string(),
                    expense.category,
                    expense.classification_source.as_ref().map(|s| s.as_db_str()),
                    batch_id,
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

    /// Check which (title, amount, date) tuples already exist in the database.
    /// Returns a Vec<bool> aligned with the input slice — true means duplicate.
    pub fn check_duplicates_batch(
        &self,
        expenses: &[(&str, f64, &chrono::NaiveDate)],
    ) -> Result<Vec<bool>, DbError> {
        if expenses.is_empty() {
            return Ok(vec![]);
        }

        // For small batches, use OR-chained conditions in a single query
        let mut conditions = Vec::new();
        let mut param_values: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

        for (i, (title, amount, date)) in expenses.iter().enumerate() {
            let base = i * 3;
            conditions.push(format!(
                "(title = ?{} AND amount = ?{} AND date = ?{})",
                base + 1,
                base + 2,
                base + 3
            ));
            param_values.push(Box::new(title.to_string()));
            param_values.push(Box::new(*amount));
            param_values.push(Box::new(date.to_string()));
        }

        let sql = format!(
            "SELECT title, amount, date FROM expenses WHERE {}",
            conditions.join(" OR ")
        );

        let params_refs: Vec<&dyn rusqlite::types::ToSql> = param_values.iter().map(|p| p.as_ref()).collect();
        let mut stmt = self.conn.prepare(&sql)?;
        let found_rows = stmt.query_map(params_refs.as_slice(), |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, f64>(1)?,
                row.get::<_, String>(2)?,
            ))
        })?;

        let found: Vec<(String, f64, String)> = found_rows
            .collect::<SqlResult<Vec<_>>>()
            .map_err(DbError::from)?;

        let results = expenses
            .iter()
            .map(|(title, amount, date)| {
                let date_str = date.to_string();
                found.iter().any(|(t, a, d)| t == title && *a == *amount && *d == date_str)
            })
            .collect();

        Ok(results)
    }

    pub fn update_expense(&self, expense: &Expense) -> Result<(), DbError> {
        let id = expense
            .id
            .ok_or(DbError::InvalidData("Cannot update expense without id".into()))?;
        if !expense.amount.is_finite() {
            return Err(DbError::InvalidData(format!(
                "Amount is not a valid number: {}",
                expense.amount
            )));
        }
        let rows = self.conn.execute(
            "UPDATE expenses SET title = ?1, amount = ?2, date = ?3, category = ?4, classification_source = ?5 WHERE id = ?6",
            params![
                expense.title,
                expense.amount,
                expense.date.to_string(),
                expense.category,
                expense.classification_source.as_ref().map(|s| s.as_db_str()),
                id,
            ],
        )?;
        if rows == 0 {
            return Err(DbError::InvalidData(format!(
                "Expense with id {} not found",
                id
            )));
        }
        Ok(())
    }

    pub fn delete_expense(&self, id: i64) -> Result<(), DbError> {
        let rows = self
            .conn
            .execute("DELETE FROM expenses WHERE id = ?1", params![id])?;
        if rows == 0 {
            return Err(DbError::InvalidData(format!(
                "Expense with id {} not found",
                id
            )));
        }
        Ok(())
    }

    pub fn delete_expenses(&self, ids: &[i64]) -> Result<usize, DbError> {
        if ids.is_empty() {
            return Ok(0);
        }
        let tx = self.conn.unchecked_transaction()?;
        let mut count = 0;
        for id in ids {
            count += tx.execute("DELETE FROM expenses WHERE id = ?1", params![id])?;
        }
        tx.commit()?;
        Ok(count)
    }

    // ── Upload Batches ──

    pub fn get_upload_batches(&self) -> Result<Vec<UploadBatch>, DbError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, filename, uploaded_at, expense_count FROM upload_batches ORDER BY uploaded_at DESC",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(UploadBatch {
                id: row.get(0)?,
                filename: row.get(1)?,
                uploaded_at: row.get(2)?,
                expense_count: row.get(3)?,
            })
        })?;
        rows.collect::<SqlResult<Vec<_>>>().map_err(DbError::from)
    }

    /// Delete a batch and all its expenses. Returns the number of deleted expenses.
    pub fn delete_batch(&self, batch_id: i64) -> Result<usize, DbError> {
        let tx = self.conn.unchecked_transaction()?;
        let deleted = tx.execute(
            "DELETE FROM expenses WHERE batch_id = ?1",
            params![batch_id],
        )?;
        tx.execute(
            "DELETE FROM upload_batches WHERE id = ?1",
            params![batch_id],
        )?;
        tx.commit()?;
        Ok(deleted)
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

    // ── Category Management ──

    /// Returns stats for all known categories (union of expenses and rules tables).
    pub fn get_category_stats(&self) -> Result<Vec<CategoryStats>, DbError> {
        let mut stmt = self.conn.prepare(
            "SELECT name,
                    COALESCE(ec, 0) AS expense_count,
                    COALESCE(rc, 0) AS rule_count
             FROM (
                 SELECT category AS name FROM expenses WHERE category IS NOT NULL AND category != ''
                 UNION
                 SELECT category AS name FROM classification_rules
             ) cats
             LEFT JOIN (SELECT category, COUNT(*) AS ec FROM expenses WHERE category IS NOT NULL GROUP BY category) e ON e.category = cats.name
             LEFT JOIN (SELECT category, COUNT(*) AS rc FROM classification_rules GROUP BY category) r ON r.category = cats.name
             ORDER BY name",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(CategoryStats {
                name: row.get(0)?,
                expense_count: row.get(1)?,
                rule_count: row.get(2)?,
            })
        })?;
        rows.collect::<SqlResult<Vec<_>>>().map_err(DbError::from)
    }

    /// Rename a category across both expenses and classification_rules tables.
    pub fn rename_category(&self, old_name: &str, new_name: &str) -> Result<(), DbError> {
        let tx = self.conn.unchecked_transaction()?;
        tx.execute(
            "UPDATE expenses SET category = ?2 WHERE category = ?1",
            params![old_name, new_name],
        )?;
        tx.execute(
            "UPDATE classification_rules SET category = ?2 WHERE category = ?1",
            params![old_name, new_name],
        )?;
        tx.commit()?;
        Ok(())
    }

    /// Delete a category: reassign its expenses and rules to `replacement`.
    pub fn delete_category(&self, category: &str, replacement: &str) -> Result<(), DbError> {
        self.rename_category(category, replacement)
    }

    /// Merge multiple source categories into a target category.
    pub fn merge_categories(&self, sources: &[String], target: &str) -> Result<(), DbError> {
        let tx = self.conn.unchecked_transaction()?;
        for source in sources {
            if source != target {
                tx.execute(
                    "UPDATE expenses SET category = ?2 WHERE category = ?1",
                    params![source, target],
                )?;
                tx.execute(
                    "UPDATE classification_rules SET category = ?2 WHERE category = ?1",
                    params![source, target],
                )?;
            }
        }
        tx.commit()?;
        Ok(())
    }

    // ── Title Cleanup Rules ──

    pub fn insert_title_cleanup_rule(&self, rule: &TitleCleanupRule) -> Result<i64, DbError> {
        self.conn.execute(
            "INSERT INTO title_cleanup_rules (pattern, replacement, is_regex) VALUES (?1, ?2, ?3)",
            params![rule.pattern, rule.replacement, rule.is_regex as i32],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn get_all_title_cleanup_rules(&self) -> Result<Vec<TitleCleanupRule>, DbError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, pattern, replacement, is_regex FROM title_cleanup_rules ORDER BY id",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(TitleCleanupRule {
                id: Some(row.get(0)?),
                pattern: row.get(1)?,
                replacement: row.get(2)?,
                is_regex: row.get::<_, i32>(3)? != 0,
            })
        })?;
        rows.collect::<SqlResult<Vec<_>>>().map_err(DbError::from)
    }

    pub fn update_title_cleanup_rule(&self, rule: &TitleCleanupRule) -> Result<(), DbError> {
        let id = rule
            .id
            .ok_or(DbError::InvalidData("Cannot update rule without id".into()))?;
        let rows = self.conn.execute(
            "UPDATE title_cleanup_rules SET pattern = ?1, replacement = ?2, is_regex = ?3 WHERE id = ?4",
            params![rule.pattern, rule.replacement, rule.is_regex as i32, id],
        )?;
        if rows == 0 {
            return Err(DbError::InvalidData(format!(
                "Title cleanup rule with id {} not found",
                id
            )));
        }
        Ok(())
    }

    pub fn delete_title_cleanup_rule(&self, id: i64) -> Result<(), DbError> {
        let rows = self.conn.execute(
            "DELETE FROM title_cleanup_rules WHERE id = ?1",
            params![id],
        )?;
        if rows == 0 {
            return Err(DbError::InvalidData(format!(
                "Title cleanup rule with id {} not found",
                id
            )));
        }
        Ok(())
    }

    pub fn get_title_cleanup_rule(&self, id: i64) -> Result<TitleCleanupRule, DbError> {
        self.conn
            .query_row(
                "SELECT id, pattern, replacement, is_regex FROM title_cleanup_rules WHERE id = ?1",
                params![id],
                |row| {
                    Ok(TitleCleanupRule {
                        id: Some(row.get(0)?),
                        pattern: row.get(1)?,
                        replacement: row.get(2)?,
                        is_regex: row.get::<_, i32>(3)? != 0,
                    })
                },
            )
            .map_err(|e| match e {
                rusqlite::Error::QueryReturnedNoRows => {
                    DbError::InvalidData(format!("Title cleanup rule with id {} not found", id))
                }
                other => DbError::from(other),
            })
    }

    fn build_cleanup_regex(rule: &TitleCleanupRule) -> Result<regex::Regex, DbError> {
        let pattern = if rule.is_regex {
            rule.pattern.clone()
        } else {
            regex::escape(&rule.pattern)
        };
        regex::Regex::new(&pattern).map_err(|e| {
            DbError::InvalidData(format!("Invalid regex pattern '{}': {}", rule.pattern, e))
        })
    }

    fn normalize_whitespace(s: &str) -> String {
        s.split_whitespace().collect::<Vec<_>>().join(" ")
    }

    /// Preview what a cleanup rule would do. Returns (expense_id, original, cleaned) for affected rows.
    pub fn preview_title_cleanup(
        &self,
        rule: &TitleCleanupRule,
    ) -> Result<Vec<(i64, String, String)>, DbError> {
        let re = Self::build_cleanup_regex(rule)?;
        let expenses = self.get_all_expenses()?;
        let mut results = Vec::new();
        for expense in expenses {
            let cleaned = re.replace_all(&expense.title, rule.replacement.as_str());
            let cleaned = Self::normalize_whitespace(&cleaned);
            if cleaned != expense.title {
                results.push((expense.id.unwrap(), expense.title, cleaned));
            }
        }
        Ok(results)
    }

    /// Apply a title cleanup rule to specific expenses. Returns count of updated rows.
    pub fn apply_title_cleanup(
        &self,
        rule: &TitleCleanupRule,
        expense_ids: &[i64],
    ) -> Result<usize, DbError> {
        if expense_ids.is_empty() {
            return Ok(0);
        }
        let re = Self::build_cleanup_regex(rule)?;
        let tx = self.conn.unchecked_transaction()?;
        let mut count = 0;

        for &id in expense_ids {
            let title: String = tx
                .query_row(
                    "SELECT title FROM expenses WHERE id = ?1",
                    params![id],
                    |row| row.get(0),
                )
                .map_err(|e| match e {
                    rusqlite::Error::QueryReturnedNoRows => {
                        DbError::InvalidData(format!("Expense with id {} not found", id))
                    }
                    other => DbError::from(other),
                })?;

            let cleaned = re.replace_all(&title, rule.replacement.as_str());
            let cleaned = Self::normalize_whitespace(&cleaned);
            if cleaned != title {
                tx.execute(
                    "UPDATE expenses SET title = ?1 WHERE id = ?2",
                    params![cleaned, id],
                )?;
                count += 1;
            }
        }

        tx.commit()?;
        Ok(count)
    }

    /// Check if a category name already exists (in either rules or expenses).
    pub fn category_exists(&self, name: &str) -> Result<bool, DbError> {
        let count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM (
                SELECT 1 FROM classification_rules WHERE category = ?1
                UNION ALL
                SELECT 1 FROM expenses WHERE category = ?1
            )",
            params![name],
            |row| row.get(0),
        )?;
        Ok(count > 0)
    }

    /// Create a standalone category by inserting a placeholder rule.
    pub fn create_category(&self, name: &str) -> Result<(), DbError> {
        self.conn.execute(
            "INSERT OR IGNORE INTO classification_rules (pattern, category) VALUES (?1, ?2)",
            params![format!("__category_placeholder__{}", name), name],
        )?;
        Ok(())
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

    // ── Budgets ──

    pub fn get_or_create_budget(&self, year: i32, month: u32) -> Result<i64, DbError> {
        self.conn.execute(
            "INSERT OR IGNORE INTO budgets (year, month) VALUES (?1, ?2)",
            params![year, month],
        )?;
        let id: i64 = self.conn.query_row(
            "SELECT id FROM budgets WHERE year = ?1 AND month = ?2",
            params![year, month],
            |row| row.get(0),
        )?;
        Ok(id)
    }

    pub fn get_budget(&self, year: i32, month: u32) -> Result<Option<Budget>, DbError> {
        let result = self.conn.query_row(
            "SELECT id, year, month FROM budgets WHERE year = ?1 AND month = ?2",
            params![year, month],
            |row| {
                Ok(Budget {
                    id: Some(row.get(0)?),
                    year: row.get(1)?,
                    month: row.get::<_, u32>(2)?,
                })
            },
        );
        match result {
            Ok(b) => Ok(Some(b)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(DbError::from(e)),
        }
    }

    pub fn save_budget_categories(
        &self,
        budget_id: i64,
        categories: &[BudgetCategory],
    ) -> Result<(), DbError> {
        let tx = self.conn.unchecked_transaction()?;
        tx.execute(
            "DELETE FROM budget_categories WHERE budget_id = ?1",
            params![budget_id],
        )?;
        for cat in categories {
            tx.execute(
                "INSERT INTO budget_categories (budget_id, category, amount) VALUES (?1, ?2, ?3)",
                params![budget_id, cat.category, cat.amount],
            )?;
        }
        tx.commit()?;
        Ok(())
    }

    pub fn get_budget_categories(&self, budget_id: i64) -> Result<Vec<BudgetCategory>, DbError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, budget_id, category, amount FROM budget_categories WHERE budget_id = ?1 ORDER BY category",
        )?;
        let rows = stmt.query_map(params![budget_id], |row| {
            Ok(BudgetCategory {
                id: Some(row.get(0)?),
                budget_id: row.get(1)?,
                category: row.get(2)?,
                amount: row.get(3)?,
            })
        })?;
        rows.collect::<SqlResult<Vec<_>>>().map_err(DbError::from)
    }

    pub fn insert_planned_expense(&self, expense: &PlannedExpense) -> Result<i64, DbError> {
        self.conn.execute(
            "INSERT INTO planned_expenses (budget_id, title, amount, date, category) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                expense.budget_id,
                expense.title,
                expense.amount,
                expense.date.to_string(),
                expense.category,
            ],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn delete_planned_expense(&self, id: i64) -> Result<(), DbError> {
        let rows = self.conn.execute(
            "DELETE FROM planned_expenses WHERE id = ?1",
            params![id],
        )?;
        if rows == 0 {
            return Err(DbError::InvalidData(format!(
                "Planned expense with id {} not found",
                id
            )));
        }
        Ok(())
    }

    pub fn get_planned_expenses(&self, budget_id: i64) -> Result<Vec<PlannedExpense>, DbError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, budget_id, title, amount, date, category FROM planned_expenses WHERE budget_id = ?1 ORDER BY date",
        )?;
        let rows = stmt.query_map(params![budget_id], |row| {
            let date_str: String = row.get(4)?;
            let date = chrono::NaiveDate::parse_from_str(&date_str, "%Y-%m-%d").map_err(|e| {
                rusqlite::Error::FromSqlConversionFailure(4, rusqlite::types::Type::Text, Box::new(e))
            })?;
            Ok(PlannedExpense {
                id: Some(row.get(0)?),
                budget_id: row.get(1)?,
                title: row.get(2)?,
                amount: row.get(3)?,
                date,
                category: row.get(5)?,
            })
        })?;
        rows.collect::<SqlResult<Vec<_>>>().map_err(DbError::from)
    }

    pub fn save_calendar_events(
        &self,
        budget_id: i64,
        events: &[CalendarEvent],
    ) -> Result<usize, DbError> {
        let tx = self.conn.unchecked_transaction()?;
        tx.execute(
            "DELETE FROM calendar_events WHERE budget_id = ?1",
            params![budget_id],
        )?;
        let mut count = 0;
        for event in events {
            tx.execute(
                "INSERT INTO calendar_events (budget_id, summary, description, location, start_date, end_date, all_day)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![
                    budget_id,
                    event.summary,
                    event.description,
                    event.location,
                    event.start_date.to_string(),
                    event.end_date.map(|d| d.to_string()),
                    event.all_day as i32,
                ],
            )?;
            count += 1;
        }
        tx.commit()?;
        Ok(count)
    }

    pub fn get_calendar_events(&self, budget_id: i64) -> Result<Vec<CalendarEvent>, DbError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, budget_id, summary, description, location, start_date, end_date, all_day
             FROM calendar_events WHERE budget_id = ?1 ORDER BY start_date",
        )?;
        let rows = stmt.query_map(params![budget_id], |row| {
            let start_str: String = row.get(5)?;
            let start_date = chrono::NaiveDate::parse_from_str(&start_str, "%Y-%m-%d").map_err(|e| {
                rusqlite::Error::FromSqlConversionFailure(5, rusqlite::types::Type::Text, Box::new(e))
            })?;
            let end_str: Option<String> = row.get(6)?;
            let end_date = end_str.and_then(|s| chrono::NaiveDate::parse_from_str(&s, "%Y-%m-%d").ok());
            Ok(CalendarEvent {
                id: Some(row.get(0)?),
                budget_id: row.get(1)?,
                summary: row.get(2)?,
                description: row.get(3)?,
                location: row.get(4)?,
                start_date,
                end_date,
                all_day: row.get::<_, i32>(7)? != 0,
            })
        })?;
        rows.collect::<SqlResult<Vec<_>>>().map_err(DbError::from)
    }

    pub fn get_expenses_for_month(&self, year: i32, month: u32) -> Result<Vec<Expense>, DbError> {
        let month_str = format!("{:02}", month);
        let year_str = year.to_string();
        let mut stmt = self.conn.prepare(
            "SELECT id, title, amount, date, category, classification_source
             FROM expenses
             WHERE strftime('%Y', date) = ?1 AND strftime('%m', date) = ?2
             ORDER BY date DESC",
        )?;
        let rows = stmt.query_map(params![year_str, month_str], |row| {
            let source_str: Option<String> = row.get(5)?;
            let source = source_str.as_deref().and_then(ClassificationSource::from_str_opt);
            let date_str: String = row.get(3)?;
            let date = chrono::NaiveDate::parse_from_str(&date_str, "%Y-%m-%d").map_err(|e| {
                rusqlite::Error::FromSqlConversionFailure(3, rusqlite::types::Type::Text, Box::new(e))
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

    pub fn get_category_averages(&self, months: u32) -> Result<Vec<CategoryAverage>, DbError> {
        let mut stmt = self.conn.prepare(
            "SELECT category, AVG(monthly_total) as average, COUNT(*) as months_with_data
             FROM (
                 SELECT category, strftime('%Y-%m', date) as month, SUM(amount) as monthly_total
                 FROM expenses
                 WHERE category IS NOT NULL AND category != ''
                   AND date >= date('now', '-' || ?1 || ' months')
                 GROUP BY category, month
             )
             GROUP BY category
             ORDER BY category",
        )?;
        let rows = stmt.query_map(params![months], |row| {
            Ok(CategoryAverage {
                category: row.get(0)?,
                average: row.get(1)?,
                months_with_data: row.get::<_, u32>(2)?,
            })
        })?;
        rows.collect::<SqlResult<Vec<_>>>().map_err(DbError::from)
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
        let count = db.insert_expenses_bulk(&expenses, None).unwrap();
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
        assert!(db.insert_expenses_bulk(&expenses, None).is_err());
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

    // ── Update expense ──

    #[test]
    fn update_expense_changes_fields() {
        let db = test_db();
        let id = db.insert_expense(&make_expense("Coffee", 4.50, "2025-01-15")).unwrap();

        let updated = Expense {
            id: Some(id),
            title: "Latte".to_string(),
            amount: 5.75,
            date: NaiveDate::from_ymd_opt(2025, 2, 20).unwrap(),
            category: Some("Drinks".to_string()),
            classification_source: Some(ClassificationSource::Manual),
        };
        db.update_expense(&updated).unwrap();

        let all = db.get_all_expenses().unwrap();
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].title, "Latte");
        assert_eq!(all[0].amount, 5.75);
        assert_eq!(all[0].date.to_string(), "2025-02-20");
        assert_eq!(all[0].category.as_deref(), Some("Drinks"));
    }

    #[test]
    fn update_expense_nonexistent_id_returns_error() {
        let db = test_db();
        let expense = Expense {
            id: Some(9999),
            title: "Ghost".to_string(),
            amount: 1.0,
            date: NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
            category: None,
            classification_source: None,
        };
        let result = db.update_expense(&expense);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[test]
    fn update_expense_without_id_returns_error() {
        let db = test_db();
        let expense = make_expense("No ID", 10.0, "2025-01-01");
        let result = db.update_expense(&expense);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("without id"));
    }

    #[test]
    fn update_expense_rejects_nan() {
        let db = test_db();
        let id = db.insert_expense(&make_expense("Coffee", 4.50, "2025-01-15")).unwrap();
        let expense = Expense {
            id: Some(id),
            title: "Coffee".to_string(),
            amount: f64::NAN,
            date: NaiveDate::from_ymd_opt(2025, 1, 15).unwrap(),
            category: None,
            classification_source: None,
        };
        let result = db.update_expense(&expense);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not a valid number"));
    }

    #[test]
    fn update_expense_rejects_infinity() {
        let db = test_db();
        let id = db.insert_expense(&make_expense("Coffee", 4.50, "2025-01-15")).unwrap();
        let expense = Expense {
            id: Some(id),
            title: "Coffee".to_string(),
            amount: f64::INFINITY,
            date: NaiveDate::from_ymd_opt(2025, 1, 15).unwrap(),
            category: None,
            classification_source: None,
        };
        assert!(db.update_expense(&expense).is_err());
    }

    // ── Delete expense ──

    #[test]
    fn delete_expense_removes_single() {
        let db = test_db();
        let id = db.insert_expense(&make_expense("Coffee", 4.50, "2025-01-15")).unwrap();
        assert_eq!(db.get_all_expenses().unwrap().len(), 1);

        db.delete_expense(id).unwrap();
        assert_eq!(db.get_all_expenses().unwrap().len(), 0);
    }

    #[test]
    fn delete_expense_nonexistent_id_returns_error() {
        let db = test_db();
        let result = db.delete_expense(9999);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[test]
    fn delete_expenses_removes_multiple() {
        let db = test_db();
        let id1 = db.insert_expense(&make_expense("A", 1.0, "2025-01-01")).unwrap();
        let id2 = db.insert_expense(&make_expense("B", 2.0, "2025-01-02")).unwrap();
        let _id3 = db.insert_expense(&make_expense("C", 3.0, "2025-01-03")).unwrap();

        let count = db.delete_expenses(&[id1, id2]).unwrap();
        assert_eq!(count, 2);

        let remaining = db.get_all_expenses().unwrap();
        assert_eq!(remaining.len(), 1);
        assert_eq!(remaining[0].title, "C");
    }

    #[test]
    fn delete_expenses_empty_ids_returns_zero() {
        let db = test_db();
        db.insert_expense(&make_expense("A", 1.0, "2025-01-01")).unwrap();
        let count = db.delete_expenses(&[]).unwrap();
        assert_eq!(count, 0);
        assert_eq!(db.get_all_expenses().unwrap().len(), 1);
    }

    #[test]
    fn delete_expense_not_visible_after_delete() {
        let db = test_db();
        let id = db.insert_expense(&make_expense("Temp", 10.0, "2025-06-01")).unwrap();
        db.delete_expense(id).unwrap();
        let all = db.get_all_expenses().unwrap();
        assert!(all.iter().all(|e| e.id != Some(id)));
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

    // ── Title Cleanup Rules ──

    use crate::models::TitleCleanupRule;

    fn make_literal_rule(pattern: &str, replacement: &str) -> TitleCleanupRule {
        TitleCleanupRule {
            id: None,
            pattern: pattern.to_string(),
            replacement: replacement.to_string(),
            is_regex: false,
        }
    }

    fn make_regex_rule(pattern: &str, replacement: &str) -> TitleCleanupRule {
        TitleCleanupRule {
            id: None,
            pattern: pattern.to_string(),
            replacement: replacement.to_string(),
            is_regex: true,
        }
    }

    #[test]
    fn title_cleanup_crud() {
        let db = test_db();

        // Insert
        let id = db.insert_title_cleanup_rule(&make_literal_rule("CARD", "")).unwrap();
        assert!(id > 0);

        // Get all
        let rules = db.get_all_title_cleanup_rules().unwrap();
        assert_eq!(rules.len(), 1);
        assert_eq!(rules[0].pattern, "CARD");
        assert_eq!(rules[0].replacement, "");
        assert!(!rules[0].is_regex);

        // Update
        let mut rule = rules[0].clone();
        rule.pattern = "CARD NUMBER".to_string();
        rule.replacement = "CARD".to_string();
        db.update_title_cleanup_rule(&rule).unwrap();
        let rules = db.get_all_title_cleanup_rules().unwrap();
        assert_eq!(rules[0].pattern, "CARD NUMBER");
        assert_eq!(rules[0].replacement, "CARD");

        // Delete
        db.delete_title_cleanup_rule(id).unwrap();
        assert_eq!(db.get_all_title_cleanup_rules().unwrap().len(), 0);
    }

    #[test]
    fn title_cleanup_update_without_id_fails() {
        let db = test_db();
        let rule = make_literal_rule("test", "");
        assert!(db.update_title_cleanup_rule(&rule).is_err());
    }

    #[test]
    fn title_cleanup_delete_nonexistent_fails() {
        let db = test_db();
        assert!(db.delete_title_cleanup_rule(9999).is_err());
    }

    #[test]
    fn title_cleanup_get_by_id() {
        let db = test_db();
        let id = db.insert_title_cleanup_rule(&make_literal_rule("TEST", "")).unwrap();
        let rule = db.get_title_cleanup_rule(id).unwrap();
        assert_eq!(rule.pattern, "TEST");
        assert!(db.get_title_cleanup_rule(9999).is_err());
    }

    #[test]
    fn preview_literal_rule() {
        let db = test_db();
        db.insert_expense(&make_expense("OP MC 5575 ORLEN Wroclaw", 71.24, "2025-01-01")).unwrap();
        db.insert_expense(&make_expense("Grocery Store", 52.30, "2025-01-02")).unwrap();

        let rule = make_literal_rule("OP MC 5575 ", "");
        let results = db.preview_title_cleanup(&rule).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].2, "ORLEN Wroclaw");
    }

    #[test]
    fn preview_regex_rule_with_capture_groups() {
        let db = test_db();
        db.insert_expense(&make_expense("Payment ref:12345 Shop ABC", 10.0, "2025-01-01")).unwrap();
        db.insert_expense(&make_expense("No ref here", 20.0, "2025-01-02")).unwrap();

        let rule = make_regex_rule(r"ref:\d+ ", "");
        let results = db.preview_title_cleanup(&rule).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].2, "Payment Shop ABC");
    }

    #[test]
    fn apply_only_selected_ids() {
        let db = test_db();
        let id1 = db.insert_expense(&make_expense("NOISE Coffee", 4.0, "2025-01-01")).unwrap();
        let id2 = db.insert_expense(&make_expense("NOISE Tea", 3.0, "2025-01-02")).unwrap();
        let _id3 = db.insert_expense(&make_expense("NOISE Water", 2.0, "2025-01-03")).unwrap();

        let rule = make_literal_rule("NOISE ", "");
        let rule_id = db.insert_title_cleanup_rule(&rule).unwrap();
        let saved_rule = db.get_title_cleanup_rule(rule_id).unwrap();

        let count = db.apply_title_cleanup(&saved_rule, &[id1, id2]).unwrap();
        assert_eq!(count, 2);

        let expenses = db.get_all_expenses().unwrap();
        let titles: Vec<&str> = expenses.iter().map(|e| e.title.as_str()).collect();
        assert!(titles.contains(&"Coffee"));
        assert!(titles.contains(&"Tea"));
        assert!(titles.contains(&"NOISE Water")); // not applied
    }

    #[test]
    fn apply_returns_zero_when_no_match() {
        let db = test_db();
        let id = db.insert_expense(&make_expense("Coffee", 4.0, "2025-01-01")).unwrap();

        let rule = make_literal_rule("NONEXISTENT", "");
        let count = db.apply_title_cleanup(&rule, &[id]).unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn literal_rule_escapes_special_regex_chars() {
        let db = test_db();
        db.insert_expense(&make_expense("Cost (5.00) item", 5.0, "2025-01-01")).unwrap();

        let rule = make_literal_rule("(5.00) ", "");
        let results = db.preview_title_cleanup(&rule).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].2, "Cost item");
    }

    #[test]
    fn cleanup_collapses_spaces_and_trims() {
        let db = test_db();
        db.insert_expense(&make_expense("A  NOISE  B", 1.0, "2025-01-01")).unwrap();

        let rule = make_literal_rule("NOISE", "");
        let results = db.preview_title_cleanup(&rule).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].2, "A B");
    }

    #[test]
    fn empty_replacement_removes_match() {
        let db = test_db();
        db.insert_expense(&make_expense("Hello World Junk", 1.0, "2025-01-01")).unwrap();

        let rule = make_literal_rule(" Junk", "");
        let results = db.preview_title_cleanup(&rule).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].2, "Hello World");
    }

    #[test]
    fn apply_empty_ids_returns_zero() {
        let db = test_db();
        let rule = make_literal_rule("test", "");
        let count = db.apply_title_cleanup(&rule, &[]).unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn invalid_regex_returns_error() {
        let db = test_db();
        db.insert_expense(&make_expense("test", 1.0, "2025-01-01")).unwrap();

        let rule = make_regex_rule("[invalid", "");
        assert!(db.preview_title_cleanup(&rule).is_err());
    }

    // ── Budget tests ──

    use crate::models::{BudgetCategory, CalendarEvent, PlannedExpense};

    #[test]
    fn budget_get_or_create_idempotent() {
        let db = test_db();
        let id1 = db.get_or_create_budget(2026, 3).unwrap();
        let id2 = db.get_or_create_budget(2026, 3).unwrap();
        assert_eq!(id1, id2);
    }

    #[test]
    fn budget_get_returns_none_when_missing() {
        let db = test_db();
        assert!(db.get_budget(2026, 3).unwrap().is_none());
    }

    #[test]
    fn budget_get_returns_some_after_create() {
        let db = test_db();
        db.get_or_create_budget(2026, 3).unwrap();
        let budget = db.get_budget(2026, 3).unwrap().unwrap();
        assert_eq!(budget.year, 2026);
        assert_eq!(budget.month, 3);
    }

    #[test]
    fn budget_categories_save_and_retrieve() {
        let db = test_db();
        let bid = db.get_or_create_budget(2026, 3).unwrap();
        let cats = vec![
            BudgetCategory { id: None, budget_id: bid, category: "Food".into(), amount: 500.0 },
            BudgetCategory { id: None, budget_id: bid, category: "Transport".into(), amount: 200.0 },
            BudgetCategory { id: None, budget_id: bid, category: "Health".into(), amount: 300.0 },
        ];
        db.save_budget_categories(bid, &cats).unwrap();

        let loaded = db.get_budget_categories(bid).unwrap();
        assert_eq!(loaded.len(), 3);
        assert_eq!(loaded[0].category, "Food");
        assert_eq!(loaded[0].amount, 500.0);
    }

    #[test]
    fn budget_categories_replace_on_resave() {
        let db = test_db();
        let bid = db.get_or_create_budget(2026, 3).unwrap();
        let cats1 = vec![
            BudgetCategory { id: None, budget_id: bid, category: "Food".into(), amount: 500.0 },
        ];
        db.save_budget_categories(bid, &cats1).unwrap();

        let cats2 = vec![
            BudgetCategory { id: None, budget_id: bid, category: "Food".into(), amount: 800.0 },
            BudgetCategory { id: None, budget_id: bid, category: "Fun".into(), amount: 100.0 },
        ];
        db.save_budget_categories(bid, &cats2).unwrap();

        let loaded = db.get_budget_categories(bid).unwrap();
        assert_eq!(loaded.len(), 2);
        let food = loaded.iter().find(|c| c.category == "Food").unwrap();
        assert_eq!(food.amount, 800.0);
    }

    #[test]
    fn planned_expenses_crud() {
        let db = test_db();
        let bid = db.get_or_create_budget(2026, 3).unwrap();

        let pe1 = PlannedExpense {
            id: None, budget_id: bid, title: "Dentist".into(), amount: 300.0,
            date: NaiveDate::from_ymd_opt(2026, 3, 12).unwrap(), category: Some("Health".into()),
        };
        let pe2 = PlannedExpense {
            id: None, budget_id: bid, title: "Car service".into(), amount: 500.0,
            date: NaiveDate::from_ymd_opt(2026, 3, 20).unwrap(), category: None,
        };
        let id1 = db.insert_planned_expense(&pe1).unwrap();
        let id2 = db.insert_planned_expense(&pe2).unwrap();

        let loaded = db.get_planned_expenses(bid).unwrap();
        assert_eq!(loaded.len(), 2);
        assert_eq!(loaded[0].title, "Dentist");
        assert_eq!(loaded[1].title, "Car service");

        db.delete_planned_expense(id1).unwrap();
        let loaded = db.get_planned_expenses(bid).unwrap();
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].id, Some(id2));
    }

    #[test]
    fn planned_expense_delete_nonexistent_fails() {
        let db = test_db();
        assert!(db.delete_planned_expense(9999).is_err());
    }

    #[test]
    fn calendar_events_save_and_replace() {
        let db = test_db();
        let bid = db.get_or_create_budget(2026, 3).unwrap();

        let events1: Vec<CalendarEvent> = (1..=5).map(|i| CalendarEvent {
            id: None, budget_id: bid, summary: format!("Event {}", i),
            description: None, location: None,
            start_date: NaiveDate::from_ymd_opt(2026, 3, i).unwrap(),
            end_date: None, all_day: false,
        }).collect();
        let count = db.save_calendar_events(bid, &events1).unwrap();
        assert_eq!(count, 5);
        assert_eq!(db.get_calendar_events(bid).unwrap().len(), 5);

        // Re-save with fewer events replaces
        let events2: Vec<CalendarEvent> = (1..=3).map(|i| CalendarEvent {
            id: None, budget_id: bid, summary: format!("New Event {}", i),
            description: None, location: None,
            start_date: NaiveDate::from_ymd_opt(2026, 3, i).unwrap(),
            end_date: None, all_day: true,
        }).collect();
        let count = db.save_calendar_events(bid, &events2).unwrap();
        assert_eq!(count, 3);

        let loaded = db.get_calendar_events(bid).unwrap();
        assert_eq!(loaded.len(), 3);
        assert!(loaded[0].all_day);
    }

    #[test]
    fn get_expenses_for_month_filters() {
        let db = test_db();
        let mut e1 = make_expense("Jan item", 100.0, "2026-01-15");
        e1.category = Some("Food".into());
        let mut e2 = make_expense("Feb item 1", 200.0, "2026-02-10");
        e2.category = Some("Food".into());
        let mut e3 = make_expense("Feb item 2", 150.0, "2026-02-20");
        e3.category = Some("Transport".into());
        let mut e4 = make_expense("Mar item", 300.0, "2026-03-05");
        e4.category = Some("Food".into());

        db.insert_expense(&e1).unwrap();
        db.insert_expense(&e2).unwrap();
        db.insert_expense(&e3).unwrap();
        db.insert_expense(&e4).unwrap();

        let feb = db.get_expenses_for_month(2026, 2).unwrap();
        assert_eq!(feb.len(), 2);
        // Ordered by date DESC
        assert_eq!(feb[0].title, "Feb item 2");
        assert_eq!(feb[1].title, "Feb item 1");
    }

    #[test]
    fn get_category_averages_computes() {
        let db = test_db();
        // Insert expenses across months — use recent dates relative to "now"
        let dates = ["2025-12-01", "2026-01-01", "2026-02-01"];
        for (i, date) in dates.iter().enumerate() {
            let amount = (i + 1) as f64 * 100.0; // 100, 200, 300
            let mut expense = make_expense("Food purchase", amount, date);
            expense.category = Some("Food".into());
            db.insert_expense(&expense).unwrap();
        }

        let avgs = db.get_category_averages(6).unwrap();
        assert!(!avgs.is_empty());
        let food = avgs.iter().find(|a| a.category == "Food").unwrap();
        // Each month has one expense: 100, 200, 300 → avg = 200
        assert!((food.average - 200.0).abs() < 0.01);
        assert_eq!(food.months_with_data, 3);
    }

    // ── Upload Batch tests ──

    #[test]
    fn bulk_insert_with_batch_tracking() {
        let db = test_db();
        let expenses = vec![
            make_expense("A", 1.0, "2025-01-01"),
            make_expense("B", 2.0, "2025-01-02"),
            make_expense("C", 3.0, "2025-01-03"),
        ];
        let count = db.insert_expenses_bulk(&expenses, Some("test.csv")).unwrap();
        assert_eq!(count, 3);

        let batches = db.get_upload_batches().unwrap();
        assert_eq!(batches.len(), 1);
        assert_eq!(batches[0].filename.as_deref(), Some("test.csv"));
        assert_eq!(batches[0].expense_count, 3);
    }

    #[test]
    fn bulk_insert_without_batch() {
        let db = test_db();
        let expenses = vec![
            make_expense("A", 1.0, "2025-01-01"),
            make_expense("B", 2.0, "2025-01-02"),
        ];
        db.insert_expenses_bulk(&expenses, None).unwrap();

        let batches = db.get_upload_batches().unwrap();
        assert!(batches.is_empty());
        assert_eq!(db.get_all_expenses().unwrap().len(), 2);
    }

    #[test]
    fn delete_batch_removes_expenses_and_record() {
        let db = test_db();
        let expenses = vec![
            make_expense("A", 1.0, "2025-01-01"),
            make_expense("B", 2.0, "2025-01-02"),
            make_expense("C", 3.0, "2025-01-03"),
        ];
        db.insert_expenses_bulk(&expenses, Some("test.csv")).unwrap();

        let batches = db.get_upload_batches().unwrap();
        let batch_id = batches[0].id;

        let deleted = db.delete_batch(batch_id).unwrap();
        assert_eq!(deleted, 3);
        assert!(db.get_all_expenses().unwrap().is_empty());
        assert!(db.get_upload_batches().unwrap().is_empty());
    }

    #[test]
    fn delete_batch_doesnt_affect_other_expenses() {
        let db = test_db();
        // Batch A
        let batch_a = vec![
            make_expense("A1", 1.0, "2025-01-01"),
            make_expense("A2", 2.0, "2025-01-02"),
        ];
        db.insert_expenses_bulk(&batch_a, Some("a.csv")).unwrap();

        // Batch B
        let batch_b = vec![
            make_expense("B1", 3.0, "2025-01-03"),
        ];
        db.insert_expenses_bulk(&batch_b, Some("b.csv")).unwrap();

        let batches = db.get_upload_batches().unwrap();
        // batches are ordered by uploaded_at DESC, so batch B is first
        let batch_a_id = batches.iter().find(|b| b.filename.as_deref() == Some("a.csv")).unwrap().id;

        db.delete_batch(batch_a_id).unwrap();

        let remaining = db.get_all_expenses().unwrap();
        assert_eq!(remaining.len(), 1);
        assert_eq!(remaining[0].title, "B1");

        let batches = db.get_upload_batches().unwrap();
        assert_eq!(batches.len(), 1);
        assert_eq!(batches[0].filename.as_deref(), Some("b.csv"));
    }

    #[test]
    fn manual_expenses_unaffected_by_batch_delete() {
        let db = test_db();
        // Manual expense (no batch)
        db.insert_expense(&make_expense("Manual", 10.0, "2025-01-01")).unwrap();

        // Batch
        let batch = vec![make_expense("Batch1", 5.0, "2025-01-02")];
        db.insert_expenses_bulk(&batch, Some("file.csv")).unwrap();

        let batches = db.get_upload_batches().unwrap();
        db.delete_batch(batches[0].id).unwrap();

        let remaining = db.get_all_expenses().unwrap();
        assert_eq!(remaining.len(), 1);
        assert_eq!(remaining[0].title, "Manual");
    }
}
