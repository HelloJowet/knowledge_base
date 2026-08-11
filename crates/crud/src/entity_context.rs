use crate::{Error, KnowledgeBase, resource};
use knowledge_base_models::EntityId;

#[derive(Clone, Copy, Debug)]
pub struct EntityContexts<'a> {
    knowledge_base: &'a KnowledgeBase,
}

impl<'a> EntityContexts<'a> {
    pub(crate) fn new(knowledge_base: &'a KnowledgeBase) -> Self {
        Self { knowledge_base }
    }

    pub fn read(&self, id: &EntityId) -> Result<String, Error> {
        resource::read(self.knowledge_base.root(), "entity_context", id.as_str(), "md")
    }
}
