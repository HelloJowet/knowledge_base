use super::super::{CommandError, write_content};
use knowledge_base_crud::KnowledgeBaseRepository;
use knowledge_base_models::EntityId;
use std::process::ExitCode;

pub fn execute(repository: &KnowledgeBaseRepository, id: &EntityId) -> Result<ExitCode, CommandError> {
    write_content(&repository.read().entity_contexts().read(id)?)
}
