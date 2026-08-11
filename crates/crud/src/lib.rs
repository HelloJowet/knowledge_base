mod entity;
mod entity_context;
mod entity_type;
mod error;
mod mutation;
mod property;
mod reference;
mod resource;
mod snapshot;

pub use entity::{
    ApplyMode, ApplyStatementsOutcome, Entities, EntitiesPage, EntityFilter, EntityRelationship, EntityRelationshipsPage, RelatedEntity, RelationshipDirection, StatementBatch,
    StatementInput, StatementResult, StatementResultStatus,
};
pub use entity_context::EntityContexts;
pub use entity_type::EntityTypes;
pub use error::Error;
pub use property::Properties;
pub use reference::{ReferenceDraft, ReferenceRegistrationOutcome, ReferenceRegistrationStatus, References};
pub use snapshot::RepositorySnapshot;

use knowledge_base_validation::{AdditionalValidator, Diagnostic, validate_repository_with};
use std::fmt;
use std::path::{Path, PathBuf};
use std::sync::Arc;

#[derive(Clone)]
pub struct KnowledgeBase {
    root: PathBuf,
    additional_validators: Vec<Arc<dyn AdditionalValidator>>,
}

impl fmt::Debug for KnowledgeBase {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("KnowledgeBase")
            .field("root", &self.root)
            .field("additional_validator_count", &self.additional_validators.len())
            .finish()
    }
}

impl KnowledgeBase {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self::with_additional_validators(root, [])
    }

    pub fn with_additional_validators(root: impl Into<PathBuf>, validators: impl IntoIterator<Item = Arc<dyn AdditionalValidator>>) -> Self {
        Self {
            root: root.into(),
            additional_validators: validators.into_iter().collect(),
        }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Validates this repository with the built-in and configured domain validators.
    pub fn validate(&self) -> Vec<Diagnostic> {
        validate_repository_with(&self.root, self.additional_validators.iter().map(AsRef::as_ref))
    }

    /// Loads a complete, read-only view of the canonical repository.
    ///
    /// This checks that managed resources can be read and parsed, but does not
    /// run generic or configured semantic validators. Call [`Self::validate`]
    /// when semantic validation is required.
    pub fn snapshot(&self) -> Result<RepositorySnapshot, Error> {
        RepositorySnapshot::load(&self.root)
    }

    pub(crate) fn additional_validators(&self) -> &[Arc<dyn AdditionalValidator>] {
        &self.additional_validators
    }

    pub fn entities(&self) -> Entities<'_> {
        Entities::new(self)
    }

    pub fn entity_types(&self) -> EntityTypes<'_> {
        EntityTypes::new(self)
    }

    pub fn properties(&self) -> Properties<'_> {
        Properties::new(self)
    }

    pub fn references(&self) -> References<'_> {
        References::new(self)
    }

    pub fn entity_contexts(&self) -> EntityContexts<'_> {
        EntityContexts::new(self)
    }
}
