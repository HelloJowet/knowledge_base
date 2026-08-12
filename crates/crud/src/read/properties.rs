use crate::{Error, KnowledgeBaseRepository, filesystem};
use knowledge_base_models::PropertyId;

#[derive(Clone, Copy, Debug)]
pub struct Properties<'a> {
    repository: &'a KnowledgeBaseRepository,
}

impl<'a> Properties<'a> {
    pub(crate) fn new(repository: &'a KnowledgeBaseRepository) -> Self {
        Self { repository }
    }

    pub fn read(&self, id: &PropertyId) -> Result<String, Error> {
        filesystem::read(self.repository.root(), "properties", id.as_str(), "yaml")
    }
}
