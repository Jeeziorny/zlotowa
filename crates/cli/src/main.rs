use clap::{Parser, Subcommand};
use colored::Colorize;
use std::path::PathBuf;

use accountant_core::backup::{create_backup, restore_backup};
use accountant_core::classifiers::{classify_pipeline, RegexClassifier};
use accountant_core::db::Database;
use accountant_core::llm::{create_provider, LlmConfig};
use accountant_core::models::{ClassificationRule, ClassificationSource, Expense};
use accountant_core::parsers::{self, ColumnMapping};

#[derive(Parser)]
#[command(name = "4ccountant", about = "Expense classifier and budget planner")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Configure LLM API key
    LlmConf,

    /// Bulk-insert expenses from a file
    BulkInsert {
        /// Path to CSV or text file with expenses
        path: PathBuf,
    },

    /// Backup all data to a JSON file
    Backup {
        /// Output file path (default: 4ccountant-backup-YYYY-MM-DD_HH-MM-SS.json)
        path: Option<PathBuf>,
    },

    /// Restore data from a backup file
    Restore {
        /// Path to backup JSON file
        path: PathBuf,
    },
}

fn open_db() -> Database {
    Database::open_default().unwrap_or_else(|e| {
        eprintln!("{} {}", "Error:".red().bold(), e);
        std::process::exit(1);
    })
}


// ── LLM Conf ──

fn cmd_llm_conf() {
    let db = open_db();

    // Show current config
    let current_provider = db.get_config("llm_provider").ok().flatten();
    let current_key = db.get_config("llm_api_key").ok().flatten();

    if let Some(ref p) = current_provider {
        if !p.is_empty() {
            println!(
                "{} {} (key: {})",
                "Current provider:".dimmed(),
                p.green(),
                current_key
                    .as_deref()
                    .map(|k| if k.len() > 8 {
                        format!("{}...{}", &k[..4], &k[k.len()-4..])
                    } else {
                        k.to_string()
                    })
                    .unwrap_or_else(|| "not set".to_string())
                    .dimmed()
            );
        }
    }

    let providers = ["openai", "anthropic", "ollama"];
    let selection = dialoguer::Select::new()
        .with_prompt("Select provider")
        .items(&providers)
        .default(0)
        .interact()
        .unwrap_or_else(|_| std::process::exit(0));

    let key = if providers[selection] == "ollama" {
        dialoguer::Input::<String>::new()
            .with_prompt("Endpoint URL")
            .default("http://localhost:11434".to_string())
            .interact_text()
            .unwrap_or_else(|_| std::process::exit(0))
    } else {
        dialoguer::Password::new()
            .with_prompt("API Key")
            .interact()
            .unwrap_or_else(|_| std::process::exit(0))
    };

    // Validate
    let provider_name = providers[selection];
    print!("{}", "Validating... ".dimmed());
    match create_provider(provider_name) {
        Some(provider) => {
            let config = LlmConfig {
                provider: provider_name.to_string(),
                api_key: key.clone(),
            };
            match provider.validate(&config) {
                Ok(()) => {
                    println!("{}", "OK".green().bold());
                    db.set_config("llm_provider", provider_name).unwrap();
                    db.set_config("llm_api_key", &key).unwrap();
                    println!("{}", "Configuration saved.".green());
                }
                Err(e) => {
                    println!("{}", "FAILED".red().bold());
                    eprintln!("{} {}", "Validation error:".red(), e);
                }
            }
        }
        None => {
            println!("{}", "FAILED".red().bold());
            eprintln!("{}", "Unknown provider".red());
        }
    }
}

// ── Bulk Insert ──

