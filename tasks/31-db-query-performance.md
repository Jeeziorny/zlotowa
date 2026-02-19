# 31 — DB Query Performance

## Problem

Several database patterns cause unnecessary load that will degrade as the table grows:

1. **N+1 in `apply_title_cleanup()`** (`db.rs:755-777`) — loops with individual SELECT + UPDATE per expense ID. 100 expenses = 200 queries.
2. **N+1 in `delete_expenses()`** (`db.rs:477-479`) — loops with individual DELETE per ID instead of `WHERE id IN (...)`.
3. **`preview_title_cleanup()` loads all expenses** (`db.rs:729-740`) — calls `get_all_expenses()` then filters in Rust instead of SQL.
4. **`strftime()` prevents index use** (`db.rs:1011-1012`) — `WHERE strftime('%Y', date) = ?1` forces full table scan on every budget/month query. Replace with range queries (`date >= ? AND date < ?`).
5. **Missing compound index on budgets(year, month)** (`db.rs:100`) — `UNIQUE` constraint doesn't automatically create a usable index in all SQLite versions.
6. **OR-chain in `check_duplicates_batch()`** (`db.rs:398-401`) — generates N OR conditions; large CSVs (1000+ rows) risk hitting SQLite query limits. Batch into chunks or use temp table.

## Scope

- Rewrite `apply_title_cleanup()` to use batch UPDATE with CASE/WHEN or temp table
- Rewrite `delete_expenses()` to use `WHERE id IN (?...)`
- Rewrite `preview_title_cleanup()` to filter in SQL (LIKE or regex extension)
- Replace `strftime()` date queries with range comparisons
- Add explicit index on `budgets(year, month)`
- Chunk `check_duplicates_batch()` OR-chain into groups of ~100
