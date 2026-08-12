pub mod entity;
pub mod entity_context;
pub mod entity_type;
pub mod ingestion;
pub mod property;
pub mod reference;
mod validate;

use clap::Subcommand;
use knowledge_base_crud::{KnowledgeBaseRepository, RepositoryError};
use std::fmt;
use std::io::{self, Write};
use std::process::ExitCode;

#[derive(Debug, clap::Parser)]
#[command(name = "knowledge-base", version, about)]
pub(crate) struct Cli {
    #[command(subcommand)]
    pub(crate) command: Command,
}

#[derive(Debug, Subcommand)]
pub(crate) enum Command {
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
    /// Work with ingestion artifacts.
    Ingestion {
        #[command(subcommand)]
        command: ingestion::Command,
    },
}

#[derive(Debug)]
pub enum CommandError {
    Crud(RepositoryError),
    Retrieval(anyhow::Error),
    InvalidFilter(String),
    Serialization(serde_yaml::Error),
    Output(io::Error),
    Snapshot(knowledge_base_snapshot::Error),
}

impl fmt::Display for CommandError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Crud(error) => error.fmt(formatter),
            Self::Retrieval(error) => error.fmt(formatter),
            Self::InvalidFilter(message) => formatter.write_str(message),
            Self::Serialization(error) => write!(formatter, "cannot serialize command output: {error}"),
            Self::Output(error) => write!(formatter, "cannot write command output: {error}"),
            Self::Snapshot(error) => error.fmt(formatter),
        }
    }
}

impl From<RepositoryError> for CommandError {
    fn from(error: RepositoryError) -> Self {
        Self::Crud(error)
    }
}

impl From<anyhow::Error> for CommandError {
    fn from(error: anyhow::Error) -> Self {
        Self::Retrieval(error)
    }
}

impl Command {
    pub(crate) fn requires_knowledge_base(&self) -> bool {
        !matches!(self, Self::Ingestion { command } if !ingestion::requires_knowledge_base(command))
    }
}

pub(crate) struct RepositoryContext<'a> {
    pub(crate) repository: &'a KnowledgeBaseRepository,
}

pub(crate) fn execute(command: Command, context: Option<&RepositoryContext<'_>>) -> Result<ExitCode, CommandError> {
    match command {
        Command::Validate => Ok(validate::execute(context.expect("validated command requires a knowledge base").repository)),
        Command::Entity { command } => entity::execute(command, context.expect("entity command requires a knowledge base").repository),
        Command::EntityType { command } => entity_type::execute(command, context.expect("entity type command requires a knowledge base").repository),
        Command::Property { command } => property::execute(command, context.expect("property command requires a knowledge base").repository),
        Command::Reference { command } => reference::execute(command, context.expect("reference command requires a knowledge base").repository),
        Command::EntityContext { command } => entity_context::execute(command, context.expect("entity context command requires a knowledge base").repository),
        Command::Ingestion { command } => ingestion::execute(command, context.map(|context| context.repository)),
    }
}

fn write_content(content: &str) -> Result<ExitCode, CommandError> {
    io::stdout().lock().write_all(content.as_bytes()).map_err(CommandError::Output)?;
    Ok(ExitCode::SUCCESS)
}
