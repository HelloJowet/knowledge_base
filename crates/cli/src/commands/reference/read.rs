use super::super::{CommandError, write_content};
use knowledge_base_crud::KnowledgeBase;
use knowledge_base_models::ReferenceId;
use std::process::ExitCode;

pub fn execute(knowledge_base: &KnowledgeBase, id: &ReferenceId) -> Result<ExitCode, CommandError> {
    write_content(&knowledge_base.read_reference(id)?)
}
