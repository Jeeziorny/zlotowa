use accountant_core::classifiers::{classify_pipeline, Classifier, RegexClassifier};
use accountant_core::db::Database;
use accountant_core::llm::{create_provider, LlmConfig};
use accountant_core::models::{
    BudgetCategory, BudgetCategoryStatus, BudgetStatus, CalendarEvent, CategoryAverage,
    CategoryStats, ClassificationRule, ClassificationSource, Expense, ExpenseQuery,
    ExpenseQueryResult, ParsedExpense, PlannedExpense, TitleCleanupRule, UploadBatch, Budget,
};
use accountant_core::parsers::{self, ColumnMapping};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tauri::State;

pub struct AppState {
    pub db: Mutex<Database>,
}

// ── Types ──

#[derive(Serialize, Deserialize)]
pub struct ExpenseInput {
    pub title: String,
    pub amount: f64,
    pub date: String,
    pub category: Option<String>,
    pub rule_pattern: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct LlmConfigInput {
    pub provider: String,
    pub api_key: String,
}

#[derive(Serialize, Deserialize)]
pub struct LlmConfigOutput {
    pub provider: Option<String>,
    pub api_key: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PreviewResult {
    pub parser_name: String,
    pub rows: Vec<Vec<String>>,
}

#[derive(Serialize, Deserialize)]
pub struct ClassifiedExpenseRow {
    pub title: String,
    pub amount: f64,
    pub date: String,
    pub category: Option<String>,
    pub source: Option<String>,
    pub confidence: Option<f64>,
    pub is_duplicate: bool,
}

#[derive(Serialize, Deserialize)]
pub struct BulkSaveExpense {
    pub title: String,
    pub amount: f64,
    pub date: String,
    pub category: Option<String>,
    pub source: Option<String>,
    pub rule_pattern: Option<String>,
}

// ── Helpers ──

fn parse_date(s: &str) -> Result<chrono::NaiveDate, String> {
    chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d")
        .map_err(|e| format!("Invalid date '{}': {}", s, e))
}

// ── Expense Commands ──

#[tauri::command]
fn get_expenses(state: State<AppState>) -> Result<Vec<Expense>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.get_all_expenses().map_err(|e| e.to_string())
}

#[tauri::command]
fn query_expenses(state: State<AppState>, query: ExpenseQuery) -> Result<ExpenseQueryResult, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.query_expenses(&query).map_err(|e| e.to_string())
}

#[tauri::command]
fn add_expense(state: State<AppState>, input: ExpenseInput) -> Result<i64, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let date = parse_date(&input.date)?;

    db.with_transaction(|| {
        let expense = Expense {
            id: None,
            title: input.title.clone(),
            amount: input.amount,
            date,
            category: input.category.clone(),
            classification_source: Some(ClassificationSource::Manual),
        };

        let id = db.insert_expense(&expense)?;
        if let Some(cat) = &input.category {
            if !cat.is_empty() {
                let pattern_source = input.rule_pattern
                    .as_deref()
                    .filter(|p| !p.is_empty())
                    .unwrap_or(&input.title);
                db.insert_rule(&ClassificationRule::from_pattern(pattern_source, cat))?;
            }
        }
        Ok(id)
    })
    .map_err(|e| e.to_string())
}

#[tauri::command]
fn get_categories(state: State<AppState>) -> Result<Vec<String>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.get_all_categories().map_err(|e| e.to_string())
}

#[tauri::command]
fn update_expense(state: State<AppState>, id: i64, input: ExpenseInput) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let date = parse_date(&input.date)?;

    db.with_transaction(|| {
        let expense = Expense {
            id: Some(id),
            title: input.title.clone(),
            amount: input.amount,
            date,
            category: input.category.clone(),
            classification_source: Some(ClassificationSource::Manual),
        };

        db.update_expense(&expense)?;
        if let Some(cat) = &input.category {
            if !cat.is_empty() {
                let pattern_source = input.rule_pattern
                    .as_deref()
                    .filter(|p| !p.is_empty())
                    .unwrap_or(&input.title);
                db.insert_rule(&ClassificationRule::from_pattern(pattern_source, cat))?;
            }
        }
        Ok(())
    })
    .map_err(|e| e.to_string())
}

#[tauri::command]
fn delete_expense(state: State<AppState>, id: i64) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.delete_expense(id).map_err(|e| e.to_string())
}

#[tauri::command]
fn delete_expenses(state: State<AppState>, ids: Vec<i64>) -> Result<usize, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.delete_expenses(&ids).map_err(|e| e.to_string())
}

// ── LLM Config Commands ──

#[tauri::command]
fn get_llm_config(state: State<AppState>) -> Result<LlmConfigOutput, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    Ok(LlmConfigOutput {
        provider: db.get_config("llm_provider").map_err(|e| e.to_string())?,
        api_key: db.get_config("llm_api_key").map_err(|e| e.to_string())?,
    })
}

#[tauri::command]
fn save_llm_config(state: State<AppState>, config: LlmConfigInput) -> Result<(), String> {
    // Validate before saving
    let provider = create_provider(&config.provider)
        .ok_or_else(|| format!("Unknown provider: {}", config.provider))?;
    let llm_config = LlmConfig {
        provider: config.provider.clone(),
        api_key: config.api_key.clone(),
    };
    provider.validate(&llm_config).map_err(|e| e.to_string())?;

    // Save atomically — both keys in one transaction
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.with_transaction(|| {
        db.set_config("llm_provider", &config.provider)?;
        db.set_config("llm_api_key", &config.api_key)?;
        Ok(())
    })
    .map_err(|e| e.to_string())
}

#[tauri::command]
fn validate_llm_config(config: LlmConfigInput) -> Result<(), String> {
    let provider = create_provider(&config.provider)
        .ok_or_else(|| format!("Unknown provider: {}", config.provider))?;
    let llm_config = LlmConfig {
        provider: config.provider,
        api_key: config.api_key,
    };
    provider.validate(&llm_config).map_err(|e| e.to_string())
}

#[tauri::command]
fn clear_llm_config(state: State<AppState>) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.with_transaction(|| {
        db.set_config("llm_provider", "")?;
        db.set_config("llm_api_key", "")?;
        Ok(())
    })
    .map_err(|e| e.to_string())
}

// ── Category Suggestion ──

#[tauri::command]
fn suggest_category(state: State<AppState>, title: String) -> Result<Option<String>, String> {
    if title.trim().is_empty() {
        return Ok(None);
    }
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let rules = db.get_all_rules().map_err(|e| e.to_string())?;
    let classifier = RegexClassifier::from_rules(&rules);
    let parsed = ParsedExpense {
        title,
        amount: 0.0,
        date: chrono::Local::now().date_naive(),
    };
    match classifier.classify(&parsed) {
        Ok(Some(result)) => Ok(Some(result.category)),
        _ => Ok(None),
    }
}

// ── Parsing Commands ──

