mod read;

use std::fmt;
use std::io;
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
}

#[derive(Debug)]
#[non_exhaustive]
pub struct CrudError {
    pub path: PathBuf,
    pub source: io::Error,
}

impl fmt::Display for CrudError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "cannot read {}: {}", self.path.display(), self.source)
    }
}

impl std::error::Error for CrudError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.source)
    }
}
