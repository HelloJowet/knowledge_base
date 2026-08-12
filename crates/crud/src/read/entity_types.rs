use crate::{Error, KnowledgeBaseRepository, filesystem};
use knowledge_base_models::EntityTypeId;

#[derive(Clone, Copy, Debug)]
pub struct EntityTypes<'a> {
    repository: &'a KnowledgeBaseRepository,
}

impl<'a> EntityTypes<'a> {
    pub(crate) fn new(repository: &'a KnowledgeBaseRepository) -> Self {
        Self { repository }
    }

    pub fn read(&self, id: &EntityTypeId) -> Result<String, Error> {
        filesystem::read(self.repository.root(), "entity_types", id.as_str(), "yaml")
    }
}