#[tauri::command]
fn preview_csv(input: String) -> Result<PreviewResult, String> {
    let parsers = parsers::builtin_parsers();
    let parser = parsers::detect_parser(&input, &parsers)
        .ok_or("Could not detect input format. Supported: CSV (comma, semicolon, tab delimited).")?;

    let rows = parser.preview_rows(&input).map_err(|e| e.to_string())?;

    Ok(PreviewResult {
        parser_name: parser.name().to_string(),
        rows,
    })
}

#[tauri::command]
fn parse_and_classify(
    state: State<AppState>,
    input: String,
    mapping: ColumnMapping,
) -> Result<Vec<ClassifiedExpenseRow>, String> {
    // Phase 1: Parse CSV (no DB needed)
    let parsers = parsers::builtin_parsers();
    let parser = parsers::detect_parser(&input, &parsers)
        .ok_or("Could not detect input format.")?;
    let parsed = parser.parse(&input, &mapping).map_err(|e| e.to_string())?;

    // Phase 2: Read all needed DB data in a single lock, then release
    let (rules, llm_provider_name, llm_api_key, categories, dup_flags) = {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        let rules = db.get_all_rules().map_err(|e| e.to_string())?;
        let llm_provider_name = db.get_config("llm_provider").map_err(|e| e.to_string())?;
        let llm_api_key = db.get_config("llm_api_key").map_err(|e| e.to_string())?;
        let categories = db.get_all_categories().unwrap_or_default();
        let dup_inputs: Vec<(&str, f64, &chrono::NaiveDate)> = parsed
            .iter()
            .map(|e| (e.title.as_str(), e.amount, &e.date))
            .collect();
        let dup_flags = db.check_duplicates_batch(&dup_inputs).map_err(|e| e.to_string())?;
        (rules, llm_provider_name, llm_api_key, categories, dup_flags)
    }; // mutex released here

    // Phase 3: Classify with regex rules (no DB needed)
    let regex_classifier = RegexClassifier::from_rules(&rules);
    let classifiers: Vec<Box<dyn Classifier>> = vec![Box::new(regex_classifier)];
    let classified = classify_pipeline(&parsed, &classifiers);

    // Phase 4: Build result rows
    let mut rows: Vec<ClassifiedExpenseRow> = Vec::new();
    for ((expense, class_result), &is_dup) in classified.iter().zip(dup_flags.iter()) {
        let (category, source, confidence) = match class_result {
            Some(cr) => (Some(cr.category.clone()), Some(cr.source.to_string()), Some(cr.confidence)),
            None => (None, None, None),
        };
        rows.push(ClassifiedExpenseRow {
            title: expense.title.clone(),
            amount: expense.amount,
            date: expense.date.to_string(),
            category,
            source,
            confidence,
            is_duplicate: is_dup,
        });
    }

    // Phase 5: LLM fallback (no DB lock held during HTTP calls)
    if let (Some(provider_name), Some(api_key)) = (&llm_provider_name, &llm_api_key) {
        if !provider_name.is_empty() && !api_key.is_empty() {
            let unclassified_indices: Vec<usize> = rows
                .iter()
                .enumerate()
                .filter(|(_, r)| r.category.is_none() && !r.is_duplicate)
                .map(|(i, _)| i)
                .collect();

            if !unclassified_indices.is_empty() {
                if let Some(provider) = create_provider(provider_name) {
                    let config = LlmConfig {
                        provider: provider_name.clone(),
                        api_key: api_key.clone(),
                    };
                    let unclassified_expenses: Vec<ParsedExpense> = unclassified_indices
                        .iter()
                        .filter_map(|&i| {
                            let row = rows.get(i)?;
                            Some(ParsedExpense {
                                title: row.title.clone(),
                                amount: row.amount,
                                date: chrono::NaiveDate::parse_from_str(&row.date, "%Y-%m-%d")
                                    .unwrap_or_else(|_| chrono::Local::now().date_naive()),
                            })
                        })
                        .collect();

                    if let Ok(llm_results) =
                        provider.classify_batch(&unclassified_expenses, &categories, &config)
                    {
                        for (idx, llm_result) in
                            unclassified_indices.iter().zip(llm_results.into_iter())
                        {
                            if let Some(classification) = llm_result {
                                if let Some(row) = rows.get_mut(*idx) {
                                    row.category = Some(classification.category);
                                    row.source = Some(ClassificationSource::Llm.to_string());
                                    row.confidence = Some(classification.confidence);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(rows)
}

// ── Export ──

#[derive(Serialize, Deserialize)]
pub struct ExportColumnsInput {
    pub date: bool,
    pub title: bool,
    pub amount: bool,
    pub category: bool,
    pub classification_source: bool,
}

#[tauri::command]
fn export_expenses(
    state: State<AppState>,
    columns: ExportColumnsInput,
    path: String,
) -> Result<(), String> {
    use accountant_core::exporters::{CsvExporter, ExportColumns, Exporter};

    let export_columns = ExportColumns {
        date: columns.date,
        title: columns.title,
        amount: columns.amount,
        category: columns.category,
        classification_source: columns.classification_source,
    };

    // Fetch data under lock, then release before disk I/O
    let bytes = {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        let expenses = db.get_all_expenses().map_err(|e| e.to_string())?;
        let exporter = CsvExporter;
        exporter.export(&expenses, &export_columns).map_err(|e| e.to_string())?
    };

    std::fs::write(&path, &bytes).map_err(|e| format!("Failed to write file: {}", e))
}

// ── Bulk Save ──

#[tauri::command]
fn bulk_save_expenses(
    state: State<AppState>,
    expenses: Vec<BulkSaveExpense>,
    filename: Option<String>,
) -> Result<usize, String> {
    // Build all expenses first (pure computation, no lock needed), fail fast on invalid data
    let mut to_insert: Vec<Expense> = Vec::with_capacity(expenses.len());
    let mut rules: Vec<ClassificationRule> = Vec::new();

    for e in &expenses {
        let date = parse_date(&e.date)?;
        let source = e
            .source
            .as_deref()
            .and_then(ClassificationSource::from_str_opt)
            .unwrap_or(ClassificationSource::Manual);

        to_insert.push(Expense {
            id: None,
            title: e.title.clone(),
            amount: e.amount,
            date,
            category: e.category.clone(),
            classification_source: Some(source),
        });

        if let Some(ref cat) = e.category {
            if !cat.is_empty() {
                let pattern_source = e.rule_pattern
                    .as_deref()
                    .filter(|p| !p.is_empty())
                    .unwrap_or(&e.title);
                rules.push(ClassificationRule::from_pattern(pattern_source, cat));
            }
        }
    }

    // Acquire lock only for the DB insert
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let saved = db
        .insert_expenses_bulk(&to_insert, filename.as_deref(), &rules)
        .map_err(|e| e.to_string())?;

    Ok(saved)
}

// ── Upload Batch Management ──

#[tauri::command]
fn get_upload_batches(state: State<AppState>) -> Result<Vec<UploadBatch>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.get_upload_batches().map_err(|e| e.to_string())
}

#[tauri::command]
fn delete_batch(state: State<AppState>, batch_id: i64) -> Result<usize, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.delete_batch(batch_id).map_err(|e| e.to_string())
}

// ── Category Management ──

#[tauri::command]
fn get_category_stats(state: State<AppState>) -> Result<Vec<CategoryStats>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.get_category_stats().map_err(|e| e.to_string())
}

#[tauri::command]
fn create_category(state: State<AppState>, name: String) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    if db.category_exists(&name).map_err(|e| e.to_string())? {
        return Err(format!("Category '{}' already exists", name));
    }
    db.create_category(&name).map_err(|e| e.to_string())
}

#[tauri::command]
fn rename_category(state: State<AppState>, old_name: String, new_name: String) -> Result<(), String> {
    if old_name == new_name {
        return Ok(());
    }
    let db = state.db.lock().map_err(|e| e.to_string())?;
    if old_name.to_lowercase() != new_name.to_lowercase()
        && db.category_exists(&new_name).map_err(|e| e.to_string())?
    {
        return Err(format!("Category '{}' already exists", new_name));
    }
    db.rename_category(&old_name, &new_name).map_err(|e| e.to_string())
}

#[tauri::command]
fn delete_category(state: State<AppState>, category: String, replacement: String) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.delete_category(&category, &replacement).map_err(|e| e.to_string())
}

#[tauri::command]
fn merge_categories(state: State<AppState>, sources: Vec<String>, target: String) -> Result<(), String> {
    if sources.is_empty() || target.is_empty() {
        return Err("Sources and target must not be empty".to_string());
    }
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.merge_categories(&sources, &target).map_err(|e| e.to_string())
}

// ── Title Cleanup ──

#[derive(Serialize, Deserialize)]
pub struct TitleCleanupPreview {
    pub expense_id: i64,
    pub original: String,
    pub cleaned: String,
}

#[tauri::command]
fn get_title_cleanup_rules(state: State<AppState>) -> Result<Vec<TitleCleanupRule>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.get_all_title_cleanup_rules().map_err(|e| e.to_string())
}

#[tauri::command]
fn save_title_cleanup_rule(state: State<AppState>, rule: TitleCleanupRule) -> Result<i64, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    if let Some(id) = rule.id {
        db.update_title_cleanup_rule(&rule).map_err(|e| e.to_string())?;
        Ok(id)
    } else {
        db.insert_title_cleanup_rule(&rule).map_err(|e| e.to_string())
    }
}

#[tauri::command]
fn delete_title_cleanup_rule(state: State<AppState>, id: i64) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.delete_title_cleanup_rule(id).map_err(|e| e.to_string())
}

