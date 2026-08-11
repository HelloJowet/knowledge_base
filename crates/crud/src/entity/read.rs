use super::Entities;
use crate::{Error, resource};
use knowledge_base_models::EntityId;

impl Entities<'_> {
    pub fn read(&self, id: &EntityId) -> Result<String, Error> {
        resource::read(self.knowledge_base.root(), "entities", id.as_str(), "yaml")
    }
}
