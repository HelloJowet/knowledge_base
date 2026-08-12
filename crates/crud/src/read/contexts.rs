use crate::{Error, KnowledgeBaseRepository, filesystem};
use knowledge_base_models::EntityId;

#[derive(Clone, Copy, Debug)]
pub struct EntityContexts<'a> {
    repository: &'a KnowledgeBaseRepository,
}

impl<'a> EntityContexts<'a> {
    pub(crate) fn new(repository: &'a KnowledgeBaseRepository) -> Self {
        Self { repository }
    }

    pub fn read(&self, id: &EntityId) -> Result<String, Error> {
        filesystem::read(self.repository.root(), "entity_context", id.as_str(), "md")
    }
}
