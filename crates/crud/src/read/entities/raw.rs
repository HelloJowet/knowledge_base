use super::Entities;
use crate::{Error, filesystem};
use knowledge_base_models::EntityId;

impl Entities<'_> {
    pub fn read(&self, id: &EntityId) -> Result<String, Error> {
        filesystem::read(self.repository.root(), "entities", id.as_str(), "yaml")
    }
}
