use crate::models::{
    Budget, BudgetCategory, CalendarEvent, CategoryAverage, CategoryStats, ClassificationRule,
    ClassificationSource, Expense, ExpenseQuery, ExpenseQueryResult, PlannedExpense,
    TitleCleanupRule, UploadBatch,
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

        // Budget date-range migration: year/month → start_date/end_date (idempotent)
        // Wrapped in a transaction so a crash mid-migration can't leave the DB
        // without a `budgets` table. PRAGMA foreign_keys is restored even on error.
        let has_old_schema = self
            .conn
            .prepare("SELECT year FROM budgets LIMIT 0")
            .is_ok();
        if has_old_schema {
            self.conn.execute_batch("PRAGMA foreign_keys = OFF;")?;
            let migration_result = (|| -> Result<(), DbError> {
                let tx = self.conn.unchecked_transaction()?;
                self.conn.execute_batch(
                    "CREATE TABLE budgets_v2 (
                        id         INTEGER PRIMARY KEY AUTOINCREMENT,
                        start_date TEXT NOT NULL,
                        end_date   TEXT NOT NULL
                    );",
                )?;
                // Migrate existing rows: year/month → first-of-month / last-day-of-month
                self.conn.execute_batch(
                    "INSERT INTO budgets_v2 (id, start_date, end_date)
                     SELECT id,
                            printf('%04d-%02d-01', year, month),
                            date(printf('%04d-%02d-01', year, month), '+1 month', '-1 day')
                     FROM budgets;",
                )?;
                self.conn.execute_batch("DROP TABLE budgets;")?;
                self.conn
                    .execute_batch("ALTER TABLE budgets_v2 RENAME TO budgets;")?;
                self.conn.execute_batch(
                    "CREATE INDEX IF NOT EXISTS idx_budgets_dates ON budgets(start_date, end_date);",
                )?;
                tx.commit()?;
                Ok(())
            })();
            self.conn.execute_batch("PRAGMA foreign_keys = ON;")?;
            migration_result?;
        }

        // Calendar events amount column (idempotent ALTER, ignore error)
        let _ = self
            .conn
            .execute_batch("ALTER TABLE calendar_events ADD COLUMN amount REAL;");

        // FK indices on budget child tables (idempotent)
        self.conn.execute_batch(
            "CREATE INDEX IF NOT EXISTS idx_budget_categories_budget_id ON budget_categories(budget_id);
             CREATE INDEX IF NOT EXISTS idx_planned_expenses_budget_id ON planned_expenses(budget_id);
             CREATE INDEX IF NOT EXISTS idx_calendar_events_budget_id ON calendar_events(budget_id);",
        )?;

        // UNIQUE constraint on title_cleanup_rules(pattern, replacement, is_regex)
        // Ignoring error in case existing data has duplicates
        let _ = self.conn.execute_batch(
            "CREATE UNIQUE INDEX IF NOT EXISTS idx_title_cleanup_rules_unique ON title_cleanup_rules(pattern, replacement, is_regex);",
        );

        Ok(())
    }

    /// Run a closure inside a database transaction.
    /// All Database method calls within the closure share the same transaction.
    /// Commits on success, rolls back on error.
    ///
    /// **Note:** Do not nest `with_transaction` or call methods that internally
    /// use `unchecked_transaction()` (e.g. `save_budget_categories`,
    /// `insert_expenses_bulk`) from within the closure — use the dedicated
    /// combined methods instead (e.g. `create_budget_with_categories`,
    /// `insert_expenses_bulk` with rules).
    pub fn with_transaction<T, F>(&self, f: F) -> Result<T, DbError>
    where
        F: FnOnce() -> Result<T, DbError>,
    {
        let tx = self.conn.unchecked_transaction()?;
        let result = f()?;
        tx.commit()?;
        Ok(result)
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

    /// Insert multiple expenses and classification rules atomically.
    /// Either all succeed or none are saved.
    /// When `batch_filename` is `Some`, creates an upload batch record and links expenses to it.
    pub fn insert_expenses_bulk(
        &self,
        expenses: &[Expense],
        batch_filename: Option<&str>,
        rules: &[ClassificationRule],
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

        for rule in rules {
            tx.execute(
                "INSERT OR REPLACE INTO classification_rules (pattern, category) VALUES (?1, ?2)",
                params![rule.pattern, rule.category],
            )?;
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

    pub fn query_expenses(&self, query: &ExpenseQuery) -> Result<ExpenseQueryResult, DbError> {
        let mut conditions: Vec<String> = Vec::new();
        let mut param_values: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();
        let mut idx = 1;

        if let Some(ref search) = query.search {
            if !search.is_empty() {
                conditions.push(format!("title LIKE '%' || ?{} || '%' COLLATE NOCASE", idx));
                param_values.push(Box::new(search.clone()));
                idx += 1;
            }
        }

        if let Some(ref category) = query.category {
            if category == crate::models::UNCATEGORIZED {
                conditions.push("category IS NULL".to_string());
            } else {
                conditions.push(format!("category = ?{}", idx));
                param_values.push(Box::new(category.clone()));
                idx += 1;
            }
        }

        if let Some(date_from) = query.date_from {
            conditions.push(format!("date >= ?{}", idx));
            param_values.push(Box::new(date_from.to_string()));
            idx += 1;
        }

        if let Some(date_to) = query.date_to {
            conditions.push(format!("date <= ?{}", idx));
            param_values.push(Box::new(date_to.to_string()));
            idx += 1;
        }

        if let Some(amount_min) = query.amount_min {
            conditions.push(format!("amount >= ?{}", idx));
            param_values.push(Box::new(amount_min));
            idx += 1;
        }

        if let Some(amount_max) = query.amount_max {
            conditions.push(format!("amount <= ?{}", idx));
            param_values.push(Box::new(amount_max));
            idx += 1;
        }

        let where_clause = if conditions.is_empty() {
            String::new()
        } else {
            format!(" WHERE {}", conditions.join(" AND "))
        };

        // Count total matching rows
        let count_sql = format!("SELECT COUNT(*) FROM expenses{}", where_clause);
        let params_refs: Vec<&dyn rusqlite::types::ToSql> =
            param_values.iter().map(|p| p.as_ref()).collect();
        let total_count: i64 = self
            .conn
            .query_row(&count_sql, params_refs.as_slice(), |row| row.get(0))?;

        // Fetch page
        let limit = query.limit.unwrap_or(50);
        let offset = query.offset.unwrap_or(0);

        let select_sql = format!(
            "SELECT id, title, amount, date, category, classification_source FROM expenses{} ORDER BY date DESC LIMIT ?{} OFFSET ?{}",
            where_clause, idx, idx + 1
        );
        param_values.push(Box::new(limit));
        param_values.push(Box::new(offset));

        let params_refs: Vec<&dyn rusqlite::types::ToSql> =
            param_values.iter().map(|p| p.as_ref()).collect();
        let mut stmt = self.conn.prepare(&select_sql)?;
        let rows = stmt.query_map(params_refs.as_slice(), |row| {
            let source_str: Option<String> = row.get(5)?;
            let source = source_str
                .as_deref()
                .and_then(ClassificationSource::from_str_opt);
            let date_str: String = row.get(3)?;
            let date = chrono::NaiveDate::parse_from_str(&date_str, "%Y-%m-%d").map_err(|e| {
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
        let expenses = rows.collect::<SqlResult<Vec<_>>>().map_err(DbError::from)?;

        Ok(ExpenseQueryResult {
            expenses,
            total_count,
        })
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

        const CHUNK_SIZE: usize = 100;
        let mut all_found: Vec<(String, f64, String)> = Vec::new();

        for chunk in expenses.chunks(CHUNK_SIZE) {
            let mut conditions = Vec::new();
            let mut param_values: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

            for (i, (title, amount, date)) in chunk.iter().enumerate() {
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

            let params_refs: Vec<&dyn rusqlite::types::ToSql> =
                param_values.iter().map(|p| p.as_ref()).collect();
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
            all_found.extend(found);
        }

        let results = expenses
            .iter()
            .map(|(title, amount, date)| {
                let date_str = date.to_string();
                all_found
                    .iter()
                    .any(|(t, a, d)| t == title && *a == *amount && *d == date_str)
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
        let placeholders: Vec<String> = (1..=ids.len()).map(|i| format!("?{}", i)).collect();
        let sql = format!(
            "DELETE FROM expenses WHERE id IN ({})",
            placeholders.join(",")
        );
        let params: Vec<&dyn rusqlite::types::ToSql> =
            ids.iter().map(|id| id as &dyn rusqlite::types::ToSql).collect();
        let count = self.conn.execute(&sql, params.as_slice())?;
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
        let mut stmt = self
            .conn
            .prepare("SELECT id, title FROM expenses")?;
        let rows = stmt.query_map([], |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
        })?;
        let id_titles: Vec<(i64, String)> = rows.collect::<SqlResult<Vec<_>>>().map_err(DbError::from)?;
        let mut results = Vec::new();
        for (id, title) in id_titles {
            let cleaned = re.replace_all(&title, rule.replacement.as_str());
            let cleaned = Self::normalize_whitespace(&cleaned);
            if cleaned != title {
                results.push((id, title, cleaned));
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

        // Batch-fetch all titles in one query
        let placeholders: Vec<String> = (1..=expense_ids.len()).map(|i| format!("?{}", i)).collect();
        let sql = format!(
            "SELECT id, title FROM expenses WHERE id IN ({})",
            placeholders.join(",")
        );
        let params: Vec<&dyn rusqlite::types::ToSql> =
            expense_ids.iter().map(|id| id as &dyn rusqlite::types::ToSql).collect();
        let mut stmt = self.conn.prepare(&sql)?;
        let rows = stmt.query_map(params.as_slice(), |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
        })?;
        let fetched: Vec<(i64, String)> = rows.collect::<SqlResult<Vec<_>>>().map_err(DbError::from)?;

        // Check all requested IDs were found
        if fetched.len() != expense_ids.len() {
            let found_ids: std::collections::HashSet<i64> = fetched.iter().map(|(id, _)| *id).collect();
            for &id in expense_ids {
                if !found_ids.contains(&id) {
                    return Err(DbError::InvalidData(format!("Expense with id {} not found", id)));
                }
            }
        }

        // Apply regex and batch-update
        let tx = self.conn.unchecked_transaction()?;
        let mut count = 0;
        for (id, title) in &fetched {
            let cleaned = re.replace_all(title, rule.replacement.as_str());
            let cleaned = Self::normalize_whitespace(&cleaned);
            if cleaned != *title {
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

    pub fn create_budget(
        &self,
        start_date: chrono::NaiveDate,
        end_date: chrono::NaiveDate,
    ) -> Result<i64, DbError> {
        if start_date >= end_date {
            return Err(DbError::InvalidData(
                "start_date must be before end_date".into(),
            ));
        }
        if self.check_budget_overlap(start_date, end_date)? {
            return Err(DbError::InvalidData(
                "Budget overlaps with an existing budget".into(),
            ));
        }
        self.conn.execute(
            "INSERT INTO budgets (start_date, end_date) VALUES (?1, ?2)",
            params![start_date.to_string(), end_date.to_string()],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn get_budget_by_id(&self, id: i64) -> Result<Option<Budget>, DbError> {
        let result = self.conn.query_row(
            "SELECT id, start_date, end_date FROM budgets WHERE id = ?1",
            params![id],
            |row| {
                let start_str: String = row.get(1)?;
                let end_str: String = row.get(2)?;
                let start_date =
                    chrono::NaiveDate::parse_from_str(&start_str, "%Y-%m-%d").map_err(|e| {
                        rusqlite::Error::FromSqlConversionFailure(
                            1,
                            rusqlite::types::Type::Text,
                            Box::new(e),
                        )
                    })?;
                let end_date =
                    chrono::NaiveDate::parse_from_str(&end_str, "%Y-%m-%d").map_err(|e| {
                        rusqlite::Error::FromSqlConversionFailure(
                            2,
                            rusqlite::types::Type::Text,
                            Box::new(e),
                        )
                    })?;
                Ok(Budget {
                    id: Some(row.get(0)?),
                    start_date,
                    end_date,
                })
            },
        );
        match result {
            Ok(b) => Ok(Some(b)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(DbError::from(e)),
        }
    }

    pub fn get_active_budget(&self) -> Result<Option<Budget>, DbError> {
        let result = self.conn.query_row(
            "SELECT id, start_date, end_date FROM budgets
             WHERE start_date <= date('now') AND end_date >= date('now')",
            [],
            |row| {
                let start_str: String = row.get(1)?;
                let end_str: String = row.get(2)?;
                let start_date =
                    chrono::NaiveDate::parse_from_str(&start_str, "%Y-%m-%d").map_err(|e| {
                        rusqlite::Error::FromSqlConversionFailure(
                            1,
                            rusqlite::types::Type::Text,
                            Box::new(e),
                        )
                    })?;
                let end_date =
                    chrono::NaiveDate::parse_from_str(&end_str, "%Y-%m-%d").map_err(|e| {
                        rusqlite::Error::FromSqlConversionFailure(
                            2,
                            rusqlite::types::Type::Text,
                            Box::new(e),
                        )
                    })?;
                Ok(Budget {
                    id: Some(row.get(0)?),
                    start_date,
                    end_date,
                })
            },
        );
        match result {
            Ok(b) => Ok(Some(b)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(DbError::from(e)),
        }
    }

    pub fn check_budget_overlap(
        &self,
        start_date: chrono::NaiveDate,
        end_date: chrono::NaiveDate,
    ) -> Result<bool, DbError> {
        let count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM budgets WHERE start_date < ?2 AND end_date > ?1",
            params![start_date.to_string(), end_date.to_string()],
            |row| row.get(0),
        )?;
        Ok(count > 0)
    }

    pub fn delete_budget(&self, id: i64) -> Result<(), DbError> {
        let tx = self.conn.unchecked_transaction()?;
        tx.execute(
            "DELETE FROM calendar_events WHERE budget_id = ?1",
            params![id],
        )?;
        tx.execute(
            "DELETE FROM planned_expenses WHERE budget_id = ?1",
            params![id],
        )?;
        tx.execute(
            "DELETE FROM budget_categories WHERE budget_id = ?1",
            params![id],
        )?;
        let rows = tx.execute("DELETE FROM budgets WHERE id = ?1", params![id])?;
        tx.commit()?;
        if rows == 0 {
            return Err(DbError::InvalidData(format!(
                "Budget with id {} not found",
                id
            )));
        }
        Ok(())
    }

    pub fn save_budget_categories(
        &self,
        budget_id: i64,
        categories: &[BudgetCategory],
    ) -> Result<(), DbError> {
        let tx = self.conn.unchecked_transaction()?;
        self.save_budget_categories_inner(budget_id, categories)?;
        tx.commit()?;
        Ok(())
    }

    /// Replace budget categories without managing a transaction.
    /// Caller must ensure this runs inside a transaction.
    fn save_budget_categories_inner(
        &self,
        budget_id: i64,
        categories: &[BudgetCategory],
    ) -> Result<(), DbError> {
        self.conn.execute(
            "DELETE FROM budget_categories WHERE budget_id = ?1",
            params![budget_id],
        )?;
        for cat in categories {
            self.conn.execute(
                "INSERT INTO budget_categories (budget_id, category, amount) VALUES (?1, ?2, ?3)",
                params![budget_id, cat.category, cat.amount],
            )?;
        }
        Ok(())
    }

    /// Create a budget and its categories atomically in a single transaction.
    pub fn create_budget_with_categories(
        &self,
        start_date: chrono::NaiveDate,
        end_date: chrono::NaiveDate,
        categories: &[BudgetCategory],
    ) -> Result<i64, DbError> {
        if start_date >= end_date {
            return Err(DbError::InvalidData(
                "start_date must be before end_date".into(),
            ));
        }
        if self.check_budget_overlap(start_date, end_date)? {
            return Err(DbError::InvalidData(
                "Budget overlaps with an existing budget".into(),
            ));
        }
        let tx = self.conn.unchecked_transaction()?;
        self.conn.execute(
            "INSERT INTO budgets (start_date, end_date) VALUES (?1, ?2)",
            params![start_date.to_string(), end_date.to_string()],
        )?;
        let budget_id = self.conn.last_insert_rowid();
        self.save_budget_categories_inner(budget_id, categories)?;
        tx.commit()?;
        Ok(budget_id)
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
                "INSERT INTO calendar_events (budget_id, summary, description, location, start_date, end_date, all_day, amount)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![
                    budget_id,
                    event.summary,
                    event.description,
                    event.location,
                    event.start_date.to_string(),
                    event.end_date.map(|d| d.to_string()),
                    event.all_day as i32,
                    event.amount,
                ],
            )?;
            count += 1;
        }
        tx.commit()?;
        Ok(count)
    }

    pub fn get_calendar_events(&self, budget_id: i64) -> Result<Vec<CalendarEvent>, DbError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, budget_id, summary, description, location, start_date, end_date, all_day, amount
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
                amount: row.get(8)?,
            })
        })?;
        rows.collect::<SqlResult<Vec<_>>>().map_err(DbError::from)
    }

    pub fn update_calendar_event_amount(
        &self,
        event_id: i64,
        amount: Option<f64>,
    ) -> Result<(), DbError> {
        let rows = self.conn.execute(
            "UPDATE calendar_events SET amount = ?1 WHERE id = ?2",
            params![amount, event_id],
        )?;
        if rows == 0 {
            return Err(DbError::InvalidData(format!(
                "Calendar event with id {} not found",
                event_id
            )));
        }
        Ok(())
    }

    pub fn get_expenses_for_date_range(
        &self,
        start: chrono::NaiveDate,
        end: chrono::NaiveDate,
    ) -> Result<Vec<Expense>, DbError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, title, amount, date, category, classification_source
             FROM expenses
             WHERE date >= ?1 AND date <= ?2
             ORDER BY date DESC",
        )?;
        let rows = stmt.query_map(params![start.to_string(), end.to_string()], |row| {
            let source_str: Option<String> = row.get(5)?;
            let source = source_str
                .as_deref()
                .and_then(ClassificationSource::from_str_opt);
            let date_str: String = row.get(3)?;
            let date = chrono::NaiveDate::parse_from_str(&date_str, "%Y-%m-%d").map_err(|e| {
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

    // ── Bulk insert ──

    #[test]
    fn bulk_insert_expenses() {
        let db = test_db();
        let expenses = vec![
            make_expense("A", 1.0, "2025-01-01"),
            make_expense("B", 2.0, "2025-01-02"),
            make_expense("C", 3.0, "2025-01-03"),
        ];
        let count = db.insert_expenses_bulk(&expenses, None, &[]).unwrap();
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
        assert!(db.insert_expenses_bulk(&expenses, None, &[]).is_err());
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

    fn create_test_budget(db: &Database, start: &str, end: &str) -> i64 {
        db.create_budget(
            NaiveDate::parse_from_str(start, "%Y-%m-%d").unwrap(),
            NaiveDate::parse_from_str(end, "%Y-%m-%d").unwrap(),
        ).unwrap()
    }

    #[test]
    fn budget_create_basic() {
        let db = test_db();
        let bid = create_test_budget(&db, "2026-03-01", "2026-03-31");
        assert!(bid > 0);
        let budget = db.get_budget_by_id(bid).unwrap().unwrap();
        assert_eq!(budget.start_date.to_string(), "2026-03-01");
        assert_eq!(budget.end_date.to_string(), "2026-03-31");
    }

    #[test]
    fn budget_create_overlap_rejected() {
        let db = test_db();
        create_test_budget(&db, "2026-03-01", "2026-03-31");
        // Fully contained
        let r = db.create_budget(
            NaiveDate::from_ymd_opt(2026, 3, 10).unwrap(),
            NaiveDate::from_ymd_opt(2026, 3, 20).unwrap(),
        );
        assert!(r.is_err());
        assert!(r.unwrap_err().to_string().contains("overlap"));
        // Partial overlap
        let r = db.create_budget(
            NaiveDate::from_ymd_opt(2026, 3, 15).unwrap(),
            NaiveDate::from_ymd_opt(2026, 4, 15).unwrap(),
        );
        assert!(r.is_err());
    }

    #[test]
    fn budget_create_adjacent_allowed() {
        let db = test_db();
        create_test_budget(&db, "2026-01-01", "2026-01-31");
        // Adjacent: Feb starts right after Jan ends
        let bid2 = create_test_budget(&db, "2026-02-01", "2026-02-28");
        assert!(bid2 > 0);
    }

    #[test]
    fn budget_get_by_id() {
        let db = test_db();
        let bid = create_test_budget(&db, "2026-03-01", "2026-03-31");
        assert!(db.get_budget_by_id(bid).unwrap().is_some());
        assert!(db.get_budget_by_id(9999).unwrap().is_none());
    }

    #[test]
    fn budget_get_active_finds_current() {
        let db = test_db();
        // Create budget spanning today
        let today = chrono::Local::now().date_naive();
        let start = today - chrono::Duration::days(5);
        let end = today + chrono::Duration::days(25);
        db.create_budget(start, end).unwrap();
        let active = db.get_active_budget().unwrap();
        assert!(active.is_some());
    }

    #[test]
    fn budget_get_active_returns_none() {
        let db = test_db();
        // Budget in the past
        create_test_budget(&db, "2020-01-01", "2020-01-31");
        assert!(db.get_active_budget().unwrap().is_none());
    }

    #[test]
    fn budget_categories_save_and_retrieve() {
        let db = test_db();
        let bid = create_test_budget(&db, "2026-03-01", "2026-03-31");
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
        let bid = create_test_budget(&db, "2026-03-01", "2026-03-31");
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
        let bid = create_test_budget(&db, "2026-03-01", "2026-03-31");

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
        let bid = create_test_budget(&db, "2026-03-01", "2026-03-31");

        let events1: Vec<CalendarEvent> = (1..=5).map(|i| CalendarEvent {
            id: None, budget_id: bid, summary: format!("Event {}", i),
            description: None, location: None,
            start_date: NaiveDate::from_ymd_opt(2026, 3, i).unwrap(),
            end_date: None, all_day: false, amount: None,
        }).collect();
        let count = db.save_calendar_events(bid, &events1).unwrap();
        assert_eq!(count, 5);
        assert_eq!(db.get_calendar_events(bid).unwrap().len(), 5);

        // Re-save with fewer events replaces
        let events2: Vec<CalendarEvent> = (1..=3).map(|i| CalendarEvent {
            id: None, budget_id: bid, summary: format!("New Event {}", i),
            description: None, location: None,
            start_date: NaiveDate::from_ymd_opt(2026, 3, i).unwrap(),
            end_date: None, all_day: true, amount: Some(50.0),
        }).collect();
        let count = db.save_calendar_events(bid, &events2).unwrap();
        assert_eq!(count, 3);

        let loaded = db.get_calendar_events(bid).unwrap();
        assert_eq!(loaded.len(), 3);
        assert!(loaded[0].all_day);
        assert_eq!(loaded[0].amount, Some(50.0));
    }

    #[test]
    fn calendar_event_amount_update() {
        let db = test_db();
        let bid = create_test_budget(&db, "2026-03-01", "2026-03-31");
        let events = vec![CalendarEvent {
            id: None, budget_id: bid, summary: "Dentist".into(),
            description: None, location: None,
            start_date: NaiveDate::from_ymd_opt(2026, 3, 5).unwrap(),
            end_date: None, all_day: false, amount: None,
        }];
        db.save_calendar_events(bid, &events).unwrap();

        let loaded = db.get_calendar_events(bid).unwrap();
        assert_eq!(loaded[0].amount, None);

        let event_id = loaded[0].id.unwrap();
        db.update_calendar_event_amount(event_id, Some(150.0)).unwrap();

        let loaded = db.get_calendar_events(bid).unwrap();
        assert_eq!(loaded[0].amount, Some(150.0));

        // Clear amount
        db.update_calendar_event_amount(event_id, None).unwrap();
        let loaded = db.get_calendar_events(bid).unwrap();
        assert_eq!(loaded[0].amount, None);
    }

    #[test]
    fn calendar_event_amount_null_default() {
        let db = test_db();
        let bid = create_test_budget(&db, "2026-03-01", "2026-03-31");
        let events = vec![CalendarEvent {
            id: None, budget_id: bid, summary: "Event".into(),
            description: None, location: None,
            start_date: NaiveDate::from_ymd_opt(2026, 3, 1).unwrap(),
            end_date: None, all_day: false, amount: None,
        }];
        db.save_calendar_events(bid, &events).unwrap();
        let loaded = db.get_calendar_events(bid).unwrap();
        assert_eq!(loaded[0].amount, None);
    }

    #[test]
    fn budget_delete_cascades() {
        let db = test_db();
        let bid = create_test_budget(&db, "2026-03-01", "2026-03-31");
        let cats = vec![
            BudgetCategory { id: None, budget_id: bid, category: "Food".into(), amount: 500.0 },
        ];
        db.save_budget_categories(bid, &cats).unwrap();
        let pe = PlannedExpense {
            id: None, budget_id: bid, title: "Dentist".into(), amount: 300.0,
            date: NaiveDate::from_ymd_opt(2026, 3, 12).unwrap(), category: None,
        };
        db.insert_planned_expense(&pe).unwrap();
        let events = vec![CalendarEvent {
            id: None, budget_id: bid, summary: "Event".into(),
            description: None, location: None,
            start_date: NaiveDate::from_ymd_opt(2026, 3, 5).unwrap(),
            end_date: None, all_day: false, amount: None,
        }];
        db.save_calendar_events(bid, &events).unwrap();

        db.delete_budget(bid).unwrap();

        assert!(db.get_budget_by_id(bid).unwrap().is_none());
        assert_eq!(db.get_budget_categories(bid).unwrap().len(), 0);
        assert_eq!(db.get_planned_expenses(bid).unwrap().len(), 0);
        assert_eq!(db.get_calendar_events(bid).unwrap().len(), 0);
    }

    #[test]
    fn get_expenses_for_date_range_filters() {
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

        let feb = db.get_expenses_for_date_range(
            NaiveDate::from_ymd_opt(2026, 2, 1).unwrap(),
            NaiveDate::from_ymd_opt(2026, 2, 28).unwrap(),
        ).unwrap();
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
        let count = db.insert_expenses_bulk(&expenses, Some("test.csv"), &[]).unwrap();
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
        db.insert_expenses_bulk(&expenses, None, &[]).unwrap();

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
        db.insert_expenses_bulk(&expenses, Some("test.csv"), &[]).unwrap();

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
        db.insert_expenses_bulk(&batch_a, Some("a.csv"), &[]).unwrap();

        // Batch B
        let batch_b = vec![
            make_expense("B1", 3.0, "2025-01-03"),
        ];
        db.insert_expenses_bulk(&batch_b, Some("b.csv"), &[]).unwrap();

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
        db.insert_expenses_bulk(&batch, Some("file.csv"), &[]).unwrap();

        let batches = db.get_upload_batches().unwrap();
        db.delete_batch(batches[0].id).unwrap();

        let remaining = db.get_all_expenses().unwrap();
        assert_eq!(remaining.len(), 1);
        assert_eq!(remaining[0].title, "Manual");
    }

    // ── Query expenses ──

    use crate::models::ExpenseQuery;

    fn seed_query_db(db: &Database) {
        let expenses = vec![
            Expense { id: None, title: "Coffee Shop".into(), amount: 4.50, date: NaiveDate::from_ymd_opt(2025, 3, 1).unwrap(), category: Some("Food".into()), classification_source: None },
            Expense { id: None, title: "Gas Station".into(), amount: 55.00, date: NaiveDate::from_ymd_opt(2025, 3, 5).unwrap(), category: Some("Transport".into()), classification_source: None },
            Expense { id: None, title: "Grocery Store".into(), amount: 120.30, date: NaiveDate::from_ymd_opt(2025, 3, 10).unwrap(), category: Some("Food".into()), classification_source: None },
            Expense { id: None, title: "Electric Bill".into(), amount: 89.99, date: NaiveDate::from_ymd_opt(2025, 3, 15).unwrap(), category: Some("Utilities".into()), classification_source: None },
            Expense { id: None, title: "Mystery Payment".into(), amount: 25.00, date: NaiveDate::from_ymd_opt(2025, 3, 20).unwrap(), category: None, classification_source: None },
        ];
        for e in &expenses {
            db.insert_expense(e).unwrap();
        }
    }

    #[test]
    fn query_expenses_empty_query_returns_all() {
        let db = test_db();
        seed_query_db(&db);
        let result = db.query_expenses(&ExpenseQuery::default()).unwrap();
        assert_eq!(result.total_count, 5);
        assert_eq!(result.expenses.len(), 5);
    }

    #[test]
    fn query_expenses_search_by_title() {
        let db = test_db();
        seed_query_db(&db);
        let query = ExpenseQuery { search: Some("coffee".into()), ..Default::default() };
        let result = db.query_expenses(&query).unwrap();
        assert_eq!(result.total_count, 1);
        assert_eq!(result.expenses[0].title, "Coffee Shop");
    }

    #[test]
    fn query_expenses_search_case_insensitive() {
        let db = test_db();
        seed_query_db(&db);
        let query = ExpenseQuery { search: Some("GROCERY".into()), ..Default::default() };
        let result = db.query_expenses(&query).unwrap();
        assert_eq!(result.total_count, 1);
        assert_eq!(result.expenses[0].title, "Grocery Store");
    }

    #[test]
    fn query_expenses_filter_by_category() {
        let db = test_db();
        seed_query_db(&db);
        let query = ExpenseQuery { category: Some("Food".into()), ..Default::default() };
        let result = db.query_expenses(&query).unwrap();
        assert_eq!(result.total_count, 2);
        assert!(result.expenses.iter().all(|e| e.category.as_deref() == Some("Food")));
    }

    #[test]
    fn query_expenses_filter_uncategorized() {
        let db = test_db();
        seed_query_db(&db);
        let query = ExpenseQuery { category: Some(crate::models::UNCATEGORIZED.into()), ..Default::default() };
        let result = db.query_expenses(&query).unwrap();
        assert_eq!(result.total_count, 1);
        assert_eq!(result.expenses[0].title, "Mystery Payment");
        assert!(result.expenses[0].category.is_none());
    }

    #[test]
    fn query_expenses_filter_date_range() {
        let db = test_db();
        seed_query_db(&db);
        let query = ExpenseQuery {
            date_from: Some(NaiveDate::from_ymd_opt(2025, 3, 5).unwrap()),
            date_to: Some(NaiveDate::from_ymd_opt(2025, 3, 15).unwrap()),
            ..Default::default()
        };
        let result = db.query_expenses(&query).unwrap();
        assert_eq!(result.total_count, 3); // Gas Station, Grocery Store, Electric Bill
    }

    #[test]
    fn query_expenses_filter_amount_range() {
        let db = test_db();
        seed_query_db(&db);
        let query = ExpenseQuery {
            amount_min: Some(10.0),
            amount_max: Some(50.0),
            ..Default::default()
        };
        let result = db.query_expenses(&query).unwrap();
        assert_eq!(result.total_count, 1); // Mystery Payment (25.00)
        assert_eq!(result.expenses[0].title, "Mystery Payment");
    }

    #[test]
    fn query_expenses_pagination() {
        let db = test_db();
        seed_query_db(&db);
        // Page 1: limit 2
        let q1 = ExpenseQuery { limit: Some(2), offset: Some(0), ..Default::default() };
        let r1 = db.query_expenses(&q1).unwrap();
        assert_eq!(r1.expenses.len(), 2);
        assert_eq!(r1.total_count, 5);

        // Page 2
        let q2 = ExpenseQuery { limit: Some(2), offset: Some(2), ..Default::default() };
        let r2 = db.query_expenses(&q2).unwrap();
        assert_eq!(r2.expenses.len(), 2);
        assert_eq!(r2.total_count, 5);

        // Page 3 (partial)
        let q3 = ExpenseQuery { limit: Some(2), offset: Some(4), ..Default::default() };
        let r3 = db.query_expenses(&q3).unwrap();
        assert_eq!(r3.expenses.len(), 1);
        assert_eq!(r3.total_count, 5);

        // No overlap between pages
        let all_titles: Vec<String> = r1.expenses.iter()
            .chain(r2.expenses.iter())
            .chain(r3.expenses.iter())
            .map(|e| e.title.clone())
            .collect();
        assert_eq!(all_titles.len(), 5);
    }

    #[test]
    fn query_expenses_combined_filters() {
        let db = test_db();
        seed_query_db(&db);
        let query = ExpenseQuery {
            search: Some("o".into()),       // matches Coffee Shop, Grocery Store
            category: Some("Food".into()),  // matches Coffee Shop, Grocery Store
            amount_max: Some(10.0),         // only Coffee Shop (4.50)
            ..Default::default()
        };
        let result = db.query_expenses(&query).unwrap();
        assert_eq!(result.total_count, 1);
        assert_eq!(result.expenses[0].title, "Coffee Shop");
    }

    #[test]
    fn query_expenses_total_count_independent_of_pagination() {
        let db = test_db();
        seed_query_db(&db);
        let query = ExpenseQuery {
            category: Some("Food".into()),
            limit: Some(1),
            offset: Some(0),
            ..Default::default()
        };
        let result = db.query_expenses(&query).unwrap();
        assert_eq!(result.expenses.len(), 1);
        assert_eq!(result.total_count, 2); // 2 Food expenses total
    }

    // ── Category management ──

    #[test]
    fn category_stats_counts_expenses_and_rules() {
        let db = test_db();
        let e = Expense {
            id: None,
            title: "Lidl".into(),
            amount: 50.0,
            date: NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
            category: Some("Food".into()),
            classification_source: None,
        };
        db.insert_expense(&e).unwrap();
        db.insert_expense(&Expense { title: "Biedronka".into(), ..e.clone() }).unwrap();
        db.insert_rule(&ClassificationRule::from_pattern("lidl", "Food")).unwrap();
        db.insert_rule(&ClassificationRule::from_pattern("uber", "Transport")).unwrap();

        let stats = db.get_category_stats().unwrap();
        let food = stats.iter().find(|s| s.name == "Food").unwrap();
        assert_eq!(food.expense_count, 2);
        assert_eq!(food.rule_count, 1);

        let transport = stats.iter().find(|s| s.name == "Transport").unwrap();
        assert_eq!(transport.expense_count, 0);
        assert_eq!(transport.rule_count, 1);
    }

    #[test]
    fn rename_category_updates_expenses_and_rules() {
        let db = test_db();
        let e = Expense {
            id: None,
            title: "Lidl".into(),
            amount: 50.0,
            date: NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
            category: Some("Groceries".into()),
            classification_source: None,
        };
        db.insert_expense(&e).unwrap();
        db.insert_rule(&ClassificationRule::from_pattern("lidl", "Groceries")).unwrap();

        db.rename_category("Groceries", "Food").unwrap();

        let expenses = db.get_all_expenses().unwrap();
        assert_eq!(expenses[0].category.as_deref(), Some("Food"));

        let rules = db.get_all_rules().unwrap();
        assert_eq!(rules[0].category, "Food");
    }

    #[test]
    fn merge_categories_combines_into_target() {
        let db = test_db();
        let base = Expense {
            id: None,
            title: "".into(),
            amount: 10.0,
            date: NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
            category: None,
            classification_source: None,
        };
        db.insert_expense(&Expense { title: "Lidl".into(), category: Some("Groceries".into()), ..base.clone() }).unwrap();
        db.insert_expense(&Expense { title: "Tesco".into(), category: Some("Supermarket".into()), ..base.clone() }).unwrap();
        db.insert_expense(&Expense { title: "Cafe".into(), category: Some("Food".into()), ..base.clone() }).unwrap();
        db.insert_rule(&ClassificationRule::from_pattern("lidl", "Groceries")).unwrap();
        db.insert_rule(&ClassificationRule::from_pattern("tesco", "Supermarket")).unwrap();

        db.merge_categories(&["Groceries".into(), "Supermarket".into()], "Food").unwrap();

        let expenses = db.get_all_expenses().unwrap();
        assert!(expenses.iter().all(|e| e.category.as_deref() == Some("Food")));

        let rules = db.get_all_rules().unwrap();
        assert!(rules.iter().all(|r| r.category == "Food"));
    }

    #[test]
    fn merge_categories_skips_target_in_sources() {
        let db = test_db();
        let e = Expense {
            id: None,
            title: "Test".into(),
            amount: 10.0,
            date: NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
            category: Some("Food".into()),
            classification_source: None,
        };
        db.insert_expense(&e).unwrap();

        // Including target in sources should not cause issues
        db.merge_categories(&["Food".into()], "Food").unwrap();

        let expenses = db.get_all_expenses().unwrap();
        assert_eq!(expenses[0].category.as_deref(), Some("Food"));
    }

    #[test]
    fn category_exists_checks_both_tables() {
        let db = test_db();

        assert!(!db.category_exists("Food").unwrap());

        // Exists via rule
        db.insert_rule(&ClassificationRule::from_pattern("lidl", "Food")).unwrap();
        assert!(db.category_exists("Food").unwrap());

        // Exists via expense only
        let e = Expense {
            id: None,
            title: "Uber".into(),
            amount: 15.0,
            date: NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
            category: Some("Transport".into()),
            classification_source: None,
        };
        db.insert_expense(&e).unwrap();
        assert!(db.category_exists("Transport").unwrap());
    }

    #[test]
    fn create_category_makes_it_discoverable() {
        let db = test_db();
        db.create_category("Entertainment").unwrap();
        assert!(db.category_exists("Entertainment").unwrap());

        let categories = db.get_all_categories().unwrap();
        assert!(categories.contains(&"Entertainment".to_string()));
    }

    #[test]
    fn create_category_is_idempotent() {
        let db = test_db();
        db.create_category("Food").unwrap();
        db.create_category("Food").unwrap(); // should not error
        let categories = db.get_all_categories().unwrap();
        assert_eq!(categories.iter().filter(|c| c.as_str() == "Food").count(), 1);
    }

    // ── Batch duplicate check ──

    #[test]
    fn check_duplicates_batch_empty_input() {
        let db = test_db();
        let result = db.check_duplicates_batch(&[]).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn check_duplicates_batch_finds_existing() {
        let db = test_db();
        let date = NaiveDate::from_ymd_opt(2025, 3, 1).unwrap();
        db.insert_expense(&Expense {
            id: None,
            title: "Coffee".into(),
            amount: 4.50,
            date,
            category: None,
            classification_source: None,
        }).unwrap();

        let inputs = vec![
            ("Coffee", 4.50, &date),       // duplicate
            ("Coffee", 5.00, &date),       // different amount
            ("Tea", 4.50, &date),          // different title
        ];
        let results = db.check_duplicates_batch(&inputs).unwrap();
        assert_eq!(results, vec![true, false, false]);
    }

    // ── Upload batches ──

    #[test]
    fn get_upload_batches_returns_all_batches() {
        let db = test_db();
        let expenses = vec![make_expense("A", 10.0, "2025-01-01")];
        db.insert_expenses_bulk(&expenses, Some("file1.csv"), &[]).unwrap();
        db.insert_expenses_bulk(&expenses, Some("file2.csv"), &[]).unwrap();

        let batches = db.get_upload_batches().unwrap();
        assert_eq!(batches.len(), 2);
        // Most recent first
        assert_eq!(batches[0].filename.as_deref(), Some("file2.csv"));
        assert_eq!(batches[1].filename.as_deref(), Some("file1.csv"));
    }

    #[test]
    fn get_upload_batches_excludes_manual_inserts() {
        let db = test_db();
        db.insert_expense(&make_expense("Manual", 10.0, "2025-01-01")).unwrap();
        let batches = db.get_upload_batches().unwrap();
        assert!(batches.is_empty());
    }

    // ── Edge cases: query_expenses ──

    #[test]
    fn query_expenses_amount_min_greater_than_max_returns_empty() {
        let db = test_db();
        seed_query_db(&db);
        let query = ExpenseQuery {
            amount_min: Some(100.0),
            amount_max: Some(1.0),
            ..Default::default()
        };
        let result = db.query_expenses(&query).unwrap();
        assert_eq!(result.total_count, 0);
        assert!(result.expenses.is_empty());
    }

    #[test]
    fn query_expenses_date_from_after_date_to_returns_empty() {
        let db = test_db();
        seed_query_db(&db);
        let query = ExpenseQuery {
            date_from: Some(NaiveDate::from_ymd_opt(2025, 12, 1).unwrap()),
            date_to: Some(NaiveDate::from_ymd_opt(2025, 1, 1).unwrap()),
            ..Default::default()
        };
        let result = db.query_expenses(&query).unwrap();
        assert_eq!(result.total_count, 0);
    }

    #[test]
    fn query_expenses_unicode_search() {
        let db = test_db();
        let e = Expense {
            id: None,
            title: "Żabka sklep".into(),
            amount: 15.0,
            date: NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
            category: None,
            classification_source: None,
        };
        db.insert_expense(&e).unwrap();
        let query = ExpenseQuery { search: Some("Żabka".into()), ..Default::default() };
        let result = db.query_expenses(&query).unwrap();
        assert_eq!(result.total_count, 1);
        assert_eq!(result.expenses[0].title, "Żabka sklep");
    }

    #[test]
    fn query_expenses_limit_zero_returns_empty_expenses_but_correct_count() {
        let db = test_db();
        seed_query_db(&db);
        let query = ExpenseQuery { limit: Some(0), ..Default::default() };
        let result = db.query_expenses(&query).unwrap();
        assert!(result.expenses.is_empty());
        assert_eq!(result.total_count, 5);
    }

    #[test]
    fn query_expenses_offset_beyond_total() {
        let db = test_db();
        seed_query_db(&db);
        let query = ExpenseQuery { offset: Some(999), ..Default::default() };
        let result = db.query_expenses(&query).unwrap();
        assert!(result.expenses.is_empty());
        assert_eq!(result.total_count, 5);
    }

    #[test]
    fn query_expenses_empty_search_string_returns_all() {
        let db = test_db();
        seed_query_db(&db);
        let query = ExpenseQuery { search: Some("".into()), ..Default::default() };
        let result = db.query_expenses(&query).unwrap();
        assert_eq!(result.total_count, 5);
    }

    // ── Edge cases: rename_category ──

    #[test]
    fn rename_category_to_empty_string() {
        let db = test_db();
        let e = Expense {
            id: None,
            title: "Test".into(),
            amount: 10.0,
            date: NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
            category: Some("Food".into()),
            classification_source: None,
        };
        db.insert_expense(&e).unwrap();
        // Renaming to empty string succeeds at DB level (no constraint)
        db.rename_category("Food", "").unwrap();
        let expenses = db.get_all_expenses().unwrap();
        assert_eq!(expenses[0].category.as_deref(), Some(""));
    }

    #[test]
    fn rename_category_nonexistent_is_noop() {
        let db = test_db();
        // Renaming a category that doesn't exist should not error
        db.rename_category("NonExistent", "Whatever").unwrap();
    }

    #[test]
    fn rename_category_same_name_is_noop() {
        let db = test_db();
        let e = Expense {
            id: None,
            title: "Test".into(),
            amount: 10.0,
            date: NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
            category: Some("Food".into()),
            classification_source: None,
        };
        db.insert_expense(&e).unwrap();
        db.rename_category("Food", "Food").unwrap();
        let expenses = db.get_all_expenses().unwrap();
        assert_eq!(expenses[0].category.as_deref(), Some("Food"));
    }

    #[test]
    fn rename_category_to_existing_name_merges() {
        let db = test_db();
        let base = Expense {
            id: None,
            title: "".into(),
            amount: 10.0,
            date: NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
            category: None,
            classification_source: None,
        };
        db.insert_expense(&Expense { title: "A".into(), category: Some("Groceries".into()), ..base.clone() }).unwrap();
        db.insert_expense(&Expense { title: "B".into(), category: Some("Food".into()), ..base.clone() }).unwrap();
        db.rename_category("Groceries", "Food").unwrap();
        let expenses = db.get_all_expenses().unwrap();
        assert!(expenses.iter().all(|e| e.category.as_deref() == Some("Food")));
    }

    // ── Edge cases: merge_categories ──

    #[test]
    fn merge_categories_empty_sources() {
        let db = test_db();
        // Empty sources list should be a no-op
        db.merge_categories(&[], "Food").unwrap();
    }

    #[test]
    fn merge_categories_duplicate_sources() {
        let db = test_db();
        let base = Expense {
            id: None,
            title: "Lidl".into(),
            amount: 10.0,
            date: NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
            category: Some("Groceries".into()),
            classification_source: None,
        };
        db.insert_expense(&base).unwrap();
        // Same source listed twice should not cause issues
        db.merge_categories(&["Groceries".into(), "Groceries".into()], "Food").unwrap();
        let expenses = db.get_all_expenses().unwrap();
        assert_eq!(expenses[0].category.as_deref(), Some("Food"));
    }

    #[test]
    fn merge_categories_nonexistent_sources() {
        let db = test_db();
        // Merging nonexistent categories should be a no-op
        db.merge_categories(&["NoSuch".into(), "AlsoNot".into()], "Food").unwrap();
    }

    // ── Performance optimization tests ──

    #[test]
    fn delete_expenses_batch_large_input() {
        let db = test_db();
        let expenses: Vec<Expense> = (0..160)
            .map(|i| make_expense(&format!("Item {}", i), i as f64, "2025-06-01"))
            .collect();
        db.insert_expenses_bulk(&expenses, None, &[]).unwrap();
        let all = db.get_all_expenses().unwrap();
        assert_eq!(all.len(), 160);

        let ids: Vec<i64> = all.iter().map(|e| e.id.unwrap()).collect();
        let count = db.delete_expenses(&ids).unwrap();
        assert_eq!(count, 160);
        assert!(db.get_all_expenses().unwrap().is_empty());
    }

    #[test]
    fn check_duplicates_batch_large_input() {
        let db = test_db();
        // Insert 50 expenses
        let dates: Vec<NaiveDate> = (1..=28)
            .map(|d| NaiveDate::from_ymd_opt(2025, 6, d).unwrap())
            .collect();
        for (i, date) in dates.iter().enumerate() {
            db.insert_expense(&Expense {
                id: None,
                title: format!("Expense {}", i),
                amount: (i + 1) as f64 * 10.0,
                date: *date,
                category: None,
                classification_source: None,
            })
            .unwrap();
        }

        // Build 160 check entries: first 28 are duplicates, rest are new
        let check_dates: Vec<NaiveDate> = (0..160)
            .map(|i| {
                let day = (i % 28) + 1;
                NaiveDate::from_ymd_opt(2025, 6, day as u32).unwrap()
            })
            .collect();
        let titles: Vec<String> = (0..160)
            .map(|i| {
                if i < 28 {
                    format!("Expense {}", i)
                } else {
                    format!("New expense {}", i)
                }
            })
            .collect();
        let inputs: Vec<(&str, f64, &NaiveDate)> = (0..160)
            .map(|i| {
                (
                    titles[i].as_str(),
                    if i < 28 { (i + 1) as f64 * 10.0 } else { i as f64 * 100.0 },
                    &check_dates[i],
                )
            })
            .collect();

        let results = db.check_duplicates_batch(&inputs).unwrap();
        assert_eq!(results.len(), 160);
        // First 28 should be duplicates
        for i in 0..28 {
            assert!(results[i], "Expected duplicate at index {}", i);
        }
        // Rest should not be duplicates
        for i in 28..160 {
            assert!(!results[i], "Expected non-duplicate at index {}", i);
        }
    }

    #[test]
    fn get_expenses_for_date_range_boundary_exclusion() {
        let db = test_db();
        // Last day of January
        db.insert_expense(&make_expense("Jan 31", 10.0, "2026-01-31")).unwrap();
        // All of February
        db.insert_expense(&make_expense("Feb 1", 20.0, "2026-02-01")).unwrap();
        db.insert_expense(&make_expense("Feb 15", 30.0, "2026-02-15")).unwrap();
        db.insert_expense(&make_expense("Feb 28", 40.0, "2026-02-28")).unwrap();
        // First day of March
        db.insert_expense(&make_expense("Mar 1", 50.0, "2026-03-01")).unwrap();

        let feb = db.get_expenses_for_date_range(
            NaiveDate::from_ymd_opt(2026, 2, 1).unwrap(),
            NaiveDate::from_ymd_opt(2026, 2, 28).unwrap(),
        ).unwrap();
        assert_eq!(feb.len(), 3);
        let titles: Vec<&str> = feb.iter().map(|e| e.title.as_str()).collect();
        assert!(titles.contains(&"Feb 1"));
        assert!(titles.contains(&"Feb 15"));
        assert!(titles.contains(&"Feb 28"));
        assert!(!titles.contains(&"Jan 31"));
        assert!(!titles.contains(&"Mar 1"));
    }

    #[test]
    fn get_expenses_for_date_range_december_boundary() {
        let db = test_db();
        db.insert_expense(&make_expense("Nov 30", 10.0, "2025-11-30")).unwrap();
        db.insert_expense(&make_expense("Dec 1", 20.0, "2025-12-01")).unwrap();
        db.insert_expense(&make_expense("Dec 31", 30.0, "2025-12-31")).unwrap();
        db.insert_expense(&make_expense("Jan 1 next", 40.0, "2026-01-01")).unwrap();

        let dec = db.get_expenses_for_date_range(
            NaiveDate::from_ymd_opt(2025, 12, 1).unwrap(),
            NaiveDate::from_ymd_opt(2025, 12, 31).unwrap(),
        ).unwrap();
        assert_eq!(dec.len(), 2);
        let titles: Vec<&str> = dec.iter().map(|e| e.title.as_str()).collect();
        assert!(titles.contains(&"Dec 1"));
        assert!(titles.contains(&"Dec 31"));
    }

    #[test]
    fn title_cleanup_rules_do_not_auto_apply_on_insert() {
        let db = test_db();

        // Create a title cleanup rule
        let rule = make_literal_rule("NOISE ", "");
        db.insert_title_cleanup_rule(&rule).unwrap();

        // Insert expenses with titles matching the rule
        db.insert_expense(&make_expense("NOISE Coffee Shop", 4.50, "2025-01-15"))
            .unwrap();
        db.insert_expense(&make_expense("NOISE Grocery Store", 52.30, "2025-01-16"))
            .unwrap();

        // Also test bulk insert path
        let bulk = vec![
            make_expense("NOISE Gas Station", 60.00, "2025-01-17"),
            make_expense("NOISE Pharmacy", 12.99, "2025-01-18"),
        ];
        db.insert_expenses_bulk(&bulk, Some("test.csv"), &[]).unwrap();

        // Verify titles were NOT cleaned — rules don't auto-apply on insert
        let all = db.get_all_expenses().unwrap();
        assert_eq!(all.len(), 4);
        for e in &all {
            assert!(
                e.title.starts_with("NOISE "),
                "Expected title to still contain 'NOISE ', got: '{}'. \
                 Title cleanup rules should not auto-apply during insert.",
                e.title
            );
        }
    }

    #[test]
    fn preview_title_cleanup_returns_correct_affected_rows() {
        let db = test_db();
        // Insert 50 expenses, half matching the rule
        for i in 0..50 {
            let title = if i % 2 == 0 {
                format!("NOISE Item {}", i)
            } else {
                format!("Clean Item {}", i)
            };
            db.insert_expense(&make_expense(&title, i as f64, "2025-01-01"))
                .unwrap();
        }

        let rule = make_literal_rule("NOISE ", "");
        let results = db.preview_title_cleanup(&rule).unwrap();
        assert_eq!(results.len(), 25);
        for (_, original, cleaned) in &results {
            assert!(original.starts_with("NOISE "));
            assert!(!cleaned.starts_with("NOISE "));
        }
    }

    #[test]
    fn apply_title_cleanup_batch_fetch() {
        let db = test_db();
        let mut ids = Vec::new();
        for i in 0..30 {
            let id = db
                .insert_expense(&make_expense(
                    &format!("PREFIX Item {}", i),
                    i as f64,
                    "2025-01-01",
                ))
                .unwrap();
            ids.push(id);
        }

        let rule = make_literal_rule("PREFIX ", "");
        let count = db.apply_title_cleanup(&rule, &ids).unwrap();
        assert_eq!(count, 30);

        let all = db.get_all_expenses().unwrap();
        for e in &all {
            assert!(
                e.title.starts_with("Item "),
                "Expected title to start with 'Item ', got: {}",
                e.title
            );
        }
    }

    // ── Transaction safety tests ──

    #[test]
    fn with_transaction_commits_on_success() {
        let db = test_db();
        db.with_transaction(|| {
            db.insert_expense(&make_expense("TX1", 10.0, "2025-01-01"))?;
            db.insert_expense(&make_expense("TX2", 20.0, "2025-01-02"))?;
            Ok(())
        })
        .unwrap();
        assert_eq!(db.get_all_expenses().unwrap().len(), 2);
    }

    #[test]
    fn with_transaction_rolls_back_on_error() {
        let db = test_db();
        // Insert one expense outside the transaction
        db.insert_expense(&make_expense("Before", 1.0, "2025-01-01"))
            .unwrap();

        let result = db.with_transaction(|| {
            db.insert_expense(&make_expense("Inside", 2.0, "2025-01-02"))?;
            // Force an error by inserting NaN
            db.insert_expense(&make_expense("Bad", f64::NAN, "2025-01-03"))?;
            Ok(())
        });
        assert!(result.is_err());

        // Only the pre-transaction expense should remain
        let all = db.get_all_expenses().unwrap();
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].title, "Before");
    }

    #[test]
    fn create_budget_with_categories_is_atomic() {
        let db = test_db();
        let start = NaiveDate::from_ymd_opt(2026, 6, 1).unwrap();
        let end = NaiveDate::from_ymd_opt(2026, 6, 30).unwrap();

        let cats = vec![
            BudgetCategory {
                id: None,
                budget_id: 0,
                category: "Food".into(),
                amount: 500.0,
            },
            BudgetCategory {
                id: None,
                budget_id: 0,
                category: "Transport".into(),
                amount: 200.0,
            },
        ];

        let budget_id = db
            .create_budget_with_categories(start, end, &cats)
            .unwrap();

        let saved_cats = db.get_budget_categories(budget_id).unwrap();
        assert_eq!(saved_cats.len(), 2);
        assert_eq!(saved_cats[0].budget_id, budget_id);
        assert_eq!(saved_cats[1].budget_id, budget_id);

        let budget = db.get_budget_by_id(budget_id).unwrap().unwrap();
        assert_eq!(budget.start_date, start);
        assert_eq!(budget.end_date, end);
    }

    #[test]
    fn insert_expenses_bulk_with_rules_is_atomic() {
        let db = test_db();
        let expenses = vec![
            make_expense("Coffee", 5.0, "2025-01-01"),
            make_expense("Bus", 2.0, "2025-01-02"),
        ];
        let rules = vec![
            ClassificationRule::from_pattern("Coffee", "Food"),
            ClassificationRule::from_pattern("Bus", "Transport"),
        ];

        let count = db
            .insert_expenses_bulk(&expenses, Some("test.csv"), &rules)
            .unwrap();
        assert_eq!(count, 2);

        // Rules were saved in the same transaction
        let saved_rules = db.get_all_rules().unwrap();
        assert!(saved_rules.iter().any(|r| r.category == "Food"));
        assert!(saved_rules.iter().any(|r| r.category == "Transport"));
    }
}
