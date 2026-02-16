use accountant_core::classifiers::{classify_pipeline, Classifier, RegexClassifier};
use accountant_core::db::Database;
use accountant_core::llm::{create_provider, LlmConfig};
use accountant_core::models::{
    BudgetCategory, BudgetCategoryStatus, CalendarEvent, CategoryAverage, CategoryStats,
    ClassificationRule, ClassificationSource, Expense, ParsedExpense, PlannedExpense,
    TitleCleanupRule, UploadBatch,
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

#[derive(Serialize, Deserialize)]
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

fn save_rule_if_categorized(db: &Database, title: &str, category: &Option<String>, rule_pattern: &Option<String>) {
    if let Some(cat) = category {
        if !cat.is_empty() {
            let pattern_source = rule_pattern
                .as_deref()
                .filter(|p| !p.is_empty())
                .unwrap_or(title);
            let _ = db.insert_rule(&ClassificationRule::from_pattern(pattern_source, cat));
        }
    }
}

// ── Expense Commands ──

#[tauri::command]
fn get_expenses(state: State<AppState>) -> Result<Vec<Expense>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.get_all_expenses().map_err(|e| e.to_string())
}

#[tauri::command]
fn add_expense(state: State<AppState>, input: ExpenseInput) -> Result<i64, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let date = parse_date(&input.date)?;

    let expense = Expense {
        id: None,
        title: input.title.clone(),
        amount: input.amount,
        date,
        category: input.category.clone(),
        classification_source: Some(ClassificationSource::Manual),
    };

    let id = db.insert_expense(&expense).map_err(|e| e.to_string())?;
    save_rule_if_categorized(&db, &input.title, &input.category, &input.rule_pattern);
    Ok(id)
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

    let expense = Expense {
        id: Some(id),
        title: input.title.clone(),
        amount: input.amount,
        date,
        category: input.category.clone(),
        classification_source: Some(ClassificationSource::Manual),
    };

    db.update_expense(&expense).map_err(|e| e.to_string())?;
    save_rule_if_categorized(&db, &input.title, &input.category, &input.rule_pattern);
    Ok(())
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

    // Save only if valid
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.set_config("llm_provider", &config.provider)
        .map_err(|e| e.to_string())?;
    db.set_config("llm_api_key", &config.api_key)
        .map_err(|e| e.to_string())?;
    Ok(())
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
    db.set_config("llm_provider", "").map_err(|e| e.to_string())?;
    db.set_config("llm_api_key", "").map_err(|e| e.to_string())?;
    Ok(())
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
    let db = state.db.lock().map_err(|e| e.to_string())?;

    // Parse
    let parsers = parsers::builtin_parsers();
    let parser = parsers::detect_parser(&input, &parsers)
        .ok_or("Could not detect input format.")?;

    let parsed = parser.parse(&input, &mapping).map_err(|e| e.to_string())?;

    // Classify using regex rules from DB
    let rules = db.get_all_rules().map_err(|e| e.to_string())?;
    let regex_classifier = RegexClassifier::from_rules(&rules);
    let classifiers: Vec<Box<dyn accountant_core::classifiers::Classifier>> =
        vec![Box::new(regex_classifier)];
    let classified = classify_pipeline(&parsed, &classifiers);

    // Batch duplicate check
    let dup_inputs: Vec<(&str, f64, &chrono::NaiveDate)> = classified
        .iter()
        .map(|(e, _)| (e.title.as_str(), e.amount, &e.date))
        .collect();
    let dup_flags = db.check_duplicates_batch(&dup_inputs).map_err(|e| e.to_string())?;

    // Build initial result
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

    // LLM fallback for unclassified, non-duplicate expenses
    let llm_provider_name = db.get_config("llm_provider").map_err(|e| e.to_string())?;
    let llm_api_key = db.get_config("llm_api_key").map_err(|e| e.to_string())?;

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
                    let categories = db.get_all_categories().unwrap_or_default();
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

                    // Best-effort: if LLM fails, expenses stay unclassified
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

    let db = state.db.lock().map_err(|e| e.to_string())?;
    let expenses = db.get_all_expenses().map_err(|e| e.to_string())?;

    let export_columns = ExportColumns {
        date: columns.date,
        title: columns.title,
        amount: columns.amount,
        category: columns.category,
        classification_source: columns.classification_source,
    };

    let exporter = CsvExporter;
    let bytes = exporter.export(&expenses, &export_columns).map_err(|e| e.to_string())?;
    std::fs::write(&path, &bytes).map_err(|e| format!("Failed to write file: {}", e))
}

