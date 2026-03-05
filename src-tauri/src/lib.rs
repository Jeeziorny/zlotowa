use accountant_core::classifiers::{classify_pipeline, Classifier, RegexClassifier};
use accountant_core::db::Database;
use accountant_core::llm::{create_provider, LlmConfig};
use accountant_core::ical::ParsedCalendarEvent;
use accountant_core::models::{
    BudgetCategory, BudgetCategoryStatus, BudgetStatus, CategoryAverage,
    CategoryStats, ClassificationRule, ClassificationSource, Expense, ExpenseQuery,
    ExpenseQueryResult, ParsedExpense, RuleQuery, RuleQueryResult, UploadBatch, Budget,
};
use accountant_core::parsers::{self, ColumnMapping};
use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tauri::State;

pub(crate) struct AppState {
    pub(crate) db: Mutex<Database>,
}

// ── Types ──

#[derive(Serialize, Deserialize)]
pub(crate) struct ExpenseInput {
    pub title: String,
    pub amount: f64,
    pub date: String,
    pub category: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct LlmConfigInput {
    pub provider: String,
    pub api_key: String,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct LlmConfigOutput {
    pub provider: Option<String>,
    pub api_key: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct PreviewResult {
    pub parser_name: String,
    pub rows: Vec<Vec<String>>,
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct ParsedExpenseRow {
    pub title: String,
    pub amount: f64,
    pub date: String,
    pub is_duplicate: bool,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct ClassifiedExpenseRow {
    pub title: String,
    pub amount: f64,
    pub date: String,
    pub category: Option<String>,
    pub source: Option<String>,
    pub confidence: Option<f64>,
    pub is_duplicate: bool,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct BulkSaveExpense {
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

// ── Expense Commands ──

#[tauri::command]
fn get_expenses(state: State<AppState>) -> Result<Vec<Expense>, String> {
    debug!("get_expenses called");
    let db = state.db.lock().map_err(|e| { error!("Mutex poisoned: {e}"); e.to_string() })?;
    db.get_all_expenses().map_err(|e| { warn!("get_expenses failed: {e}"); e.to_string() })
}

#[tauri::command]
fn query_expenses(state: State<AppState>, query: ExpenseQuery) -> Result<ExpenseQueryResult, String> {
    debug!("query_expenses called");
    let db = state.db.lock().map_err(|e| { error!("Mutex poisoned: {e}"); e.to_string() })?;
    db.query_expenses(&query).map_err(|e| { warn!("query_expenses failed: {e}"); e.to_string() })
}

#[tauri::command]
fn add_expense(state: State<AppState>, input: ExpenseInput) -> Result<i64, String> {
    info!("add_expense: title='{}' amount={:.2} date={}", input.title, input.amount, input.date);
    let db = state.db.lock().map_err(|e| { error!("Mutex poisoned: {e}"); e.to_string() })?;
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
                db.insert_rule(&ClassificationRule::from_pattern(&input.title, cat))?;
            }
        }
        Ok(id)
    })
    .map_err(|e| { warn!("add_expense failed: {e}"); e.to_string() })
}

#[tauri::command]
fn get_categories(state: State<AppState>) -> Result<Vec<String>, String> {
    debug!("get_categories called");
    let db = state.db.lock().map_err(|e| { error!("Mutex poisoned: {e}"); e.to_string() })?;
    db.get_all_categories().map_err(|e| { warn!("get_categories failed: {e}"); e.to_string() })
}

#[tauri::command]
fn update_expense(state: State<AppState>, id: i64, input: ExpenseInput) -> Result<(), String> {
    info!("update_expense: id={id} title='{}' amount={:.2}", input.title, input.amount);
    let db = state.db.lock().map_err(|e| { error!("Mutex poisoned: {e}"); e.to_string() })?;
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
                db.insert_rule(&ClassificationRule::from_pattern(&input.title, cat))?;
            }
        }
        Ok(())
    })
    .map_err(|e| { warn!("update_expense failed: {e}"); e.to_string() })
}

#[tauri::command]
fn delete_expense(state: State<AppState>, id: i64) -> Result<(), String> {
    info!("delete_expense: id={id}");
    let db = state.db.lock().map_err(|e| { error!("Mutex poisoned: {e}"); e.to_string() })?;
    db.delete_expense(id).map_err(|e| { warn!("delete_expense failed: {e}"); e.to_string() })
}

#[tauri::command]
fn delete_expenses(state: State<AppState>, ids: Vec<i64>) -> Result<usize, String> {
    info!("delete_expenses: count={}", ids.len());
    let db = state.db.lock().map_err(|e| { error!("Mutex poisoned: {e}"); e.to_string() })?;
    db.delete_expenses(&ids).map_err(|e| { warn!("delete_expenses failed: {e}"); e.to_string() })
}

// ── LLM Config Commands ──