#[tauri::command]
fn preview_title_cleanup(state: State<AppState>, rule: TitleCleanupRule) -> Result<Vec<TitleCleanupPreview>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let results = db.preview_title_cleanup(&rule).map_err(|e| e.to_string())?;
    Ok(results
        .into_iter()
        .map(|(expense_id, original, cleaned)| TitleCleanupPreview {
            expense_id,
            original,
            cleaned,
        })
        .collect())
}

#[tauri::command]
fn apply_title_cleanup(state: State<AppState>, rule_id: i64, expense_ids: Vec<i64>) -> Result<usize, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let rule = db.get_title_cleanup_rule(rule_id).map_err(|e| e.to_string())?;
    db.apply_title_cleanup(&rule, &expense_ids).map_err(|e| e.to_string())
}

// ── Budget Planning ──

#[derive(Debug, Serialize, Deserialize)]
pub struct BudgetCategoryInput {
    pub category: String,
    pub amount: f64,
}

#[derive(Serialize, Deserialize)]
pub struct PlannedExpenseInput {
    pub title: String,
    pub amount: f64,
    pub date: String,
    pub category: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BudgetSummaryOutput {
    pub budget_id: i64,
    pub start_date: String,
    pub end_date: String,
    pub categories: Vec<BudgetCategoryStatus>,
    pub budget_categories: Vec<BudgetCategoryInput>,
    pub planned_expenses: Vec<PlannedExpense>,
    pub calendar_events: Vec<CalendarEvent>,
    pub total_budgeted: f64,
    pub total_spent: f64,
    pub total_planned: f64,
    pub total_calendar: f64,
}

fn build_budget_summary(db: &Database, budget: &Budget) -> Result<BudgetSummaryOutput, String> {
    let budget_id = budget.id.ok_or_else(|| "Budget has no id".to_string())?;
    let budget_cats = db.get_budget_categories(budget_id).map_err(|e| e.to_string())?;
    let planned = db.get_planned_expenses(budget_id).map_err(|e| e.to_string())?;
    let cal_events = db.get_calendar_events(budget_id).map_err(|e| e.to_string())?;
    let actual_expenses = db
        .get_expenses_for_date_range(budget.start_date, budget.end_date)
        .map_err(|e| e.to_string())?;

    let mut category_statuses: Vec<BudgetCategoryStatus> = Vec::new();
    let mut total_budgeted = 0.0;
    let mut total_spent = 0.0;

    for bc in &budget_cats {
        let spent: f64 = actual_expenses
            .iter()
            .filter(|e| e.category.as_deref() == Some(&bc.category))
            .map(|e| e.amount)
            .sum();

        let ratio = if bc.amount > 0.0 {
            spent / bc.amount
        } else {
            0.0
        };

        total_budgeted += bc.amount;
        total_spent += spent;

        category_statuses.push(BudgetCategoryStatus {
            category: bc.category.clone(),
            budgeted: bc.amount,
            spent,
            status: BudgetStatus::from_ratio(ratio),
        });
    }

    let total_planned: f64 = planned.iter().map(|p| p.amount).sum();
    let total_calendar: f64 = cal_events.iter().filter_map(|e| e.amount).sum();

    let budget_category_inputs: Vec<BudgetCategoryInput> = budget_cats
        .iter()
        .map(|bc| BudgetCategoryInput {
            category: bc.category.clone(),
            amount: bc.amount,
        })
        .collect();

    Ok(BudgetSummaryOutput {
        budget_id,
        start_date: budget.start_date.to_string(),
        end_date: budget.end_date.to_string(),
        categories: category_statuses,
        budget_categories: budget_category_inputs,
        planned_expenses: planned,
        calendar_events: cal_events,
        total_budgeted,
        total_spent,
        total_planned,
        total_calendar,
    })
}

#[tauri::command]
fn get_budget_summary(
    state: State<AppState>,
    budget_id: i64,
) -> Result<BudgetSummaryOutput, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let budget = db
        .get_budget_by_id(budget_id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("Budget with id {} not found", budget_id))?;
    build_budget_summary(&db, &budget)
}

