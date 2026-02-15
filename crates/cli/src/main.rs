use clap::{Parser, Subcommand};
use std::path::PathBuf;

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

    /// Insert a single expense
    Insert,

    /// Export classified expenses
    Export {
        /// Optional path to grammar/config file defining export columns
        grammar: Option<PathBuf>,
    },

    /// Open the GUI dashboard
    Dashboard,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::LlmConf => {
            println!("LLM configuration - not yet implemented");
        }
        Commands::BulkInsert { path } => {
            println!("Bulk insert from: {} - not yet implemented", path.display());
        }
        Commands::Insert => {
            println!("Single expense insert - not yet implemented");
        }
        Commands::Export { grammar } => {
            match grammar {
                Some(path) => println!("Export with grammar: {} - not yet implemented", path.display()),
                None => println!("Interactive export - not yet implemented"),
            }
        }
        Commands::Dashboard => {
            println!("Opening dashboard - not yet implemented");
        }
    }
}