#[tauri::command]
fn get_llm_config(state: State<AppState>) -> Result<LlmConfigOutput, String> {
    debug!("get_llm_config called");
    let db = state.db.lock().map_err(|e| { error!("Mutex poisoned: {e}"); e.to_string() })?;
    Ok(LlmConfigOutput {
        provider: db.get_config("llm_provider").map_err(|e| e.to_string())?,
        api_key: db.get_config("llm_api_key").map_err(|e| e.to_string())?,
    })
}

#[tauri::command]
fn save_llm_config(state: State<AppState>, config: LlmConfigInput) -> Result<(), String> {
    info!("save_llm_config: provider='{}'", config.provider);
    // Validate before saving
    let provider = create_provider(&config.provider)
        .ok_or_else(|| format!("Unknown provider: {}", config.provider))?;
    let llm_config = LlmConfig {
        provider: config.provider.clone(),
        api_key: config.api_key.clone(),
    };
    provider.validate(&llm_config).map_err(|e| { warn!("LLM validation failed: {e}"); e.to_string() })?;

    // Save atomically — both keys in one transaction
    let db = state.db.lock().map_err(|e| { error!("Mutex poisoned: {e}"); e.to_string() })?;
    db.with_transaction(|| {
        db.set_config("llm_provider", &config.provider)?;
        db.set_config("llm_api_key", &config.api_key)?;
        Ok(())
    })
    .map_err(|e| { warn!("save_llm_config failed: {e}"); e.to_string() })
}

#[tauri::command]
fn validate_llm_config(config: LlmConfigInput) -> Result<(), String> {
    debug!("validate_llm_config: provider='{}'", config.provider);
    let provider = create_provider(&config.provider)
        .ok_or_else(|| format!("Unknown provider: {}", config.provider))?;
    let llm_config = LlmConfig {
        provider: config.provider,
        api_key: config.api_key,
    };
    provider.validate(&llm_config).map_err(|e| { warn!("validate_llm_config failed: {e}"); e.to_string() })
}

#[tauri::command]
fn clear_llm_config(state: State<AppState>) -> Result<(), String> {
    info!("clear_llm_config called");
    let db = state.db.lock().map_err(|e| { error!("Mutex poisoned: {e}"); e.to_string() })?;
    db.with_transaction(|| {
        db.set_config("llm_provider", "")?;
        db.set_config("llm_api_key", "")?;
        Ok(())
    })
    .map_err(|e| { warn!("clear_llm_config failed: {e}"); e.to_string() })
}

// ── Category Suggestion ──

