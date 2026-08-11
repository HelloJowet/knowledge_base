mod read;
mod register;

use super::CommandError;
use clap::Subcommand;
use knowledge_base_crud::KnowledgeBase;
use knowledge_base_models::ReferenceId;
use std::process::ExitCode;

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Read a reference by identifier.
    Read { id: ReferenceId },
    /// Register or reuse a reference by its canonical URL.
    Register {
        /// Canonical source URL.
        #[arg(long)]
        url: String,
        /// Human-readable source title.
        #[arg(long)]
        title: String,
        /// Source publisher.
        #[arg(long)]
        publisher: Option<String>,
        /// Publication date in YYYY, YYYY-MM, or YYYY-MM-DD form.
        #[arg(long)]
        publication_date: Option<String>,
        /// Source language as a BCP 47 tag.
        #[arg(long)]
        source_language: Option<String>,
        /// Archived copy URL.
        #[arg(long)]
        archive_url: Option<String>,
        /// Validate and report the registration without writing files.
        #[arg(long)]
        dry_run: bool,
    },
}

pub fn execute(command: Command, knowledge_base: &KnowledgeBase) -> Result<ExitCode, CommandError> {
    match command {
        Command::Read { id } => read::execute(knowledge_base, &id),
        Command::Register {
            url,
            title,
            publisher,
            publication_date,
            source_language,
            archive_url,
            dry_run,
        } => register::execute(
            knowledge_base,
            register::Args {
                url,
                title,
                publisher,
                publication_date,
                source_language,
                archive_url,
                dry_run,
            },
        ),
    }
}
