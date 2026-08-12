mod execution;
mod references;
pub mod statements;

pub use references::{ReferenceDraft, ReferenceRegistrationOutcome, ReferenceRegistrationStatus, References};
pub use statements::{ApplyStatementsOutcome, StatementBatch, StatementInput, StatementResult, StatementResultStatus};

use crate::KnowledgeBaseRepository;

/// Controls whether a mutation is validated only or written to disk.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WriteMode {
    Preview,
    Commit,
}

/// State-changing repository operations.
#[derive(Clone, Copy, Debug)]
pub struct WriteRepository<'a> {
    repository: &'a KnowledgeBaseRepository,
}

impl<'a> WriteRepository<'a> {
    pub(crate) fn new(repository: &'a KnowledgeBaseRepository) -> Self {
        Self { repository }
    }
    pub fn statements(&self) -> StatementMutations<'a> {
        StatementMutations { repository: self.repository }
    }
    pub fn references(&self) -> References<'a> {
        References::new(self.repository)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct StatementMutations<'a> {
    pub(crate) repository: &'a KnowledgeBaseRepository,
}
