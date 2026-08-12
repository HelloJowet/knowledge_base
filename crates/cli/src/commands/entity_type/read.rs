use super::super::{CommandError, write_content};
use knowledge_base_crud::KnowledgeBaseRepository;
use knowledge_base_models::EntityTypeId;
use std::process::ExitCode;

pub fn execute(repository: &KnowledgeBaseRepository, id: &EntityTypeId) -> Result<ExitCode, CommandError> {
    write_content(&repository.read().entity_types().read(id)?)
}