#[tauri::command]
fn suggest_category(state: State<AppState>, title: String) -> Result<Option<String>, String> {
    debug!("suggest_category: title='{title}'");
    if title.trim().is_empty() {
        return Ok(None);
    }
    let db = state.db.lock().map_err(|e| { error!("Mutex poisoned: {e}"); e.to_string() })?;
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
fn preview_csv(input: String, delimiter: Option<String>) -> Result<PreviewResult, String> {
    debug!("preview_csv: input length={} delimiter={:?}", input.len(), delimiter);

    if let Some(ref d) = delimiter {
        let delim_char = d.chars().next().ok_or("Invalid delimiter")?;
        let parser = parsers::csv_parser::CsvParser;
        let rows = parser.preview_rows_with_delimiter(&input, delim_char)
            .map_err(|e| { warn!("preview_csv failed: {e}"); e.to_string() })?;
        debug!("preview_csv: forced delimiter='{}' rows={}", delim_char, rows.len());
        return Ok(PreviewResult {
            parser_name: "CSV".to_string(),
            rows,
        });
    }

    let parsers = parsers::builtin_parsers();
    let parser = parsers::detect_parser(&input, &parsers)
        .ok_or("Could not detect input format. Supported: CSV (comma, semicolon, tab delimited).")?;

    let rows = parser.preview_rows(&input).map_err(|e| { warn!("preview_csv failed: {e}"); e.to_string() })?;
    debug!("preview_csv: parser='{}' rows={}", parser.name(), rows.len());

    Ok(PreviewResult {
        parser_name: parser.name().to_string(),
        rows,
    })
}

#[tauri::command]
fn parse_csv_data(
    state: State<AppState>,
    input: String,
    mapping: ColumnMapping,
    delimiter: Option<String>,
) -> Result<Vec<ParsedExpenseRow>, String> {
    info!("parse_csv_data: input length={} delimiter={:?}", input.len(), delimiter);

    let parsed = if let Some(ref d) = delimiter {
        let delim_char = d.chars().next().ok_or("Invalid delimiter")?;
        let parser = parsers::csv_parser::CsvParser;
        parser.parse_with_delimiter(&input, &mapping, delim_char)
            .map_err(|e| { warn!("parse_csv_data parse failed: {e}"); e.to_string() })?
    } else {
        let parsers = parsers::builtin_parsers();
        let parser = parsers::detect_parser(&input, &parsers)
            .ok_or("Could not detect input format.")?;
        parser.parse(&input, &mapping).map_err(|e| { warn!("parse_csv_data parse failed: {e}"); e.to_string() })?
    };
    info!("parse_csv_data: parsed {} expenses", parsed.len());

    let dup_flags = {
        let db = state.db.lock().map_err(|e| { error!("Mutex poisoned: {e}"); e.to_string() })?;
        let dup_inputs: Vec<(&str, f64, &chrono::NaiveDate)> = parsed
            .iter()
            .map(|e| (e.title.as_str(), e.amount, &e.date))
            .collect();
        db.check_duplicates_batch(&dup_inputs).map_err(|e| e.to_string())?
    };

    let rows: Vec<ParsedExpenseRow> = parsed
        .iter()
        .zip(dup_flags.iter())
        .map(|(e, &is_dup)| ParsedExpenseRow {
            title: e.title.clone(),
            amount: e.amount,
            date: e.date.to_string(),
            is_duplicate: is_dup,
        })
        .collect();

    Ok(rows)
}

#[tauri::command]
fn classify_expenses(
    state: State<AppState>,
    rows: Vec<ParsedExpenseRow>,
) -> Result<Vec<ClassifiedExpenseRow>, String> {
    info!("classify_expenses: {} rows", rows.len());

    let parsed: Vec<ParsedExpense> = rows
        .iter()
        .filter_map(|r| {
            chrono::NaiveDate::parse_from_str(&r.date, "%Y-%m-%d")
                .ok()
                .map(|date| ParsedExpense {
                    title: r.title.clone(),
                    amount: r.amount,
                    date,
                })
        })
        .collect();

    let (rules, llm_provider_name, llm_api_key, categories) = {
        let db = state.db.lock().map_err(|e| { error!("Mutex poisoned: {e}"); e.to_string() })?;
        let rules = db.get_all_rules().map_err(|e| e.to_string())?;
        let llm_provider_name = db.get_config("llm_provider").map_err(|e| e.to_string())?;
        let llm_api_key = db.get_config("llm_api_key").map_err(|e| e.to_string())?;
        let categories = db.get_all_categories().unwrap_or_default();
        (rules, llm_provider_name, llm_api_key, categories)
    };

    let regex_classifier = RegexClassifier::from_rules(&rules);
    let classifiers: Vec<Box<dyn Classifier>> = vec![Box::new(regex_classifier)];
    let classified = classify_pipeline(&parsed, &classifiers);

    let mut result: Vec<ClassifiedExpenseRow> = Vec::new();
    for ((expense, class_result), orig_row) in classified.iter().zip(rows.iter()) {
        let (category, source, confidence) = match class_result {
            Some(cr) => (Some(cr.category.clone()), Some(cr.source.to_string()), Some(cr.confidence)),
            None => (None, None, None),
        };
        result.push(ClassifiedExpenseRow {
            title: expense.title.clone(),
            amount: expense.amount,
            date: orig_row.date.clone(),
            category,
            source,
            confidence,
            is_duplicate: orig_row.is_duplicate,
        });
    }

    let regex_classified = result.iter().filter(|r| r.category.is_some()).count();
    info!("classify_expenses: {regex_classified}/{} classified by regex", result.len());

    // LLM fallback
    if let (Some(provider_name), Some(api_key)) = (&llm_provider_name, &llm_api_key) {
        if !provider_name.is_empty() && !api_key.is_empty() {
            let unclassified_indices: Vec<usize> = result
                .iter()
                .enumerate()
                .filter(|(_, r)| r.category.is_none() && !r.is_duplicate)
                .map(|(i, _)| i)
                .collect();

            if !unclassified_indices.is_empty() {
                info!("classify_expenses: LLM fallback — {} unclassified, provider='{provider_name}'", unclassified_indices.len());
                if let Some(provider) = create_provider(provider_name) {
                    let config = LlmConfig {
                        provider: provider_name.clone(),
                        api_key: api_key.clone(),
                    };
                    let unclassified_expenses: Vec<ParsedExpense> = unclassified_indices
                        .iter()
                        .filter_map(|&i| {
                            let row = result.get(i)?;
                            Some(ParsedExpense {
                                title: row.title.clone(),
                                amount: row.amount,
                                date: chrono::NaiveDate::parse_from_str(&row.date, "%Y-%m-%d")
                                    .unwrap_or_else(|_| chrono::Local::now().date_naive()),
                            })
                        })
                        .collect();

                    // Chunk into batches of 30 to avoid LLM response truncation
                    const CHUNK_SIZE: usize = 30;
                    let mut llm_errors: Vec<String> = Vec::new();
                    let mut total_classified = 0usize;

                    for (chunk_idx, chunk) in unclassified_expenses.chunks(CHUNK_SIZE).enumerate() {
                        let idx_offset = chunk_idx * CHUNK_SIZE;
                        info!("classify_expenses: LLM chunk {}: {} expenses", chunk_idx + 1, chunk.len());

                        match provider.classify_batch(chunk, &categories, &config) {
                            Ok(llm_results) => {
                                let classified = llm_results.iter().filter(|r| r.is_some()).count();
                                total_classified += classified;
                                for (j, llm_result) in llm_results.into_iter().enumerate() {
                                    if let Some(classification) = llm_result {
                                        let orig_idx = unclassified_indices[idx_offset + j];
                                        if let Some(row) = result.get_mut(orig_idx) {
                                            row.category = Some(classification.category);
                                            row.source = Some(ClassificationSource::Llm.to_string());
                                            row.confidence = Some(classification.confidence);
                                        }
                                    }
                                }
                            }
                            Err(e) => {
                                warn!("classify_expenses: LLM chunk {} failed: {e}", chunk_idx + 1);
                                llm_errors.push(format!("{e}"));
                            }
                        }
                    }
                    info!("classify_expenses: LLM classified {total_classified}/{}", unclassified_expenses.len());
                    if !llm_errors.is_empty() && total_classified == 0 {
                        return Err(format!("LLM classification failed: {}", llm_errors[0]));
                    }
                }
            }
        }
    }

    let final_classified = result.iter().filter(|r| r.category.is_some()).count();
    info!("classify_expenses: complete — {final_classified}/{} classified total", result.len());
    Ok(result)
}

#[tauri::command]
fn parse_and_classify(
    state: State<AppState>,
    input: String,
    mapping: ColumnMapping,
) -> Result<Vec<ClassifiedExpenseRow>, String> {
    info!("parse_and_classify: input length={}", input.len());

    // Phase 1: Parse CSV (no DB needed)
    let parsers = parsers::builtin_parsers();
    let parser = parsers::detect_parser(&input, &parsers)
        .ok_or("Could not detect input format.")?;
    let parsed = parser.parse(&input, &mapping).map_err(|e| { warn!("parse_and_classify parse failed: {e}"); e.to_string() })?;
    info!("parse_and_classify: phase 1 complete — parsed {} expenses", parsed.len());

    // Phase 2: Read all needed DB data in a single lock, then release
    let (rules, llm_provider_name, llm_api_key, categories, dup_flags) = {
        let db = state.db.lock().map_err(|e| { error!("Mutex poisoned: {e}"); e.to_string() })?;
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
    info!("parse_and_classify: phase 2 complete — loaded {} rules, {} categories", rules.len(), categories.len());

    // Phase 2b: Detect intra-batch duplicates (second+ occurrence of same title/amount/date)
    let mut dup_flags = dup_flags;
    {
        let mut seen = std::collections::HashSet::new();
        for (i, expense) in parsed.iter().enumerate() {
            if dup_flags[i] {
                continue; // already flagged as DB duplicate
            }
            let key = (expense.title.as_str(), expense.amount.to_bits(), expense.date);
            if !seen.insert(key) {
                dup_flags[i] = true;
            }
        }
    }

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

    let regex_classified = rows.iter().filter(|r| r.category.is_some()).count();
    info!("parse_and_classify: phase 3-4 complete — {regex_classified}/{} classified by regex", rows.len());

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
                info!("parse_and_classify: LLM fallback — {} unclassified expenses, provider='{provider_name}'", unclassified_indices.len());
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

                    match provider.classify_batch(&unclassified_expenses, &categories, &config) {
                    Ok(llm_results) => {
                        let llm_classified = llm_results.iter().filter(|r| r.is_some()).count();
                        info!("parse_and_classify: LLM classified {llm_classified}/{} expenses", unclassified_expenses.len());
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
                    Err(e) => {
                        warn!("parse_and_classify: LLM fallback failed: {e}");
                    }
                    }
                }
            }
        }
    }

    let final_classified = rows.iter().filter(|r| r.category.is_some()).count();
    info!("parse_and_classify: complete — {final_classified}/{} classified total", rows.len());
    Ok(rows)
}

