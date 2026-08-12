mod read;

use super::CommandError;
use clap::Subcommand;
use knowledge_base_crud::KnowledgeBaseRepository;
use knowledge_base_models::EntityTypeId;
use std::process::ExitCode;

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Read an entity type by identifier.
    Read { id: EntityTypeId },
}

pub fn execute(command: Command, knowledge_base: &KnowledgeBaseRepository) -> Result<ExitCode, CommandError> {
    match command {
        Command::Read { id } => read::execute(knowledge_base, &id),
    }
}
