mod read;

use super::CommandError;
use clap::Subcommand;
use knowledge_base_crud::KnowledgeBase;
use knowledge_base_models::PropertyId;
use std::process::ExitCode;

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Read a property by identifier.
    Read { id: PropertyId },
}

pub fn execute(command: Command, knowledge_base: &KnowledgeBase) -> Result<ExitCode, CommandError> {
    match command {
        Command::Read { id } => read::execute(knowledge_base, &id),
    }
}
