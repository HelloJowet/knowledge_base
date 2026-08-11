mod query;
mod read;
mod relationships;
mod search;
mod statement;

use super::CommandError;
use clap::Subcommand;
use knowledge_base_crud::KnowledgeBase;
use knowledge_base_models::EntityId;
use std::num::NonZeroUsize;
use std::process::ExitCode;

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Read an entity by identifier.
    Read { id: EntityId },
    /// Find entities whose statements match every filter.
    Query {
        /// Property/value filter in P<n>=value syntax. Repeat for AND matching.
        #[arg(long = "filter", required = true)]
        filters: Vec<String>,
        /// Maximum number of entities to return.
        #[arg(long, default_value = "100")]
        limit: NonZeroUsize,
        /// Number of matching entities to skip.
        #[arg(long, default_value_t = 0)]
        offset: usize,
    },
    /// Search localized entity labels.
    Search {
        /// Case-insensitive label text to search for.
        query: String,
        /// Maximum number of entities to return.
        #[arg(long, default_value = "100")]
        limit: NonZeroUsize,
        /// Number of matching entities to skip.
        #[arg(long, default_value_t = 0)]
        offset: usize,
    },
    /// Show direct incoming and outgoing entity relationships.
    Relationships {
        id: EntityId,
        /// Maximum number of relationships to return.
        #[arg(long, default_value = "100")]
        limit: NonZeroUsize,
        /// Number of relationships to skip.
        #[arg(long, default_value_t = 0)]
        offset: usize,
    },
    /// Work with entity statements.
    Statement {
        #[command(subcommand)]
        command: statement::Command,
    },
}

pub fn execute(command: Command, knowledge_base: &KnowledgeBase) -> Result<ExitCode, CommandError> {
    match command {
        Command::Read { id } => read::execute(knowledge_base, &id),
        Command::Query { filters, limit, offset } => query::execute(knowledge_base, &filters, limit.get(), offset),
        Command::Search { query, limit, offset } => search::execute(knowledge_base, &query, limit.get(), offset),
        Command::Relationships { id, limit, offset } => relationships::execute(knowledge_base, &id, limit.get(), offset),
        Command::Statement { command } => statement::execute(command, knowledge_base),
    }
}
