use crate::{Error, KnowledgeBase, resource};
use knowledge_base_models::PropertyId;

#[derive(Clone, Copy, Debug)]
pub struct Properties<'a> {
    knowledge_base: &'a KnowledgeBase,
}

impl<'a> Properties<'a> {
    pub(crate) fn new(knowledge_base: &'a KnowledgeBase) -> Self {
        Self { knowledge_base }
    }

    pub fn read(&self, id: &PropertyId) -> Result<String, Error> {
        resource::read(self.knowledge_base.root(), "properties", id.as_str(), "yaml")
    }
}
