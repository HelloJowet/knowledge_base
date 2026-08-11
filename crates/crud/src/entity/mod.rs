mod read;
mod statement;

pub use statement::{ApplyMode, ApplyStatementsOutcome, StatementBatch, StatementInput, StatementResult, StatementResultStatus};

use crate::KnowledgeBase;

#[derive(Clone, Copy, Debug)]
pub struct Entities<'a> {
    knowledge_base: &'a KnowledgeBase,
}

impl<'a> Entities<'a> {
    pub(crate) fn new(knowledge_base: &'a KnowledgeBase) -> Self {
        Self { knowledge_base }
    }
}
