use crate::{read::ReadRepository, write::WriteRepository};
use knowledge_base_validation::{Diagnostic, KnowledgeBaseValidator, validate_repository_with};
use std::fmt;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Filesystem-backed access point for one canonical knowledge-base repository.
#[derive(Clone)]
pub struct KnowledgeBaseRepository {
    root: PathBuf,
    validators: Vec<Arc<dyn KnowledgeBaseValidator>>,
}

impl fmt::Debug for KnowledgeBaseRepository {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("KnowledgeBaseRepository")
            .field("root", &self.root)
            .field("validator_count", &self.validators.len())
            .finish()
    }
}

impl KnowledgeBaseRepository {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self::with_validators(root, [])
    }

    pub fn with_validators(root: impl Into<PathBuf>, validators: impl IntoIterator<Item = Arc<dyn KnowledgeBaseValidator>>) -> Self {
        Self {
            root: root.into(),
            validators: validators.into_iter().collect(),
        }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Validates the current repository with generic and configured validators.
    pub fn validate(&self) -> Vec<Diagnostic> {
        validate_repository_with(&self.root, self.validators.iter().map(AsRef::as_ref))
    }

    pub fn read(&self) -> ReadRepository<'_> {
        ReadRepository::new(self)
    }

    pub fn write(&self) -> WriteRepository<'_> {
        WriteRepository::new(self)
    }

    pub(crate) fn validators(&self) -> &[Arc<dyn KnowledgeBaseValidator>] {
        &self.validators
    }
}
