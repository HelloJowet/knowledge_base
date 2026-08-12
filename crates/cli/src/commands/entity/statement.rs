use super::super::{CommandError, write_content};
use clap::Subcommand;
use knowledge_base_crud::KnowledgeBaseRepository;
use knowledge_base_crud::write::{StatementBatch, WriteMode};
use std::path::PathBuf;
use std::process::ExitCode;

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Add statements from a strict YAML manifest.
    Apply {
        /// Path to the statement manifest.
        manifest: PathBuf,
        /// Validate and report the changes without writing entity files.
        #[arg(long)]
        dry_run: bool,
    },
}

pub fn execute(command: Command, knowledge_base: &KnowledgeBaseRepository) -> Result<ExitCode, CommandError> {
    match command {
        Command::Apply { manifest, dry_run } => {
            let batch = StatementBatch::read(manifest)?;
            let mode = if dry_run { WriteMode::Preview } else { WriteMode::Commit };
            let outcome = knowledge_base.write().statements().apply(&batch, mode)?;
            let rejected = outcome.was_rejected();
            let output = serde_yaml::to_string(&outcome).map_err(CommandError::Serialization)?;
            write_content(&output)?;
            Ok(if rejected { ExitCode::FAILURE } else { ExitCode::SUCCESS })
        }
    }
}
