use crate::{Error, KnowledgeBase, resource};
use knowledge_base_models::ReferenceId;

#[derive(Clone, Copy, Debug)]
pub struct References<'a> {
    knowledge_base: &'a KnowledgeBase,
}

impl<'a> References<'a> {
    pub(crate) fn new(knowledge_base: &'a KnowledgeBase) -> Self {
        Self { knowledge_base }
    }

    pub fn read(&self, id: &ReferenceId) -> Result<String, Error> {
        resource::read(self.knowledge_base.root(), "references", id.as_str(), "yaml")
    }
}