#[tauri::command]
fn get_active_budget_summary(
    state: State<AppState>,
) -> Result<Option<BudgetSummaryOutput>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    match db.get_active_budget().map_err(|e| e.to_string())? {
        Some(budget) => Ok(Some(build_budget_summary(&db, &budget)?)),
        None => Ok(None),
    }
}

#[tauri::command]
fn create_budget(
    state: State<AppState>,
    start_date: String,
    end_date: String,
    categories: Vec<BudgetCategoryInput>,
) -> Result<i64, String> {
    let start = parse_date(&start_date)?;
    let end = parse_date(&end_date)?;
    let db = state.db.lock().map_err(|e| e.to_string())?;

    // budget_id is not known upfront, so use 0 as placeholder — create_budget_with_categories
    // assigns the real budget_id internally
    let cats: Vec<BudgetCategory> = categories
        .into_iter()
        .map(|c| BudgetCategory {
            id: None,
            budget_id: 0, // placeholder, overridden by create_budget_with_categories
            category: c.category,
            amount: c.amount,
        })
        .collect();

    db.create_budget_with_categories(start, end, &cats)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn save_budget_categories(
    state: State<AppState>,
    budget_id: i64,
    categories: Vec<BudgetCategoryInput>,
) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;

    let cats: Vec<BudgetCategory> = categories
        .into_iter()
        .map(|c| BudgetCategory {
            id: None,
            budget_id,
            category: c.category,
            amount: c.amount,
        })
        .collect();

    db.save_budget_categories(budget_id, &cats)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn add_planned_expense(
    state: State<AppState>,
    budget_id: i64,
    expense: PlannedExpenseInput,
) -> Result<i64, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let date = parse_date(&expense.date)?;

    let pe = PlannedExpense {
        id: None,
        budget_id,
        title: expense.title,
        amount: expense.amount,
        date,
        category: expense.category,
    };

    db.insert_planned_expense(&pe).map_err(|e| e.to_string())
}

#[tauri::command]
fn delete_planned_expense(state: State<AppState>, id: i64) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.delete_planned_expense(id).map_err(|e| e.to_string())
}

#[tauri::command]
fn delete_budget(state: State<AppState>, id: i64) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.delete_budget(id).map_err(|e| e.to_string())
}

#[tauri::command]
fn import_calendar_events(
    state: State<AppState>,
    budget_id: i64,
    ics_content: String,
) -> Result<usize, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let budget = db
        .get_budget_by_id(budget_id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("Budget with id {} not found", budget_id))?;

    let all_events =
        accountant_core::ical::parse_ics(&ics_content).map_err(|e| e.to_string())?;
    let filtered = accountant_core::ical::filter_events_by_date_range(
        &all_events,
        budget.start_date,
        budget.end_date,
    );

    let cal_events: Vec<CalendarEvent> = filtered
        .into_iter()
        .map(|e| CalendarEvent {
            id: None,
            budget_id,
            summary: e.summary,
            description: e.description,
            location: e.location,
            start_date: e.start_date,
            end_date: e.end_date,
            all_day: e.all_day,
            amount: None,
        })
        .collect();

    db.save_calendar_events(budget_id, &cal_events)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn update_calendar_event_amount(
    state: State<AppState>,
    event_id: i64,
    amount: Option<f64>,
) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.update_calendar_event_amount(event_id, amount)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn check_budget_overlap(
    state: State<AppState>,
    start_date: String,
    end_date: String,
) -> Result<bool, String> {
    let start = parse_date(&start_date)?;
    let end = parse_date(&end_date)?;
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.check_budget_overlap(start, end)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn get_category_averages(state: State<AppState>) -> Result<Vec<CategoryAverage>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.get_category_averages(3).map_err(|e| e.to_string())
}

// ── Dashboard Widget Config ──

#[tauri::command]
fn get_active_widgets(state: State<AppState>) -> Result<Option<Vec<String>>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let val = db.get_config("active_widgets").map_err(|e| e.to_string())?;
    match val {
        Some(json) if !json.is_empty() => {
            let ids: Vec<String> = serde_json::from_str(&json).map_err(|e| e.to_string())?;
            Ok(Some(ids))
        }
        _ => Ok(None),
    }
}

#[tauri::command]
fn save_active_widgets(state: State<AppState>, widget_ids: Vec<String>) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let json = serde_json::to_string(&widget_ids).map_err(|e| e.to_string())?;
    db.set_config("active_widgets", &json).map_err(|e| e.to_string())
}

// ── Tests ──

#[cfg(test)]
mod tests {
    use super::*;
    use tauri::Manager;

    fn app() -> tauri::App<tauri::test::MockRuntime> {
        tauri::test::mock_builder()
            .manage(AppState {
                db: Mutex::new(Database::open_memory().unwrap()),
            })
            .build(tauri::test::mock_context(tauri::test::noop_assets()))
            .unwrap()
    }

    // ── Expense CRUD ──

