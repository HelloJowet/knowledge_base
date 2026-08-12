use crate::{Error, KnowledgeBaseRepository, filesystem};
use knowledge_base_models::ReferenceId;

#[derive(Clone, Copy, Debug)]
pub struct References<'a> {
    repository: &'a KnowledgeBaseRepository,
}

impl<'a> References<'a> {
    pub(crate) fn new(repository: &'a KnowledgeBaseRepository) -> Self {
        Self { repository }
    }
    pub fn read(&self, id: &ReferenceId) -> Result<String, Error> {
        filesystem::read(self.repository.root(), "references", id.as_str(), "yaml")
    }
}
