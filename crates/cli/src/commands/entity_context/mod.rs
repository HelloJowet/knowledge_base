mod read;

use super::CommandError;
use clap::Subcommand;
use knowledge_base_crud::KnowledgeBase;
use knowledge_base_models::EntityId;
use std::process::ExitCode;

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Read an entity context document by entity identifier.
    Read { id: EntityId },
}

pub fn execute(command: Command, knowledge_base: &KnowledgeBase) -> Result<ExitCode, CommandError> {
    match command {
        Command::Read { id } => read::execute(knowledge_base, &id),
    }
}