    #[test]
    fn add_and_get_expense() {
        let app = app();
        let state: State<AppState> = app.state();
        let id = add_expense(
            state,
            ExpenseInput {
                title: "Coffee".into(),
                amount: 3.50,
                date: "2024-01-15".into(),
                category: Some("Food".into()),
                rule_pattern: None,
            },
        )
        .unwrap();
        assert!(id > 0);

        let state: State<AppState> = app.state();
        let all = get_expenses(state).unwrap();
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].title, "Coffee");
        assert_eq!(all[0].amount, 3.50);
        assert_eq!(all[0].category.as_deref(), Some("Food"));
    }

    #[test]
    fn add_expense_invalid_date() {
        let app = app();
        let state: State<AppState> = app.state();
        let err = add_expense(
            state,
            ExpenseInput {
                title: "X".into(),
                amount: 1.0,
                date: "not-a-date".into(),
                category: None,
                rule_pattern: None,
            },
        )
        .unwrap_err();
        assert!(err.contains("Invalid date"));
    }

    #[test]
    fn update_expense_happy_path() {
        let app = app();
        let state: State<AppState> = app.state();
        let id = add_expense(
            state,
            ExpenseInput {
                title: "Old".into(),
                amount: 10.0,
                date: "2024-02-01".into(),
                category: None,
                rule_pattern: None,
            },
        )
        .unwrap();

        let state: State<AppState> = app.state();
        update_expense(
            state,
            id,
            ExpenseInput {
                title: "New".into(),
                amount: 20.0,
                date: "2024-02-02".into(),
                category: Some("Transport".into()),
                rule_pattern: None,
            },
        )
        .unwrap();

        let state: State<AppState> = app.state();
        let all = get_expenses(state).unwrap();
        assert_eq!(all[0].title, "New");
        assert_eq!(all[0].amount, 20.0);
    }

    #[test]
    fn update_expense_invalid_date() {
        let app = app();
        let state: State<AppState> = app.state();
        let id = add_expense(
            state,
            ExpenseInput {
                title: "X".into(),
                amount: 1.0,
                date: "2024-01-01".into(),
                category: None,
                rule_pattern: None,
            },
        )
        .unwrap();

        let state: State<AppState> = app.state();
        let err = update_expense(
            state,
            id,
            ExpenseInput {
                title: "X".into(),
                amount: 1.0,
                date: "bad".into(),
                category: None,
                rule_pattern: None,
            },
        )
        .unwrap_err();
        assert!(err.contains("Invalid date"));
    }

    #[test]
    fn delete_single_expense() {
        let app = app();
        let state: State<AppState> = app.state();
        let id = add_expense(
            state,
            ExpenseInput {
                title: "Gone".into(),
                amount: 5.0,
                date: "2024-03-01".into(),
                category: None,
                rule_pattern: None,
            },
        )
        .unwrap();

        let state: State<AppState> = app.state();
        delete_expense(state, id).unwrap();

        let state: State<AppState> = app.state();
        assert!(get_expenses(state).unwrap().is_empty());
    }

    #[test]
    fn delete_expense_nonexistent() {
        let app = app();
        let state: State<AppState> = app.state();
        let err = delete_expense(state, 99999).unwrap_err();
        assert!(err.contains("not found"));
    }

    #[test]
    fn delete_multiple_expenses() {
        let app = app();
        for i in 0..3 {
            let state: State<AppState> = app.state();
            add_expense(
                state,
                ExpenseInput {
                    title: format!("E{}", i),
                    amount: 1.0,
                    date: "2024-01-01".into(),
                    category: None,
                    rule_pattern: None,
                },
            )
            .unwrap();
        }

        let state: State<AppState> = app.state();
        let all = get_expenses(state).unwrap();
        let ids: Vec<i64> = all.iter().map(|e| e.id.unwrap()).collect();

        let state: State<AppState> = app.state();
        let deleted = delete_expenses(state, ids[..2].to_vec()).unwrap();
        assert_eq!(deleted, 2);

        let state: State<AppState> = app.state();
        let remaining = get_expenses(state).unwrap();
        assert_eq!(remaining.len(), 1);
    }

    // ── Query Expenses ──

    #[test]
    fn query_expenses_search_and_pagination() {
        let app = app();
        let items = vec![
            ("Coffee shop", 5.0, "Food"),
            ("Bus ticket", 2.5, "Transport"),
            ("Coffee beans", 12.0, "Food"),
        ];
        for (title, amount, cat) in items {
            let state: State<AppState> = app.state();
            add_expense(
                state,
                ExpenseInput {
                    title: title.into(),
                    amount,
                    date: "2024-01-15".into(),
                    category: Some(cat.into()),
                    rule_pattern: None,
                },
            )
            .unwrap();
        }

        // Search
        let state: State<AppState> = app.state();
        let result = query_expenses(
            state,
            ExpenseQuery {
                search: Some("Coffee".into()),
                category: None,
                date_from: None,
                date_to: None,
                amount_min: None,
                amount_max: None,
                limit: None,
                offset: None,
            },
        )
        .unwrap();
        assert_eq!(result.total_count, 2);

        // Category filter
        let state: State<AppState> = app.state();
        let result = query_expenses(
            state,
            ExpenseQuery {
                search: None,
                category: Some("Transport".into()),
                date_from: None,
                date_to: None,
                amount_min: None,
                amount_max: None,
                limit: None,
                offset: None,
            },
        )
        .unwrap();
        assert_eq!(result.total_count, 1);
        assert_eq!(result.expenses[0].title, "Bus ticket");

        // Pagination
        let state: State<AppState> = app.state();
        let result = query_expenses(
            state,
            ExpenseQuery {
                search: None,
                category: None,
                date_from: None,
                date_to: None,
                amount_min: None,
                amount_max: None,
                limit: Some(1),
                offset: Some(0),
            },
        )
        .unwrap();
        assert_eq!(result.expenses.len(), 1);
        assert_eq!(result.total_count, 3);
    }

    // ── Categories ──

    #[test]
    fn category_lifecycle() {
        let app = app();

        // Add expense to create a category implicitly
        let state: State<AppState> = app.state();
        add_expense(
            state,
            ExpenseInput {
                title: "X".into(),
                amount: 1.0,
                date: "2024-01-01".into(),
                category: Some("Food".into()),
                rule_pattern: None,
            },
        )
        .unwrap();

        let state: State<AppState> = app.state();
        let cats = get_categories(state).unwrap();
        assert!(cats.contains(&"Food".to_string()));

        // Create standalone category
        let state: State<AppState> = app.state();
        create_category(state, "Entertainment".into()).unwrap();

        // Duplicate should fail
        let state: State<AppState> = app.state();
        let err = create_category(state, "Entertainment".into()).unwrap_err();
        assert!(err.contains("already exists"));

        // Rename
        let state: State<AppState> = app.state();
        rename_category(state, "Entertainment".into(), "Fun".into()).unwrap();

        // Rename to existing should fail
        let state: State<AppState> = app.state();
        let err = rename_category(state, "Fun".into(), "Food".into()).unwrap_err();
        assert!(err.contains("already exists"));

        // Rename same casing is ok
        let state: State<AppState> = app.state();
        rename_category(state, "Fun".into(), "fun".into()).unwrap();
    }

    #[test]
    fn merge_categories_happy_path() {
        let app = app();
        for (title, cat) in [("A", "Cat1"), ("B", "Cat2"), ("C", "Cat3")] {
            let state: State<AppState> = app.state();
            add_expense(
                state,
                ExpenseInput {
                    title: title.into(),
                    amount: 1.0,
                    date: "2024-01-01".into(),
                    category: Some(cat.into()),
                    rule_pattern: None,
                },
            )
            .unwrap();
        }

        let state: State<AppState> = app.state();
        merge_categories(state, vec!["Cat1".into(), "Cat2".into()], "Cat3".into()).unwrap();

        let state: State<AppState> = app.state();
        let result = query_expenses(
            state,
            ExpenseQuery {
                search: None,
                category: Some("Cat3".into()),
                date_from: None,
                date_to: None,
                amount_min: None,
                amount_max: None,
                limit: None,
                offset: None,
            },
        )
        .unwrap();
        assert_eq!(result.total_count, 3);
    }

    #[test]
    fn merge_categories_empty_sources_fails() {
        let app = app();
        let state: State<AppState> = app.state();
        let err = merge_categories(state, vec![], "Target".into()).unwrap_err();
        assert!(err.contains("must not be empty"));
    }

    // ── Suggest Category ──

    #[test]
    fn suggest_category_with_rule() {
        let app = app();
        // Add an expense with category — this auto-creates a rule
        let state: State<AppState> = app.state();
        add_expense(
            state,
            ExpenseInput {
                title: "Starbucks".into(),
                amount: 5.0,
                date: "2024-01-01".into(),
                category: Some("Coffee".into()),
                rule_pattern: None,
            },
        )
        .unwrap();

        let state: State<AppState> = app.state();
        let suggestion = suggest_category(state, "Starbucks downtown".into()).unwrap();
        assert_eq!(suggestion, Some("Coffee".to_string()));
    }

    #[test]
    fn suggest_category_empty_title() {
        let app = app();
        let state: State<AppState> = app.state();
        let result = suggest_category(state, "".into()).unwrap();
        assert_eq!(result, None);
    }

    #[test]
    fn suggest_category_no_match() {
        let app = app();
        let state: State<AppState> = app.state();
        let result = suggest_category(state, "xyzzy".into()).unwrap();
        assert_eq!(result, None);
    }

    // ── Bulk Save + Batches ──

    #[test]
    fn bulk_save_and_batch_undo() {
        let app = app();
        let expenses = vec![
            BulkSaveExpense {
                title: "Item1".into(),
                amount: 10.0,
                date: "2024-01-01".into(),
                category: Some("Food".into()),
                source: Some("Manual".into()),
                rule_pattern: None,
            },
            BulkSaveExpense {
                title: "Item2".into(),
                amount: 20.0,
                date: "2024-01-02".into(),
                category: None,
                source: None,
                rule_pattern: None,
            },
        ];

        let state: State<AppState> = app.state();
        let saved = bulk_save_expenses(state, expenses, Some("test.csv".into())).unwrap();
        assert_eq!(saved, 2);

        // Check batches
        let state: State<AppState> = app.state();
        let batches = get_upload_batches(state).unwrap();
        assert_eq!(batches.len(), 1);
        assert_eq!(batches[0].filename, Some("test.csv".to_string()));
        assert_eq!(batches[0].expense_count, 2);

        // Delete batch (undo)
        let state: State<AppState> = app.state();
        let deleted = delete_batch(state, batches[0].id).unwrap();
        assert_eq!(deleted, 2);

        let state: State<AppState> = app.state();
        assert!(get_expenses(state).unwrap().is_empty());
    }

    #[test]
    fn bulk_save_invalid_date_fails() {
        let app = app();
        let expenses = vec![BulkSaveExpense {
            title: "Bad".into(),
            amount: 1.0,
            date: "nope".into(),
            category: None,
            source: None,
            rule_pattern: None,
        }];

        let state: State<AppState> = app.state();
        let err = bulk_save_expenses(state, expenses, None).unwrap_err();
        assert!(err.contains("Invalid date"));
    }

    // ── CSV Preview + Parse ──

    #[test]
    fn preview_csv_happy_path() {
        let csv = "date,title,amount\n2024-01-01,Coffee,3.50\n2024-01-02,Bus,2.00\n";
        let result = preview_csv(csv.into()).unwrap();
        assert_eq!(result.parser_name, "CSV");
        assert!(!result.rows.is_empty());
    }

    #[test]
    fn preview_csv_unrecognized_format() {
        let err = preview_csv("not a csv at all".into()).unwrap_err();
        assert!(err.contains("Could not detect"));
    }

    #[test]
    fn parse_and_classify_happy_path() {
        let app = app();
        let csv = "date,title,amount\n2024-01-01,Coffee,3.50\n2024-01-02,Bus,2.00\n";
        let mapping = ColumnMapping {
            title_index: 1,
            amount_index: 2,
            date_index: 0,
            date_format: "%Y-%m-%d".into(),
        };

        let state: State<AppState> = app.state();
        let rows = parse_and_classify(state, csv.into(), mapping).unwrap();
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].title, "Coffee");
        assert_eq!(rows[0].amount, 3.50);
        assert!(!rows[0].is_duplicate);
    }

    #[test]
    fn parse_and_classify_with_existing_rule() {
        let app = app();

        // Create rule by adding a categorized expense
        let state: State<AppState> = app.state();
        add_expense(
            state,
            ExpenseInput {
                title: "Coffee".into(),
                amount: 5.0,
                date: "2024-06-01".into(),
                category: Some("Drinks".into()),
                rule_pattern: None,
            },
        )
        .unwrap();

        let csv = "date,title,amount\n2024-01-01,Coffee,3.50\n";
        let mapping = ColumnMapping {
            title_index: 1,
            amount_index: 2,
            date_index: 0,
            date_format: "%Y-%m-%d".into(),
        };

        let state: State<AppState> = app.state();
        let rows = parse_and_classify(state, csv.into(), mapping).unwrap();
        assert_eq!(rows[0].category.as_deref(), Some("Drinks"));
        assert_eq!(rows[0].source.as_deref(), Some("Database"));
    }

    #[test]
    fn parse_and_classify_detects_duplicates() {
        let app = app();
        let state: State<AppState> = app.state();
        add_expense(
            state,
            ExpenseInput {
                title: "Coffee".into(),
                amount: 3.50,
                date: "2024-01-01".into(),
                category: None,
                rule_pattern: None,
            },
        )
        .unwrap();

        let csv = "date,title,amount\n2024-01-01,Coffee,3.50\n";
        let mapping = ColumnMapping {
            title_index: 1,
            amount_index: 2,
            date_index: 0,
            date_format: "%Y-%m-%d".into(),
        };

        let state: State<AppState> = app.state();
        let rows = parse_and_classify(state, csv.into(), mapping).unwrap();
        assert!(rows[0].is_duplicate);
    }

    // ── Export ──

    #[test]
    fn export_expenses_to_file() {
        let app = app();
        let state: State<AppState> = app.state();
        add_expense(
            state,
            ExpenseInput {
                title: "Test".into(),
                amount: 42.0,
                date: "2024-01-01".into(),
                category: Some("Misc".into()),
                rule_pattern: None,
            },
        )
        .unwrap();

        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("export.csv");

        let state: State<AppState> = app.state();
        export_expenses(
            state,
            ExportColumnsInput {
                date: true,
                title: true,
                amount: true,
                category: true,
                classification_source: false,
            },
            path.to_str().unwrap().into(),
        )
        .unwrap();

        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.contains("Test"));
        assert!(content.contains("42"));
    }

    // ── Title Cleanup ──

    #[test]
    fn title_cleanup_lifecycle() {
        let app = app();

        // Add expense
        let state: State<AppState> = app.state();
        add_expense(
            state,
            ExpenseInput {
                title: "CARD *1234 Coffee Shop".into(),
                amount: 5.0,
                date: "2024-01-01".into(),
                category: None,
                rule_pattern: None,
            },
        )
        .unwrap();

        // Create rule
        let rule = TitleCleanupRule {
            id: None,
            pattern: "CARD *1234 ".into(),
            replacement: "".into(),
            is_regex: false,
        };
        let state: State<AppState> = app.state();
        let rule_id = save_title_cleanup_rule(state, rule).unwrap();
        assert!(rule_id > 0);

        // List rules
        let state: State<AppState> = app.state();
        let rules = get_title_cleanup_rules(state).unwrap();
        assert_eq!(rules.len(), 1);

        // Preview
        let preview_rule = TitleCleanupRule {
            id: Some(rule_id),
            pattern: "CARD *1234 ".into(),
            replacement: "".into(),
            is_regex: false,
        };
        let state: State<AppState> = app.state();
        let previews = preview_title_cleanup(state, preview_rule).unwrap();
        assert_eq!(previews.len(), 1);
        assert_eq!(previews[0].cleaned, "Coffee Shop");

        // Apply
        let expense_ids: Vec<i64> = previews.iter().map(|p| p.expense_id).collect();
        let state: State<AppState> = app.state();
        let applied = apply_title_cleanup(state, rule_id, expense_ids).unwrap();
        assert_eq!(applied, 1);

        // Verify
        let state: State<AppState> = app.state();
        let all = get_expenses(state).unwrap();
        assert_eq!(all[0].title, "Coffee Shop");

        // Delete rule
        let state: State<AppState> = app.state();
        delete_title_cleanup_rule(state, rule_id).unwrap();

        let state: State<AppState> = app.state();
        assert!(get_title_cleanup_rules(state).unwrap().is_empty());
    }

    // ── Budget Planning ──

    #[test]
    fn budget_lifecycle() {
        let app = app();

        // Use dates that span "today" so get_active_budget works
        let today = chrono::Local::now().date_naive();
        let start = today.format("%Y-%m-%d").to_string();
        let end = (today + chrono::Duration::days(30)).format("%Y-%m-%d").to_string();
        let mid = (today + chrono::Duration::days(15)).format("%Y-%m-%d").to_string();

        // Create budget
        let state: State<AppState> = app.state();
        let budget_id = create_budget(
            state,
            start.clone(),
            end.clone(),
            vec![
                BudgetCategoryInput {
                    category: "Food".into(),
                    amount: 500.0,
                },
                BudgetCategoryInput {
                    category: "Transport".into(),
                    amount: 200.0,
                },
            ],
        )
        .unwrap();
        assert!(budget_id > 0);

        // Get summary
        let state: State<AppState> = app.state();
        let summary = get_budget_summary(state, budget_id).unwrap();
        assert_eq!(summary.budget_id, budget_id);
        assert_eq!(summary.categories.len(), 2);
        assert_eq!(summary.total_budgeted, 700.0);
        assert_eq!(summary.total_spent, 0.0);

        // Active budget
        let state: State<AppState> = app.state();
        let active = get_active_budget_summary(state).unwrap();
        assert!(active.is_some());

        // Add planned expense
        let state: State<AppState> = app.state();
        let pe_id = add_planned_expense(
            state,
            budget_id,
            PlannedExpenseInput {
                title: "Groceries".into(),
                amount: 50.0,
                date: mid,
                category: Some("Food".into()),
            },
        )
        .unwrap();

        let state: State<AppState> = app.state();
        let summary = get_budget_summary(state, budget_id).unwrap();
        assert_eq!(summary.total_planned, 50.0);
        assert_eq!(summary.planned_expenses.len(), 1);

        // Delete planned expense
        let state: State<AppState> = app.state();
        delete_planned_expense(state, pe_id).unwrap();

        // Update categories
        let state: State<AppState> = app.state();
        save_budget_categories(
            state,
            budget_id,
            vec![BudgetCategoryInput {
                category: "Food".into(),
                amount: 600.0,
            }],
        )
        .unwrap();

        let state: State<AppState> = app.state();
        let summary = get_budget_summary(state, budget_id).unwrap();
        assert_eq!(summary.categories.len(), 1);
        assert_eq!(summary.total_budgeted, 600.0);

        // Delete budget
        let state: State<AppState> = app.state();
        delete_budget(state, budget_id).unwrap();

        let state: State<AppState> = app.state();
        let active = get_active_budget_summary(state).unwrap();
        assert!(active.is_none());
    }

    #[test]
    fn budget_summary_nonexistent() {
        let app = app();
        let state: State<AppState> = app.state();
        let err = get_budget_summary(state, 99999).unwrap_err();
        assert!(err.contains("not found"));
    }

    #[test]
    fn budget_overlap_check() {
        let app = app();
        let state: State<AppState> = app.state();
        create_budget(state, "2024-01-01".into(), "2024-01-31".into(), vec![]).unwrap();

        // Overlapping range
        let state: State<AppState> = app.state();
        let overlaps = check_budget_overlap(state, "2024-01-15".into(), "2024-02-15".into()).unwrap();
        assert!(overlaps);

        // Non-overlapping range
        let state: State<AppState> = app.state();
        let overlaps = check_budget_overlap(state, "2024-02-01".into(), "2024-02-28".into()).unwrap();
        assert!(!overlaps);
    }

    #[test]
    fn budget_with_actual_spending() {
        let app = app();

        let state: State<AppState> = app.state();
        let budget_id = create_budget(
            state,
            "2024-01-01".into(),
            "2024-01-31".into(),
            vec![BudgetCategoryInput {
                category: "Food".into(),
                amount: 100.0,
            }],
        )
        .unwrap();

        // Add expense in budget range
        let state: State<AppState> = app.state();
        add_expense(
            state,
            ExpenseInput {
                title: "Lunch".into(),
                amount: 85.0,
                date: "2024-01-10".into(),
                category: Some("Food".into()),
                rule_pattern: None,
            },
        )
        .unwrap();

        let state: State<AppState> = app.state();
        let summary = get_budget_summary(state, budget_id).unwrap();
        assert_eq!(summary.total_spent, 85.0);
        assert_eq!(summary.categories[0].status, BudgetStatus::Approaching); // 85/100 = 0.85
    }

    // ── Calendar Events ──

    #[test]
    fn import_calendar_events_and_update_amount() {
        let app = app();
        let state: State<AppState> = app.state();
        let budget_id = create_budget(state, "2024-01-01".into(), "2024-01-31".into(), vec![]).unwrap();

        let ics = "BEGIN:VCALENDAR\r\n\
BEGIN:VEVENT\r\n\
DTSTART:20240115\r\n\
DTEND:20240116\r\n\
SUMMARY:Dentist\r\n\
END:VEVENT\r\n\
BEGIN:VEVENT\r\n\
DTSTART:20240601\r\n\
DTEND:20240602\r\n\
SUMMARY:Out of range\r\n\
END:VEVENT\r\n\
END:VCALENDAR";

        let state: State<AppState> = app.state();
        let count = import_calendar_events(state, budget_id, ics.into()).unwrap();
        assert_eq!(count, 1); // only the Jan event

        let state: State<AppState> = app.state();
        let summary = get_budget_summary(state, budget_id).unwrap();
        assert_eq!(summary.calendar_events.len(), 1);
        assert_eq!(summary.calendar_events[0].summary, "Dentist");
        assert_eq!(summary.calendar_events[0].amount, None);

        // Update amount
        let event_id = summary.calendar_events[0].id.unwrap();
        let state: State<AppState> = app.state();
        update_calendar_event_amount(state, event_id, Some(150.0)).unwrap();

        let state: State<AppState> = app.state();
        let summary = get_budget_summary(state, budget_id).unwrap();
        assert_eq!(summary.calendar_events[0].amount, Some(150.0));
        assert_eq!(summary.total_calendar, 150.0);
    }

    #[test]
    fn import_calendar_nonexistent_budget() {
        let app = app();
        let state: State<AppState> = app.state();
        let err = import_calendar_events(state, 99999, "BEGIN:VCALENDAR\r\nEND:VCALENDAR".into()).unwrap_err();
        assert!(err.contains("not found"));
    }

    // ── Category Averages ──

    #[test]
    fn category_averages() {
        let app = app();
        // Add expenses across recent months (get_category_averages uses last 3 months from "now")
        let today = chrono::Local::now().date_naive();
        let dates = [
            (today - chrono::Duration::days(60)).format("%Y-%m-%d").to_string(),
            (today - chrono::Duration::days(30)).format("%Y-%m-%d").to_string(),
            today.format("%Y-%m-%d").to_string(),
        ];
        for date in &dates {
            let state: State<AppState> = app.state();
            add_expense(
                state,
                ExpenseInput {
                    title: "Meal".into(),
                    amount: 100.0,
                    date: date.clone(),
                    category: Some("Food".into()),
                    rule_pattern: None,
                },
            )
            .unwrap();
        }

        let state: State<AppState> = app.state();
        let avgs = get_category_averages(state).unwrap();
        assert!(!avgs.is_empty());
        assert_eq!(avgs[0].category, "Food");
    }

    // ── Widget Config ──

    #[test]
    fn widget_config_save_and_load() {
        let app = app();

        // Initially none
        let state: State<AppState> = app.state();
        let result = get_active_widgets(state).unwrap();
        assert!(result.is_none());

        // Save
        let state: State<AppState> = app.state();
        save_active_widgets(state, vec!["total".into(), "chart".into()]).unwrap();

        // Load
        let state: State<AppState> = app.state();
        let result = get_active_widgets(state).unwrap();
        assert_eq!(result.unwrap(), vec!["total", "chart"]);
    }

    // ── LLM Config ──

    #[test]
    fn llm_config_lifecycle() {
        let app = app();

        // Initially empty
        let state: State<AppState> = app.state();
        let config = get_llm_config(state).unwrap();
        assert!(config.provider.is_none() || config.provider.as_deref() == Some(""));

        // Validate unknown provider
        let err = validate_llm_config(LlmConfigInput {
            provider: "unknown".into(),
            api_key: "key".into(),
        })
        .unwrap_err();
        assert!(err.contains("Unknown provider"));

        // save_llm_config calls provider.validate() which hits the network,
        // so we test the DB layer directly via set_config
        {
            let state: State<AppState> = app.state();
            let db = state.db.lock().unwrap();
            db.set_config("llm_provider", "ollama").unwrap();
            db.set_config("llm_api_key", "").unwrap();
        }

        let state: State<AppState> = app.state();
        let config = get_llm_config(state).unwrap();
        assert_eq!(config.provider.as_deref(), Some("ollama"));

        // Clear
        let state: State<AppState> = app.state();
        clear_llm_config(state).unwrap();

        let state: State<AppState> = app.state();
        let config = get_llm_config(state).unwrap();
        assert_eq!(config.provider.as_deref(), Some(""));
    }

    #[test]
    fn save_llm_config_unknown_provider() {
        let app = app();
        let state: State<AppState> = app.state();
        let err = save_llm_config(
            state,
            LlmConfigInput {
                provider: "nonexistent".into(),
                api_key: "x".into(),
            },
        )
        .unwrap_err();
        assert!(err.contains("Unknown provider"));
    }

    // ── Category Stats ──

    #[test]
    fn category_stats() {
        let app = app();
        for (title, cat) in [("A", "Food"), ("B", "Food"), ("C", "Transport")] {
            let state: State<AppState> = app.state();
            add_expense(
                state,
                ExpenseInput {
                    title: title.into(),
                    amount: 10.0,
                    date: "2024-01-01".into(),
                    category: Some(cat.into()),
                    rule_pattern: None,
                },
            )
            .unwrap();
        }

        let state: State<AppState> = app.state();
        let stats = get_category_stats(state).unwrap();
        assert_eq!(stats.len(), 2);
        let food = stats.iter().find(|s| s.name == "Food").unwrap();
        assert_eq!(food.expense_count, 2);
    }

    // ── Add Expense with Rule Pattern ──

    #[test]
    fn add_expense_with_custom_rule_pattern() {
        let app = app();
        let state: State<AppState> = app.state();
        add_expense(
            state,
            ExpenseInput {
                title: "CARD*1234 Starbucks NYC".into(),
                amount: 5.0,
                date: "2024-01-01".into(),
                category: Some("Coffee".into()),
                rule_pattern: Some("Starbucks".into()),
            },
        )
        .unwrap();

        // The rule should match "Starbucks" not the full title
        let state: State<AppState> = app.state();
        let suggestion = suggest_category(state, "Starbucks Seattle".into()).unwrap();
        assert_eq!(suggestion, Some("Coffee".to_string()));

        // Full noisy title should also match (Starbucks is a substring)
        let state: State<AppState> = app.state();
        let suggestion = suggest_category(state, "Another Starbucks".into()).unwrap();
        assert_eq!(suggestion, Some("Coffee".to_string()));
    }
}

