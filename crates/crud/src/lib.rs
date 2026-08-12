mod error;
mod filesystem;
mod read;
mod repository;
pub mod write;

pub use error::RepositoryError;
pub use knowledge_base_snapshot::RepositorySnapshot;
pub use read::{
    Entities, EntitiesPage, EntityContexts, EntityFilter, EntityRelationship, EntityRelationshipsPage, EntitySearchPage, EntityTypes, Properties, References, RelatedEntity,
    RelationshipDirection,
};
pub use repository::KnowledgeBaseRepository;

// Internal modules use the short name; only `RepositoryError` is public.
pub(crate) use error::RepositoryError as Error;