// ── Backup & Restore ──

#[tauri::command]
fn backup_database(state: State<AppState>, path: String) -> Result<(), String> {
    use accountant_core::backup::create_backup;
    info!("backup_database: path='{path}'");

    let backup = {
        let db = state.db.lock().map_err(|e| { error!("Mutex poisoned: {e}"); e.to_string() })?;
        create_backup(&db).map_err(|e| e.to_string())?
    };

    let json = serde_json::to_string_pretty(&backup).map_err(|e| e.to_string())?;
    std::fs::write(&path, json).map_err(|e| { warn!("backup_database write failed: {e}"); format!("Failed to write file: {}", e) })?;

    info!("backup_database: complete — {} expenses, {} rules, {} budgets",
        backup.expenses.len(), backup.classification_rules.len(),
        backup.budgets.len());
    Ok(())
}

#[tauri::command]
fn preview_backup(path: String) -> Result<accountant_core::backup::BackupPreview, String> {
    use accountant_core::backup::preview_backup as do_preview;
    info!("preview_backup: path='{path}'");

    let content = std::fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read file: {}", e))?;
    let backup: accountant_core::backup::BackupData =
        serde_json::from_str(&content).map_err(|e| format!("Invalid backup file: {}", e))?;

    let preview = do_preview(&backup).map_err(|e| e.to_string())?;
    info!("preview_backup: {} expenses, {} rules, {} categories, {} budgets",
        preview.expense_count, preview.rule_count, preview.category_count, preview.budget_count);
    Ok(preview)
}

