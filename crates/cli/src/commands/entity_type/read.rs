use super::super::{CommandError, write_content};
use knowledge_base_crud::KnowledgeBase;
use knowledge_base_models::EntityTypeId;
use std::process::ExitCode;

pub fn execute(knowledge_base: &KnowledgeBase, id: &EntityTypeId) -> Result<ExitCode, CommandError> {
    write_content(&knowledge_base.entity_types().read(id)?)
}
