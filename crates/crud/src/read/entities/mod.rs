mod query;
mod raw;
mod relationships;
mod search;

use crate::KnowledgeBaseRepository;
pub use query::{EntitiesPage, EntityFilter};
pub use relationships::{EntityRelationship, EntityRelationshipsPage, RelatedEntity, RelationshipDirection};
pub use search::EntitySearchPage;

#[derive(Clone, Copy, Debug)]
pub struct Entities<'a> {
    pub(super) repository: &'a KnowledgeBaseRepository,
}

impl<'a> Entities<'a> {
    pub(crate) fn new(repository: &'a KnowledgeBaseRepository) -> Self {
        Self { repository }
    }
}