#[tauri::command]
fn restore_database(
    state: State<AppState>,
    path: String,
) -> Result<accountant_core::backup::RestoreSummary, String> {
    use accountant_core::backup::restore_backup;
    info!("restore_database: path='{path}'");

    let content = std::fs::read_to_string(&path)
        .map_err(|e| { warn!("restore_database read failed: {e}"); format!("Failed to read file: {}", e) })?;
    let backup: accountant_core::backup::BackupData =
        serde_json::from_str(&content).map_err(|e| format!("Invalid backup file: {}", e))?;

    let db = state.db.lock().map_err(|e| { error!("Mutex poisoned: {e}"); e.to_string() })?;
    let summary = restore_backup(&db, &backup).map_err(|e| e.to_string())?;

    info!("restore_database: complete — {} expenses inserted ({} skipped), {} rules, {} budgets ({} skipped)",
        summary.expenses_inserted, summary.expenses_skipped, summary.rules_upserted,
        summary.budgets_inserted, summary.budgets_skipped);
    Ok(summary)
}

// ── Bulk Save ──

#[tauri::command]
fn bulk_save_expenses(
    state: State<AppState>,
    expenses: Vec<BulkSaveExpense>,
    filename: Option<String>,
) -> Result<usize, String> {
    info!("bulk_save_expenses: count={} filename={:?}", expenses.len(), filename);

    // Build all expenses (pure computation, no lock needed), fail fast on invalid data
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
                rules.push(ClassificationRule::from_pattern(&e.title, cat));
            }
        }
    }

    // Acquire lock for the DB insert
    let db = state.db.lock().map_err(|e| { error!("Mutex poisoned: {e}"); e.to_string() })?;
    let saved = db
        .insert_expenses_bulk(&to_insert, filename.as_deref(), &rules)
        .map_err(|e| { warn!("bulk_save_expenses failed: {e}"); e.to_string() })?;

    info!("bulk_save_expenses: saved {saved} expenses");
    Ok(saved)
}

// ── Upload Batch Management ──

#[tauri::command]
fn get_upload_batches(state: State<AppState>) -> Result<Vec<UploadBatch>, String> {
    debug!("get_upload_batches called");
    let db = state.db.lock().map_err(|e| { error!("Mutex poisoned: {e}"); e.to_string() })?;
    db.get_upload_batches().map_err(|e| { warn!("get_upload_batches failed: {e}"); e.to_string() })
}

#[tauri::command]
fn delete_batch(state: State<AppState>, batch_id: i64) -> Result<usize, String> {
    info!("delete_batch: batch_id={batch_id}");
    let db = state.db.lock().map_err(|e| { error!("Mutex poisoned: {e}"); e.to_string() })?;
    db.delete_batch(batch_id).map_err(|e| { warn!("delete_batch failed: {e}"); e.to_string() })
}

// ── Category Management ──

#[tauri::command]
fn get_category_stats(state: State<AppState>) -> Result<Vec<CategoryStats>, String> {
    debug!("get_category_stats called");
    let db = state.db.lock().map_err(|e| { error!("Mutex poisoned: {e}"); e.to_string() })?;
    db.get_category_stats().map_err(|e| { warn!("get_category_stats failed: {e}"); e.to_string() })
}

#[tauri::command]
fn create_category(state: State<AppState>, name: String) -> Result<(), String> {
    info!("create_category: name='{name}'");
    let db = state.db.lock().map_err(|e| { error!("Mutex poisoned: {e}"); e.to_string() })?;
    if db.category_exists(&name).map_err(|e| e.to_string())? {
        return Err(format!("Category '{}' already exists", name));
    }
    db.create_category(&name).map_err(|e| { warn!("create_category failed: {e}"); e.to_string() })
}

