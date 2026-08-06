use clap::{Parser, Subcommand};
use knowledge_base_validation::validate_repository;
use std::path::PathBuf;
use std::process::ExitCode;

#[derive(Debug, Parser)]
#[command(name = "knowledge-base", version, about)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Validate a knowledge-base directory.
    Validate {
        /// Root directory containing the knowledge-base files.
        path: PathBuf,
    },
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    match cli.command {
        Command::Validate { path } => {
            let diagnostics = validate_repository(&path);
            if diagnostics.is_empty() {
                println!("valid knowledge base: {}", path.display());
                ExitCode::SUCCESS
            } else {
                for diagnostic in diagnostics {
                    eprintln!("{diagnostic}");
                }
                ExitCode::FAILURE
            }
        }
    }
}
