use accountant_core::classifiers::{classify_pipeline, Classifier, RegexClassifier};
use accountant_core::db::Database;
use accountant_core::llm::{create_provider, LlmConfig};
use accountant_core::models::{ClassificationRule, ClassificationSource, Expense, ParsedExpense};
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
    pub is_duplicate: bool,
}

#[derive(Serialize, Deserialize)]
pub struct BulkSaveExpense {
    pub title: String,
    pub amount: f64,
    pub date: String,
    pub category: Option<String>,
    pub source: Option<String>,
}

// ── Helpers ──

fn parse_date(s: &str) -> Result<chrono::NaiveDate, String> {
    chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d")
        .map_err(|e| format!("Invalid date '{}': {}", s, e))
}

fn make_classification_rule(title: &str, category: &str) -> ClassificationRule {
    let pattern = regex::escape(title);
    ClassificationRule {
        id: None,
        pattern: format!("(?i){}", pattern),
        category: category.to_string(),
    }
}

fn save_rule_if_categorized(db: &Database, title: &str, category: &Option<String>) {
    if let Some(cat) = category {
        if !cat.is_empty() {
            let _ = db.insert_rule(&make_classification_rule(title, cat));
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
    save_rule_if_categorized(&db, &input.title, &input.category);
    Ok(id)
}

#[tauri::command]
fn get_categories(state: State<AppState>) -> Result<Vec<String>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.get_all_categories().map_err(|e| e.to_string())
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

    // Check for duplicates and build initial result
    let mut rows: Vec<ClassifiedExpenseRow> = Vec::new();
    for (expense, class_result) in &classified {
        let is_dup = db
            .is_duplicate(&expense.title, expense.amount, &expense.date)
            .map_err(|e| e.to_string())?;

        let (category, source) = match class_result {
            Some(cr) => (Some(cr.category.clone()), Some(cr.source.to_string())),
            None => (None, None),
        };

        rows.push(ClassifiedExpenseRow {
            title: expense.title.clone(),
            amount: expense.amount,
            date: expense.date.to_string(),
            category,
            source,
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
                        .map(|&i| ParsedExpense {
                            title: rows[i].title.clone(),
                            amount: rows[i].amount,
                            date: chrono::NaiveDate::parse_from_str(&rows[i].date, "%Y-%m-%d")
                                .unwrap_or_else(|_| chrono::Local::now().date_naive()),
                        })
                        .collect();

                    // Best-effort: if LLM fails, expenses stay unclassified
                    if let Ok(llm_results) =
                        provider.classify_batch(&unclassified_expenses, &categories, &config)
                    {
                        for (idx, llm_cat) in
                            unclassified_indices.iter().zip(llm_results.into_iter())
                        {
                            if let Some(cat) = llm_cat {
                                rows[*idx].category = Some(cat);
                                rows[*idx].source = Some("Llm".to_string());
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
) -> Result<Vec<u8>, String> {
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
    exporter.export(&expenses, &export_columns).map_err(|e| e.to_string())
}

// ── Bulk Save ──

#[tauri::command]
fn bulk_save_expenses(
    state: State<AppState>,
    expenses: Vec<BulkSaveExpense>,
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
                rules.push(make_classification_rule(&e.title, cat));
            }
        }
    }

    // Insert atomically
    let saved = db
        .insert_expenses_bulk(&to_insert)
        .map_err(|e| e.to_string())?;

    // Save rules (best-effort, don't fail the whole operation)
    let _ = db.insert_rules_bulk(&rules);

    Ok(saved)
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
    let db = Database::open_default().expect("Failed to open database");

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(AppState {
            db: Mutex::new(db),
        })
        .invoke_handler(tauri::generate_handler![
            get_expenses,
            add_expense,
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
            get_active_widgets,
            save_active_widgets,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
