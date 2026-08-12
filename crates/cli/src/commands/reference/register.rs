use super::super::{CommandError, write_content};
use chrono::{SecondsFormat, Utc};
use knowledge_base_crud::KnowledgeBaseRepository;
use knowledge_base_crud::write::{ReferenceDraft, WriteMode};
use std::process::ExitCode;

pub(super) struct Args {
    pub(super) url: String,
    pub(super) title: String,
    pub(super) publisher: Option<String>,
    pub(super) publication_date: Option<String>,
    pub(super) source_language: Option<String>,
    pub(super) archive_url: Option<String>,
    pub(super) dry_run: bool,
}

pub fn execute(repository: &KnowledgeBaseRepository, args: Args) -> Result<ExitCode, CommandError> {
    let draft = ReferenceDraft {
        url: args.url,
        title: args.title,
        publisher: args.publisher,
        publication_date: args.publication_date,
        source_language: args.source_language,
        retrieved_at: Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true),
        archive_url: args.archive_url,
    };
    let mode = if args.dry_run { WriteMode::Preview } else { WriteMode::Commit };
    let outcome = repository.write().references().register(&draft, mode)?;
    let output = serde_yaml::to_string(&outcome).map_err(CommandError::Serialization)?;
    write_content(&output)
}