#[tauri::command]
fn rename_category(state: State<AppState>, old_name: String, new_name: String) -> Result<(), String> {
    info!("rename_category: '{old_name}' -> '{new_name}'");
    if old_name == new_name {
        return Ok(());
    }
    let db = state.db.lock().map_err(|e| { error!("Mutex poisoned: {e}"); e.to_string() })?;
    if old_name.to_lowercase() != new_name.to_lowercase()
        && db.category_exists(&new_name).map_err(|e| e.to_string())?
    {
        return Err(format!("Category '{}' already exists", new_name));
    }
    db.rename_category(&old_name, &new_name).map_err(|e| { warn!("rename_category failed: {e}"); e.to_string() })
}

#[tauri::command]
fn delete_category(state: State<AppState>, category: String, replacement: String) -> Result<(), String> {
    info!("delete_category: '{category}' -> replacement='{replacement}'");
    let db = state.db.lock().map_err(|e| { error!("Mutex poisoned: {e}"); e.to_string() })?;
    db.delete_category(&category, &replacement).map_err(|e| { warn!("delete_category failed: {e}"); e.to_string() })
}

#[tauri::command]
fn merge_categories(state: State<AppState>, sources: Vec<String>, target: String) -> Result<(), String> {
    info!("merge_categories: {} sources -> '{target}'", sources.len());
    if sources.is_empty() || target.is_empty() {
        return Err("Sources and target must not be empty".to_string());
    }
    let db = state.db.lock().map_err(|e| { error!("Mutex poisoned: {e}"); e.to_string() })?;
    db.merge_categories(&sources, &target).map_err(|e| { warn!("merge_categories failed: {e}"); e.to_string() })
}

// ── Budget Planning ──

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct BudgetCategoryInput {
    pub category: String,
    pub amount: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct BudgetSummaryOutput {
    pub budget_id: i64,
    pub start_date: String,
    pub end_date: String,
    pub categories: Vec<BudgetCategoryStatus>,
    pub budget_categories: Vec<BudgetCategoryInput>,
    pub total_budgeted: f64,
    pub total_spent: f64,
}

fn build_budget_summary(db: &Database, budget: &Budget) -> Result<BudgetSummaryOutput, String> {
    let budget_id = budget.id.ok_or_else(|| "Budget has no id".to_string())?;
    let budget_cats = db.get_budget_categories(budget_id).map_err(|e| e.to_string())?;
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
        total_budgeted,
        total_spent,
    })
}

#[tauri::command]
fn get_budget_summary(
    state: State<AppState>,
    budget_id: i64,
) -> Result<BudgetSummaryOutput, String> {
    debug!("get_budget_summary: budget_id={budget_id}");
    let db = state.db.lock().map_err(|e| { error!("Mutex poisoned: {e}"); e.to_string() })?;
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
    debug!("get_active_budget_summary called");
    let db = state.db.lock().map_err(|e| { error!("Mutex poisoned: {e}"); e.to_string() })?;
    match db.get_active_budget().map_err(|e| e.to_string())? {
        Some(budget) => Ok(Some(build_budget_summary(&db, &budget)?)),
        None => Ok(None),
    }
}

#[tauri::command]
fn list_budgets(state: State<AppState>) -> Result<Vec<Budget>, String> {
    debug!("list_budgets called");
    let db = state.db.lock().map_err(|e| { error!("Mutex poisoned: {e}"); e.to_string() })?;
    db.get_all_budgets().map_err(|e| { warn!("list_budgets failed: {e}"); e.to_string() })
}

#[tauri::command]
fn create_budget(
    state: State<AppState>,
    start_date: String,
    end_date: String,
    categories: Vec<BudgetCategoryInput>,
) -> Result<i64, String> {
    info!("create_budget: {start_date} to {end_date}, {} categories", categories.len());
    let start = parse_date(&start_date)?;
    let end = parse_date(&end_date)?;
    let db = state.db.lock().map_err(|e| { error!("Mutex poisoned: {e}"); e.to_string() })?;

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
        .map_err(|e| { warn!("create_budget failed: {e}"); e.to_string() })
}

#[tauri::command]
fn save_budget_categories(
    state: State<AppState>,
    budget_id: i64,
    categories: Vec<BudgetCategoryInput>,
) -> Result<(), String> {
    info!("save_budget_categories: budget_id={budget_id} categories={}", categories.len());
    let db = state.db.lock().map_err(|e| { error!("Mutex poisoned: {e}"); e.to_string() })?;

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
        .map_err(|e| { warn!("save_budget_categories failed: {e}"); e.to_string() })
}

#[tauri::command]
fn delete_budget(state: State<AppState>, id: i64) -> Result<(), String> {
    info!("delete_budget: id={id}");
    let db = state.db.lock().map_err(|e| { error!("Mutex poisoned: {e}"); e.to_string() })?;
    db.delete_budget(id).map_err(|e| { warn!("delete_budget failed: {e}"); e.to_string() })
}

