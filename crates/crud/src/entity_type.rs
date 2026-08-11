use crate::{Error, KnowledgeBase, resource};
use knowledge_base_models::EntityTypeId;

#[derive(Clone, Copy, Debug)]
pub struct EntityTypes<'a> {
    knowledge_base: &'a KnowledgeBase,
}

impl<'a> EntityTypes<'a> {
    pub(crate) fn new(knowledge_base: &'a KnowledgeBase) -> Self {
        Self { knowledge_base }
    }

    pub fn read(&self, id: &EntityTypeId) -> Result<String, Error> {
        resource::read(self.knowledge_base.root(), "entity_types", id.as_str(), "yaml")
    }
}