// ── Bulk Save ──

#[tauri::command]
fn bulk_save_expenses(
    state: State<AppState>,
    expenses: Vec<BulkSaveExpense>,
    filename: Option<String>,
) -> Result<usize, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;

    // Build all expenses first, fail fast on invalid data
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

    // Insert atomically
    let saved = db
        .insert_expenses_bulk(&to_insert, filename.as_deref())
        .map_err(|e| e.to_string())?;

    // Save rules (best-effort, don't fail the whole operation)
    let _ = db.insert_rules_bulk(&rules);

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

#[derive(Serialize, Deserialize)]
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

#[derive(Serialize, Deserialize)]
pub struct BudgetSummaryOutput {
    pub budget_id: i64,
    pub year: i32,
    pub month: u32,
    pub categories: Vec<BudgetCategoryStatus>,
    pub budget_categories: Vec<BudgetCategoryInput>,
    pub planned_expenses: Vec<PlannedExpense>,
    pub calendar_events: Vec<CalendarEvent>,
    pub total_budgeted: f64,
    pub total_spent: f64,
    pub total_planned: f64,
}

#[tauri::command]
fn get_budget_summary(
    state: State<AppState>,
    year: i32,
    month: u32,
) -> Result<BudgetSummaryOutput, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let budget_id = db.get_or_create_budget(year, month).map_err(|e| e.to_string())?;

    let budget_cats = db.get_budget_categories(budget_id).map_err(|e| e.to_string())?;
    let planned = db.get_planned_expenses(budget_id).map_err(|e| e.to_string())?;
    let cal_events = db.get_calendar_events(budget_id).map_err(|e| e.to_string())?;
    let actual_expenses = db.get_expenses_for_month(year, month).map_err(|e| e.to_string())?;

    // Compute per-category status
    let mut category_statuses: Vec<BudgetCategoryStatus> = Vec::new();
    let mut total_budgeted = 0.0;
    let mut total_spent = 0.0;

    for bc in &budget_cats {
        let spent: f64 = actual_expenses
            .iter()
            .filter(|e| e.category.as_deref() == Some(&bc.category))
            .map(|e| e.amount)
            .sum();

        let ratio = if bc.amount > 0.0 { spent / bc.amount } else { 0.0 };
        let status = if ratio > 1.0 {
            "over"
        } else if ratio >= 0.8 {
            "approaching"
        } else {
            "under"
        };

        total_budgeted += bc.amount;
        total_spent += spent;

        category_statuses.push(BudgetCategoryStatus {
            category: bc.category.clone(),
            budgeted: bc.amount,
            spent,
            status: status.to_string(),
        });
    }

    let total_planned: f64 = planned.iter().map(|p| p.amount).sum();

    let budget_category_inputs: Vec<BudgetCategoryInput> = budget_cats
        .iter()
        .map(|bc| BudgetCategoryInput {
            category: bc.category.clone(),
            amount: bc.amount,
        })
        .collect();

    Ok(BudgetSummaryOutput {
        budget_id,
        year,
        month,
        categories: category_statuses,
        budget_categories: budget_category_inputs,
        planned_expenses: planned,
        calendar_events: cal_events,
        total_budgeted,
        total_spent,
        total_planned,
    })
}

#[tauri::command]
fn save_budget_categories(
    state: State<AppState>,
    year: i32,
    month: u32,
    categories: Vec<BudgetCategoryInput>,
) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let budget_id = db.get_or_create_budget(year, month).map_err(|e| e.to_string())?;

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
    year: i32,
    month: u32,
    expense: PlannedExpenseInput,
) -> Result<i64, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let budget_id = db.get_or_create_budget(year, month).map_err(|e| e.to_string())?;
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
fn import_calendar_events(
    state: State<AppState>,
    year: i32,
    month: u32,
    ics_content: String,
) -> Result<usize, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let budget_id = db.get_or_create_budget(year, month).map_err(|e| e.to_string())?;

    let all_events =
        accountant_core::ical::parse_ics(&ics_content).map_err(|e| e.to_string())?;
    let filtered = accountant_core::ical::filter_events_by_month(&all_events, year, month);

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
        })
        .collect();

    db.save_calendar_events(budget_id, &cal_events)
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
            save_budget_categories,
            add_planned_expense,
            delete_planned_expense,
            import_calendar_events,
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