#[tauri::command]
fn parse_calendar_events(
    ics_content: String,
    start_date: String,
    end_date: String,
) -> Result<Vec<ParsedCalendarEvent>, String> {
    debug!("parse_calendar_events: range {start_date} to {end_date}");
    let start = parse_date(&start_date)?;
    let end = parse_date(&end_date)?;
    let all_events =
        accountant_core::ical::parse_ics(&ics_content).map_err(|e| { warn!("parse_calendar_events failed: {e}"); e.to_string() })?;
    let filtered = accountant_core::ical::filter_events_by_date_range(
        &all_events, start, end,
    );
    debug!("parse_calendar_events: {} total events, {} in range", all_events.len(), filtered.len());
    Ok(filtered)
}

#[tauri::command]
fn check_budget_overlap(
    state: State<AppState>,
    start_date: String,
    end_date: String,
) -> Result<bool, String> {
    debug!("check_budget_overlap: {start_date} to {end_date}");
    let start = parse_date(&start_date)?;
    let end = parse_date(&end_date)?;
    let db = state.db.lock().map_err(|e| { error!("Mutex poisoned: {e}"); e.to_string() })?;
    db.check_budget_overlap(start, end)
        .map_err(|e| { warn!("check_budget_overlap failed: {e}"); e.to_string() })
}

#[tauri::command]
fn get_category_averages(state: State<AppState>) -> Result<Vec<CategoryAverage>, String> {
    debug!("get_category_averages called");
    let db = state.db.lock().map_err(|e| { error!("Mutex poisoned: {e}"); e.to_string() })?;
    db.get_category_averages(3).map_err(|e| { warn!("get_category_averages failed: {e}"); e.to_string() })
}

// ── Config ──

#[tauri::command]
fn get_config(state: State<AppState>, key: String) -> Result<Option<String>, String> {
    debug!("get_config: key={key}");
    let db = state.db.lock().map_err(|e| { error!("Mutex poisoned: {e}"); e.to_string() })?;
    db.get_config(&key).map_err(|e| { warn!("get_config failed: {e}"); e.to_string() })
}

#[tauri::command]
fn save_config(state: State<AppState>, key: String, value: String) -> Result<(), String> {
    debug!("save_config: key={key}");
    let db = state.db.lock().map_err(|e| { error!("Mutex poisoned: {e}"); e.to_string() })?;
    db.set_config(&key, &value).map_err(|e| { warn!("save_config failed: {e}"); e.to_string() })
}

// ── Classification Rules ──

#[tauri::command]
fn query_rules(state: State<AppState>, query: RuleQuery) -> Result<RuleQueryResult, String> {
    debug!("query_rules called");
    let db = state.db.lock().map_err(|e| { error!("Mutex poisoned: {e}"); e.to_string() })?;
    db.query_rules(&query).map_err(|e| { warn!("query_rules failed: {e}"); e.to_string() })
}

#[tauri::command]
fn add_rule(state: State<AppState>, pattern: String, category: String) -> Result<i64, String> {
    debug!("add_rule: pattern={pattern}, category={category}");
    let rule = ClassificationRule { id: None, pattern, category };
    let db = state.db.lock().map_err(|e| { error!("Mutex poisoned: {e}"); e.to_string() })?;
    db.insert_rule(&rule).map_err(|e| { warn!("add_rule failed: {e}"); e.to_string() })
}

#[tauri::command]
fn update_rule(state: State<AppState>, id: i64, pattern: String, category: String) -> Result<(), String> {
    debug!("update_rule: id={id}");
    let rule = ClassificationRule { id: Some(id), pattern, category };
    let db = state.db.lock().map_err(|e| { error!("Mutex poisoned: {e}"); e.to_string() })?;
    db.update_rule(&rule).map_err(|e| { warn!("update_rule failed: {e}"); e.to_string() })
}

#[tauri::command]
fn delete_rule(state: State<AppState>, id: i64) -> Result<(), String> {
    debug!("delete_rule: id={id}");
    let db = state.db.lock().map_err(|e| { error!("Mutex poisoned: {e}"); e.to_string() })?;
    db.delete_rule(id).map_err(|e| { warn!("delete_rule failed: {e}"); e.to_string() })
}

// ── Dashboard Widget Config ──

#[tauri::command]
fn get_active_widgets(state: State<AppState>) -> Result<Option<serde_json::Value>, String> {
    debug!("get_active_widgets called");
    let db = state.db.lock().map_err(|e| { error!("Mutex poisoned: {e}"); e.to_string() })?;
    let val = db.get_config("active_widgets").map_err(|e| e.to_string())?;
    match val {
        Some(json) if !json.is_empty() => {
            let value: serde_json::Value = serde_json::from_str(&json).map_err(|e| e.to_string())?;
            Ok(Some(value))
        }
        _ => Ok(None),
    }
}

