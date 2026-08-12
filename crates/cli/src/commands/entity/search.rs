use super::super::{CommandError, write_content};
use knowledge_base_crud::KnowledgeBaseRepository;
use std::process::ExitCode;

pub fn execute(repository: &KnowledgeBaseRepository, query: &str, limit: usize, offset: usize) -> Result<ExitCode, CommandError> {
    let page = repository.read().entities().search(query, limit, offset)?;
    let output = serde_yaml::to_string(&page).map_err(CommandError::Serialization)?;
    write_content(&output)
}
