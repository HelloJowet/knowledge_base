use super::super::{CommandError, write_content};
use knowledge_base_crud::KnowledgeBase;
use std::process::ExitCode;

pub fn execute(knowledge_base: &KnowledgeBase, query: &str, limit: usize, offset: usize) -> Result<ExitCode, CommandError> {
    let page = knowledge_base.entities().search(query, limit, offset)?;
    let output = serde_yaml::to_string(&page).map_err(CommandError::Serialization)?;
    write_content(&output)
}