// ── App Entry ──

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let db = Database::open_default().unwrap_or_else(|e| {
        eprintln!("Fatal: failed to open database: {e}");
        std::process::exit(1);
    });

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(AppState {
            db: Mutex::new(db),
        })
        .invoke_handler(tauri::generate_handler![
            get_expenses,
            query_expenses,
            add_expense,
            update_expense,
            delete_expense,
            delete_expenses,
            get_categories,
            suggest_category,
            get_llm_config,
            save_llm_config,
            validate_llm_config,
            clear_llm_config,
            preview_csv,
            parse_and_classify,
            export_expenses,
            bulk_save_expenses,
            get_upload_batches,
            delete_batch,
            get_category_stats,
            create_category,
            rename_category,
            delete_category,
            merge_categories,
            get_active_widgets,
            save_active_widgets,
            get_budget_summary,
            get_active_budget_summary,
            create_budget,
            save_budget_categories,
            add_planned_expense,
            delete_planned_expense,
            delete_budget,
            import_calendar_events,
            update_calendar_event_amount,
            check_budget_overlap,
            get_category_averages,
            get_title_cleanup_rules,
            save_title_cleanup_rule,
            delete_title_cleanup_rule,
            preview_title_cleanup,
            apply_title_cleanup,
        ])
        .run(tauri::generate_context!())
        .unwrap_or_else(|e| {
            eprintln!("Fatal: failed to start application: {e}");
            std::process::exit(1);
        });
}