#[tauri::command]
fn save_active_widgets(state: State<AppState>, widgets: serde_json::Value) -> Result<(), String> {
    info!("save_active_widgets called");
    let db = state.db.lock().map_err(|e| { error!("Mutex poisoned: {e}"); e.to_string() })?;
    let json = serde_json::to_string(&widgets).map_err(|e| e.to_string())?;
    db.set_config("active_widgets", &json).map_err(|e| { warn!("save_active_widgets failed: {e}"); e.to_string() })
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


            },
            BulkSaveExpense {
                title: "Item2".into(),
                amount: 20.0,
                date: "2024-01-02".into(),
                category: None,
                source: None,


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

    #[test]
    fn parse_and_classify_detects_intra_batch_duplicates() {
        let app = app();
        let csv = "date,title,amount\n2024-01-01,Coffee,3.50\n2024-01-01,Coffee,3.50\n2024-01-01,Bus,2.00\n";
        let mapping = ColumnMapping {
            title_index: 1,
            amount_index: 2,
            date_index: 0,
            date_format: "%Y-%m-%d".into(),
        };

        let state: State<AppState> = app.state();
        let rows = parse_and_classify(state, csv.into(), mapping).unwrap();
        assert!(!rows[0].is_duplicate, "first Coffee should be kept");
        assert!(rows[1].is_duplicate, "second Coffee should be flagged as intra-batch duplicate");
        assert!(!rows[2].is_duplicate, "Bus is unique");
    }

    // ── Budget Planning ──

    #[test]
    fn budget_lifecycle() {
        let app = app();

        // Use dates that span "today" so get_active_budget works
        let today = chrono::Local::now().date_naive();
        let start = today.format("%Y-%m-%d").to_string();
        let end = (today + chrono::Duration::days(30)).format("%Y-%m-%d").to_string();

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
    fn list_budgets_returns_sorted() {
        let app = app();
        // Create budgets out of order
        let state: State<AppState> = app.state();
        create_budget(state, "2024-06-01".into(), "2024-06-30".into(), vec![]).unwrap();
        let state: State<AppState> = app.state();
        create_budget(state, "2024-01-01".into(), "2024-01-31".into(), vec![]).unwrap();
        let state: State<AppState> = app.state();
        create_budget(state, "2024-03-01".into(), "2024-03-31".into(), vec![]).unwrap();

        let state: State<AppState> = app.state();
        let budgets = list_budgets(state).unwrap();
        assert_eq!(budgets.len(), 3);
        assert_eq!(budgets[0].start_date.to_string(), "2024-01-01");
        assert_eq!(budgets[1].start_date.to_string(), "2024-03-01");
        assert_eq!(budgets[2].start_date.to_string(), "2024-06-01");
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

            },
        )
        .unwrap();

        let state: State<AppState> = app.state();
        let summary = get_budget_summary(state, budget_id).unwrap();
        assert_eq!(summary.total_spent, 85.0);
        assert_eq!(summary.categories[0].status, BudgetStatus::Approaching); // 85/100 = 0.85
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

        // Save instance-object format
        let state: State<AppState> = app.state();
        let widgets = serde_json::json!([
            { "widgetId": "total-stats", "instanceId": "total-stats" },
            { "widgetId": "keyword-tracker", "instanceId": "kw-1", "config": { "keyword": "LIDL" } }
        ]);
        save_active_widgets(state, widgets.clone()).unwrap();

        // Load
        let state: State<AppState> = app.state();
        let result = get_active_widgets(state).unwrap();
        assert_eq!(result.unwrap(), widgets);
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

}

// ── App Entry ──

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    use tauri_plugin_log::{Target, TargetKind, TimezoneStrategy, RotationStrategy};

    let db = Database::open_default().unwrap_or_else(|e| {
        eprintln!("Fatal: failed to open database: {e}");
        std::process::exit(1);
    });

    tauri::Builder::default()
        .plugin(
            tauri_plugin_log::Builder::new()
                .targets([
                    Target::new(TargetKind::Stdout),
                    Target::new(TargetKind::LogDir { file_name: None }),
                ])
                .timezone_strategy(TimezoneStrategy::UseLocal)
                .max_file_size(5_000_000) // 5 MB
                .rotation_strategy(RotationStrategy::KeepAll)
                .level(log::LevelFilter::Debug)
                .build(),
        )
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(AppState {
            db: Mutex::new(db),
        })
        .setup(|_app| {
            info!("złotówa started — log plugin loaded");
            Ok(())
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
            parse_csv_data,
            classify_expenses,
            parse_and_classify,
            backup_database,
            preview_backup,
            restore_database,
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
            list_budgets,
            create_budget,
            save_budget_categories,
            delete_budget,
            parse_calendar_events,
            check_budget_overlap,
            get_category_averages,
            get_config,
            save_config,
            query_rules,
            add_rule,
            update_rule,
            delete_rule,
        ])
        .run(tauri::generate_context!())
        .unwrap_or_else(|e| {
            eprintln!("Fatal: failed to start application: {e}");
            std::process::exit(1);
        });
}
