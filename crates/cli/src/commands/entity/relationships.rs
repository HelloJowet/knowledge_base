use super::super::{CommandError, write_content};
use knowledge_base_crud::KnowledgeBaseRepository;
use knowledge_base_models::EntityId;
use std::process::ExitCode;

pub fn execute(repository: &KnowledgeBaseRepository, id: &EntityId, limit: usize, offset: usize) -> Result<ExitCode, CommandError> {
    let page = repository.read().entities().relationships(id, limit, offset)?;
    let output = serde_yaml::to_string(&page).map_err(CommandError::Serialization)?;
    write_content(&output)
}