fn cmd_bulk_insert(path: PathBuf) {
    let db = open_db();

    let content = std::fs::read_to_string(&path).unwrap_or_else(|e| {
        eprintln!("{} Cannot read {}: {}", "Error:".red().bold(), path.display(), e);
        std::process::exit(1);
    });

    let all_parsers = parsers::builtin_parsers();
    let parser = parsers::detect_parser(&content, &all_parsers).unwrap_or_else(|| {
        eprintln!("{} Could not detect file format.", "Error:".red().bold());
        std::process::exit(1);
    });

    println!("{} {}", "Detected format:".dimmed(), parser.name().green());

    // Preview
    let preview = parser.preview_rows(&content).unwrap_or_else(|e| {
        eprintln!("{} {}", "Error:".red().bold(), e);
        std::process::exit(1);
    });

    if preview.is_empty() {
        eprintln!("{} File appears empty.", "Error:".red().bold());
        std::process::exit(1);
    }

    let headers = &preview[0];
    println!("\n{}", "Columns detected:".dimmed());
    for (i, h) in headers.iter().enumerate() {
        println!("  {} {}", format!("[{}]", i).dimmed(), h);
    }

    // Auto-detect or prompt for column mapping
    let mut title_idx = 0usize;
    let mut amount_idx = 1usize;
    let mut date_idx = 2usize;

    for (i, h) in headers.iter().enumerate() {
        let lower = h.to_lowercase();
        if lower.contains("title") || lower.contains("description") || lower.contains("name")
            || lower.contains("merchant") || lower.contains("opis") || lower.contains("tytuł")
        {
            title_idx = i;
        }
        if lower.contains("amount") || lower.contains("value") || lower.contains("sum")
            || lower.contains("kwota")
        {
            amount_idx = i;
        }
        if lower.contains("date") || lower.contains("data") {
            date_idx = i;
        }
    }

    println!(
        "\n{} title={}, amount={}, date={}",
        "Auto-detected mapping:".dimmed(),
        headers.get(title_idx).unwrap_or(&"?".to_string()).green(),
        headers.get(amount_idx).unwrap_or(&"?".to_string()).green(),
        headers.get(date_idx).unwrap_or(&"?".to_string()).green(),
    );

    if !dialoguer::Confirm::new()
        .with_prompt("Use this mapping?")
        .default(true)
        .interact()
        .unwrap_or(true)
    {
        title_idx = dialoguer::Input::<usize>::new()
            .with_prompt("Title column index")
            .default(title_idx)
            .interact_text()
            .unwrap_or(title_idx);
        amount_idx = dialoguer::Input::<usize>::new()
            .with_prompt("Amount column index")
            .default(amount_idx)
            .interact_text()
            .unwrap_or(amount_idx);
        date_idx = dialoguer::Input::<usize>::new()
            .with_prompt("Date column index")
            .default(date_idx)
            .interact_text()
            .unwrap_or(date_idx);
    }

    let date_format: String = dialoguer::Input::new()
        .with_prompt("Date format")
        .default("%Y-%m-%d".to_string())
        .interact_text()
        .unwrap_or_else(|_| "%Y-%m-%d".to_string());

    let mapping = ColumnMapping {
        title_index: title_idx,
        amount_index: amount_idx,
        date_index: date_idx,
        date_format,
    };

    // Parse
    let parsed = parser.parse(&content, &mapping).unwrap_or_else(|e| {
        eprintln!("{} {}", "Parse error:".red().bold(), e);
        std::process::exit(1);
    });

    println!("\n{} {} expenses parsed.", "OK".green().bold(), parsed.len());

    // Classify with DB rules
    let rules = db.get_all_rules().unwrap_or_default();
    let regex_classifier = RegexClassifier::from_rules(&rules);
    let classifiers: Vec<Box<dyn accountant_core::classifiers::Classifier>> =
        vec![Box::new(regex_classifier)];
    let classified = classify_pipeline(&parsed, &classifiers);

    // Batch duplicate check
    let dup_inputs: Vec<(&str, f64, &chrono::NaiveDate)> = classified
        .iter()
        .map(|(e, _)| (e.title.as_str(), e.amount, &e.date))
        .collect();
    let dup_flags = db.check_duplicates_batch(&dup_inputs).unwrap_or_else(|e| {
        eprintln!("{} Duplicate check failed: {}", "Warning:".yellow(), e);
        vec![false; classified.len()]
    });

    let mut results: Vec<(String, f64, String, Option<String>, Option<String>, bool)> = Vec::new();
    for ((expense, result), &is_dup) in classified.iter().zip(dup_flags.iter()) {
        let (cat, source) = match result {
            Some(cr) => (Some(cr.category.clone()), Some(cr.source.to_string())),
            None => (None, None),
        };
        results.push((
            expense.title.clone(),
            expense.amount,
            expense.date.to_string(),
            cat,
            source,
            is_dup,
        ));
    }

    // LLM fallback
    let llm_provider_name = db.get_config("llm_provider").ok().flatten();
    let llm_api_key = db.get_config("llm_api_key").ok().flatten();

    if let (Some(ref pname), Some(ref pkey)) = (&llm_provider_name, &llm_api_key) {
        if !pname.is_empty() && !pkey.is_empty() {
            let unclassified_indices: Vec<usize> = results.iter().enumerate()
                .filter(|(_, r)| r.3.is_none() && !r.5)
                .map(|(i, _)| i)
                .collect();

            if !unclassified_indices.is_empty() {
                println!("{}", "Classifying with LLM...".dimmed());
                if let Some(provider) = create_provider(pname) {
                    let config = LlmConfig { provider: pname.clone(), api_key: pkey.clone() };
                    let categories = db.get_all_categories().unwrap_or_default();
                    let unclassified_expenses: Vec<_> = unclassified_indices.iter()
                        .map(|&i| accountant_core::models::ParsedExpense {
                            title: results[i].0.clone(),
                            amount: results[i].1,
                            date: chrono::NaiveDate::parse_from_str(&results[i].2, "%Y-%m-%d")
                                .unwrap_or_else(|_| chrono::Local::now().date_naive()),
                        })
                        .collect();

                    match provider.classify_batch(&unclassified_expenses, &categories, &config) {
                        Ok(llm_results) => {
                            let mut llm_count = 0;
                            for (idx, llm_result) in unclassified_indices.iter().zip(llm_results.into_iter()) {
                                if let Some(classification) = llm_result {
                                    results[*idx].3 = Some(classification.category);
                                    results[*idx].4 = Some(ClassificationSource::Llm.to_string());
                                    llm_count += 1;
                                }
                            }
                            println!("  {} classified {} expenses.", "LLM".purple(), llm_count);
                        }
                        Err(e) => {
                            eprintln!("  {} LLM classification failed: {}", "Warning:".yellow(), e);
                        }
                    }
                }
            }
        }
    } else {
        let unclassified_count = results.iter().filter(|r| r.3.is_none() && !r.5).count();
        if unclassified_count > 0 {
            println!(
                "{} {} expenses unclassified. Configure LLM with '4ccountant llm-conf' for AI classification.",
                "Note:".yellow(),
                unclassified_count
            );
        }
    }

    // Display results table
    let mut table = comfy_table::Table::new();
    table.set_header(vec!["Date", "Title", "Amount", "Category", "Source"]);
    table.load_preset(comfy_table::presets::UTF8_BORDERS_ONLY);

    let mut dup_count = 0;
    let mut new_count = 0;

    for r in &results {
        if r.5 {
            dup_count += 1;
            continue;
        }
        new_count += 1;
        table.add_row(vec![
            r.2.clone(),
            if r.0.len() > 40 { format!("{}...", &r.0[..37]) } else { r.0.clone() },
            format!("{:.2}", r.1),
            r.3.clone().unwrap_or_else(|| "-".to_string()),
            r.4.clone().unwrap_or_else(|| "Unclassified".to_string()),
        ]);
    }

    println!("\n{}", table);
    if dup_count > 0 {
        println!("{} {} duplicates will be skipped.", "Note:".yellow(), dup_count);
    }

    if new_count == 0 {
        println!("{}", "No new expenses to save.".dimmed());
        return;
    }

    if !dialoguer::Confirm::new()
        .with_prompt(format!("Save {} expenses?", new_count))
        .default(true)
        .interact()
        .unwrap_or(false)
    {
        println!("{}", "Cancelled.".dimmed());
        return;
    }

    // Save
    let mut to_insert: Vec<Expense> = Vec::new();
    let mut rules_to_save: Vec<ClassificationRule> = Vec::new();

    for r in &results {
        if r.5 { continue; }
        let date = chrono::NaiveDate::parse_from_str(&r.2, "%Y-%m-%d")
            .unwrap_or_else(|_| chrono::Local::now().date_naive());
        let source = r.4.as_deref()
            .and_then(ClassificationSource::from_str_opt)
            .unwrap_or(ClassificationSource::Manual);
        to_insert.push(Expense {
            id: None,
            title: r.0.clone(),
            amount: r.1,
            date,
            category: r.3.clone(),
            classification_source: Some(source),
        });
        if let Some(ref cat) = r.3 {
            if !cat.is_empty() {
                rules_to_save.push(ClassificationRule::from_pattern(&r.0, cat));
            }
        }
    }

    let filename = path.file_name().map(|n| n.to_string_lossy().to_string());
    match db.insert_expenses_bulk(&to_insert, filename.as_deref(), &rules_to_save) {
        Ok(count) => {
            println!("{} {} expenses saved.", "OK".green().bold(), count);
        }
        Err(e) => {
            eprintln!("{} {}", "Error:".red().bold(), e);
            std::process::exit(1);
        }
    }
}

