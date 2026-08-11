use super::super::{CommandError, write_content};
use chrono::{SecondsFormat, Utc};
use knowledge_base_crud::{ApplyMode, KnowledgeBase, ReferenceDraft};
use std::process::ExitCode;

pub fn execute(
    knowledge_base: &KnowledgeBase,
    url: String,
    title: String,
    publisher: Option<String>,
    publication_date: Option<String>,
    source_language: Option<String>,
    archive_url: Option<String>,
    dry_run: bool,
) -> Result<ExitCode, CommandError> {
    let draft = ReferenceDraft {
        url,
        title,
        publisher,
        publication_date,
        source_language,
        retrieved_at: Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true),
        archive_url,
    };
    let mode = if dry_run { ApplyMode::Preview } else { ApplyMode::Commit };
    let outcome = knowledge_base.references().register(&draft, mode)?;
    let output = serde_yaml::to_string(&outcome).map_err(CommandError::Serialization)?;
    write_content(&output)
}
