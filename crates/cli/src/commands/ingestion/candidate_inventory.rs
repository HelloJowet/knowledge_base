use clap::Subcommand;
use knowledge_base_ingestion_candidate_inventory::validate_ingestion_candidate_inventory;
use knowledge_base_snapshot::RepositorySnapshot;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use crate::commands::CommandError;

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Validate an ingestion candidate inventory.
    Validate {
        /// Path to ingestion_candidate_inventory.yaml.
        path: PathBuf,
    },
}

pub fn execute(command: Command, root: &Path) -> Result<ExitCode, CommandError> {
    match command {
        Command::Validate { path } => validate(&path, root),
    }
}

fn validate(path: &Path, root: &Path) -> Result<ExitCode, CommandError> {
    let snapshot = RepositorySnapshot::load(root).map_err(CommandError::Snapshot)?;
    let diagnostics = validate_ingestion_candidate_inventory(path, &snapshot);
    if diagnostics.is_empty() {
        println!("valid ingestion candidate inventory: {}", path.display());
        Ok(ExitCode::SUCCESS)
    } else {
        for diagnostic in diagnostics {
            eprintln!("{diagnostic}");
        }
        Ok(ExitCode::FAILURE)
    }
}