// ── Backup ──

fn cmd_backup(path: Option<PathBuf>) {
    let db = open_db();

    let path = path.unwrap_or_else(|| {
        let ts = chrono::Local::now().format("%Y-%m-%d_%H-%M-%S");
        PathBuf::from(format!("4ccountant-backup-{}.json", ts))
    });

    let backup = create_backup(&db).unwrap_or_else(|e| {
        eprintln!("{} {}", "Error:".red().bold(), e);
        std::process::exit(1);
    });

    let json = serde_json::to_string_pretty(&backup).unwrap_or_else(|e| {
        eprintln!("{} {}", "Error:".red().bold(), e);
        std::process::exit(1);
    });

    std::fs::write(&path, json).unwrap_or_else(|e| {
        eprintln!("{} Cannot write {}: {}", "Error:".red().bold(), path.display(), e);
        std::process::exit(1);
    });

    println!(
        "{} Backup saved to {} ({} expenses, {} rules, {} budgets)",
        "OK".green().bold(),
        path.display(),
        backup.expenses.len(),
        backup.classification_rules.len(),
        backup.budgets.len(),
    );
}

// ── Restore ──

fn cmd_restore(path: PathBuf) {
    let db = open_db();

    let content = std::fs::read_to_string(&path).unwrap_or_else(|e| {
        eprintln!("{} Cannot read {}: {}", "Error:".red().bold(), path.display(), e);
        std::process::exit(1);
    });

    let backup: accountant_core::backup::BackupData =
        serde_json::from_str(&content).unwrap_or_else(|e| {
            eprintln!("{} Invalid backup file: {}", "Error:".red().bold(), e);
            std::process::exit(1);
        });

    println!(
        "{} {} expenses, {} rules, {} budgets",
        "Backup contains:".dimmed(),
        backup.expenses.len(),
        backup.classification_rules.len(),
        backup.budgets.len(),
    );

    if !dialoguer::Confirm::new()
        .with_prompt("Restore this backup?")
        .default(true)
        .interact()
        .unwrap_or(false)
    {
        println!("{}", "Cancelled.".dimmed());
        return;
    }

    let summary = restore_backup(&db, &backup).unwrap_or_else(|e| {
        eprintln!("{} {}", "Error:".red().bold(), e);
        std::process::exit(1);
    });

    let mut table = comfy_table::Table::new();
    table.set_header(vec!["Data", "Inserted", "Skipped"]);
    table.load_preset(comfy_table::presets::UTF8_BORDERS_ONLY);
    table.add_row(vec![
        "Expenses".to_string(),
        summary.expenses_inserted.to_string(),
        summary.expenses_skipped.to_string(),
    ]);
    table.add_row(vec![
        "Classification rules".to_string(),
        summary.rules_upserted.to_string(),
        "-".to_string(),
    ]);
    table.add_row(vec![
        "Budgets".to_string(),
        summary.budgets_inserted.to_string(),
        summary.budgets_skipped.to_string(),
    ]);

    println!("\n{}", table);
    println!("{}", "Restore complete.".green().bold());
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::LlmConf => cmd_llm_conf(),
        Commands::BulkInsert { path } => cmd_bulk_insert(path),
        Commands::Backup { path } => cmd_backup(path),
        Commands::Restore { path } => cmd_restore(path),
    }
}
