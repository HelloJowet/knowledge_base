mod read;
mod statement;

use super::CommandError;
use clap::Subcommand;
use knowledge_base_crud::KnowledgeBase;
use knowledge_base_models::EntityId;
use std::process::ExitCode;

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Read an entity by identifier.
    Read { id: EntityId },
    /// Work with entity statements.
    Statement {
        #[command(subcommand)]
        command: statement::Command,
    },
}

pub fn execute(command: Command, knowledge_base: &KnowledgeBase) -> Result<ExitCode, CommandError> {
    match command {
        Command::Read { id } => read::execute(knowledge_base, &id),
        Command::Statement { command } => statement::execute(command, knowledge_base),
    }
}
