mod contexts;
mod entities;
mod entity_types;
mod properties;
mod references;

pub use contexts::EntityContexts;
pub use entities::{Entities, EntitiesPage, EntityFilter, EntityRelationship, EntityRelationshipsPage, EntitySearchPage, RelatedEntity, RelationshipDirection};
pub use entity_types::EntityTypes;
pub use properties::Properties;
pub use references::References;

use crate::{KnowledgeBaseRepository, RepositoryError, RepositorySnapshot};

/// Read-only repository operations, including exact resource reads and typed queries.
#[derive(Clone, Copy, Debug)]
pub struct ReadRepository<'a> {
    repository: &'a KnowledgeBaseRepository,
}

impl<'a> ReadRepository<'a> {
    pub(crate) fn new(repository: &'a KnowledgeBaseRepository) -> Self {
        Self { repository }
    }
    pub fn snapshot(&self) -> Result<RepositorySnapshot, RepositoryError> {
        RepositorySnapshot::load(self.repository.root()).map_err(RepositoryError::Snapshot)
    }
    pub fn entities(&self) -> Entities<'a> {
        Entities::new(self.repository)
    }
    pub fn entity_types(&self) -> EntityTypes<'a> {
        EntityTypes::new(self.repository)
    }
    pub fn properties(&self) -> Properties<'a> {
        Properties::new(self.repository)
    }
    pub fn references(&self) -> References<'a> {
        References::new(self.repository)
    }
    pub fn entity_contexts(&self) -> EntityContexts<'a> {
        EntityContexts::new(self.repository)
    }
}
