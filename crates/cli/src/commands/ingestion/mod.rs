mod candidate_inventory;
mod retrieval;

use clap::Subcommand;
use std::process::ExitCode;

use super::CommandError;

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Work with ingestion candidate inventories.
    CandidateInventory {
        #[command(subcommand)]
        command: candidate_inventory::Command,
    },
    /// Fetch and register webpage retrieval bundles.
    Retrieval {
        #[command(subcommand)]
        command: retrieval::Command,
    },
}

pub fn requires_knowledge_base(command: &Command) -> bool {
    match command {
        Command::CandidateInventory { .. } => true,
        Command::Retrieval { command } => retrieval::requires_knowledge_base(command),
    }
}

pub fn execute(command: Command, repository: Option<&knowledge_base_crud::KnowledgeBaseRepository>) -> Result<ExitCode, CommandError> {
    match command {
        Command::CandidateInventory { command } => candidate_inventory::execute(command, repository.expect("candidate inventory command requires a knowledge base").root()),
        Command::Retrieval { command } => retrieval::execute(command, repository),
    }
}
