use super::super::{CommandError, write_content};
use knowledge_base_crud::KnowledgeBaseRepository;
use knowledge_base_models::ReferenceId;
use std::process::ExitCode;

pub fn execute(repository: &KnowledgeBaseRepository, id: &ReferenceId) -> Result<ExitCode, CommandError> {
    write_content(&repository.read().references().read(id)?)
}
