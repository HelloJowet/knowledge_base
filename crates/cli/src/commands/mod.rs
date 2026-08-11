pub mod entity;
pub mod entity_context;
pub mod entity_type;
pub mod property;
pub mod reference;
mod validate;

use clap::Subcommand;
use knowledge_base_crud::{CrudError, KnowledgeBase};
use std::fmt;
use std::io::{self, Write};
use std::path::Path;
use std::process::ExitCode;

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Validate the configured knowledge base.
    Validate,
    /// Work with entities.
    Entity {
        #[command(subcommand)]
        command: entity::Command,
    },
    /// Work with entity types.
    EntityType {
        #[command(subcommand)]
        command: entity_type::Command,
    },
    /// Work with properties.
    Property {
        #[command(subcommand)]
        command: property::Command,
    },
    /// Work with references.
    Reference {
        #[command(subcommand)]
        command: reference::Command,
    },
    /// Work with entity context documents.
    EntityContext {
        #[command(subcommand)]
        command: entity_context::Command,
    },
}

#[derive(Debug)]
pub enum CommandError {
    Crud(CrudError),
    Output(io::Error),
}

impl fmt::Display for CommandError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Crud(error) => error.fmt(formatter),
            Self::Output(error) => write!(formatter, "cannot write command output: {error}"),
        }
    }
}

impl From<CrudError> for CommandError {
    fn from(error: CrudError) -> Self {
        Self::Crud(error)
    }
}

pub fn execute(command: Command, root: &Path) -> Result<ExitCode, CommandError> {
    let knowledge_base = KnowledgeBase::new(root);
    match command {
        Command::Validate => Ok(validate::execute(root)),
        Command::Entity { command } => entity::execute(command, &knowledge_base),
        Command::EntityType { command } => entity_type::execute(command, &knowledge_base),
        Command::Property { command } => property::execute(command, &knowledge_base),
        Command::Reference { command } => reference::execute(command, &knowledge_base),
        Command::EntityContext { command } => entity_context::execute(command, &knowledge_base),
    }
}

fn write_content(content: &str) -> Result<ExitCode, CommandError> {
    io::stdout().lock().write_all(content.as_bytes()).map_err(CommandError::Output)?;
    Ok(ExitCode::SUCCESS)
}
