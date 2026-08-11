mod entity;
mod entity_context;
mod entity_type;
mod error;
mod mutation;
mod property;
mod reference;
mod resource;

pub use entity::{ApplyMode, ApplyStatementsOutcome, Entities, StatementBatch, StatementInput, StatementResult, StatementResultStatus};
pub use entity_context::EntityContexts;
pub use entity_type::EntityTypes;
pub use error::Error;
pub use property::Properties;
pub use reference::References;

use std::path::{Path, PathBuf};

#[derive(Clone, Debug)]
pub struct KnowledgeBase {
    root: PathBuf,
}

impl KnowledgeBase {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    pub fn root(&self) -> &Path {
        &self.root
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
