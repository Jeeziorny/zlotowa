use accountant_core::db::Database;
use accountant_core::models::{ClassificationSource, Expense};
use serde::{Deserialize, Serialize};
use tauri::State;
use std::sync::Mutex;

pub struct AppState {
    pub db: Mutex<Database>,
}

#[derive(Serialize, Deserialize)]
pub struct ExpenseInput {
    pub title: String,
    pub amount: f64,
    pub date: String,
    pub category: Option<String>,
}

#[tauri::command]
fn get_expenses(state: State<AppState>) -> Result<Vec<Expense>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.get_all_expenses().map_err(|e| e.to_string())
}

#[tauri::command]
fn add_expense(state: State<AppState>, input: ExpenseInput) -> Result<i64, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let date = chrono::NaiveDate::parse_from_str(&input.date, "%Y-%m-%d")
        .map_err(|e| e.to_string())?;
    let expense = Expense {
        id: None,
        title: input.title,
        amount: input.amount,
        date,
        category: input.category,
        classification_source: Some(ClassificationSource::Manual),
    };
    db.insert_expense(&expense).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_categories(state: State<AppState>) -> Result<Vec<String>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.get_all_categories().map_err(|e| e.to_string())
}

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
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
