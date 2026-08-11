use crate::{CrudError, KnowledgeBase};
use knowledge_base_models::{EntityId, EntityTypeId, PropertyId, ReferenceId};
use std::fs;
use std::path::Path;

impl KnowledgeBase {
    pub fn read_entity(&self, id: &EntityId) -> Result<String, CrudError> {
        self.read_file("entities", id.as_str(), "yaml")
    }

    pub fn read_entity_type(&self, id: &EntityTypeId) -> Result<String, CrudError> {
        self.read_file("entity_types", id.as_str(), "yaml")
    }

    pub fn read_property(&self, id: &PropertyId) -> Result<String, CrudError> {
        self.read_file("properties", id.as_str(), "yaml")
    }

    pub fn read_reference(&self, id: &ReferenceId) -> Result<String, CrudError> {
        self.read_file("references", id.as_str(), "yaml")
    }

    pub fn read_entity_context(&self, id: &EntityId) -> Result<String, CrudError> {
        self.read_file("entity_context", id.as_str(), "md")
    }

    fn read_file(&self, directory: &str, id: &str, extension: &str) -> Result<String, CrudError> {
        let path = self.root().join(directory).join(Path::new(id).with_extension(extension));
        fs::read_to_string(&path).map_err(|source| CrudError { path, source })
    }
}
