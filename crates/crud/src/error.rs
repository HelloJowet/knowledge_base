use knowledge_base_validation::Diagnostic;
use std::fmt;
use std::io;
use std::path::PathBuf;

#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    Read { path: PathBuf, source: io::Error },
    ParseStatementBatch { path: PathBuf, source: serde_yaml::Error },
    ParseReference { path: PathBuf, source: serde_yaml::Error },
    ParseAllocation { path: PathBuf, source: serde_yaml::Error },
    ParseEntityType { path: PathBuf, source: serde_yaml::Error },
    ParseProperty { path: PathBuf, source: serde_yaml::Error },
    InvalidRequest(String),
    ParseEntity { path: PathBuf, source: serde_yaml::Error },
    InvalidRepository(String),
    InvalidSnapshot { path: PathBuf, message: String },
    Edit { path: PathBuf, message: String },
    Validation(Vec<Diagnostic>),
    Write { path: PathBuf, source: io::Error },
    ConcurrentChange(PathBuf),
    Commit { message: String },
}

impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Read { path, source } => write!(formatter, "cannot read {}: {source}", path.display()),
            Self::ParseStatementBatch { path, source } => {
                write!(formatter, "cannot parse statement manifest {}: {source}", path.display())
            }
            Self::ParseReference { path, source } => write!(formatter, "cannot parse reference {}: {source}", path.display()),
            Self::ParseAllocation { path, source } => write!(formatter, "cannot parse identifier allocation {}: {source}", path.display()),
            Self::ParseEntityType { path, source } => write!(formatter, "cannot parse entity type {}: {source}", path.display()),
            Self::ParseProperty { path, source } => write!(formatter, "cannot parse property {}: {source}", path.display()),
            Self::InvalidRequest(message) => write!(formatter, "invalid mutation request: {message}"),
            Self::ParseEntity { path, source } => {
                write!(formatter, "cannot parse entity {}: {source}", path.display())
            }
            Self::InvalidRepository(message) => write!(formatter, "cannot query knowledge base: {message}"),
            Self::InvalidSnapshot { path, message } => write!(formatter, "cannot load repository snapshot at {}: {message}", path.display()),
            Self::Edit { path, message } => {
                write!(formatter, "cannot edit resource {}: {message}", path.display())
            }
            Self::Validation(diagnostics) => {
                writeln!(formatter, "mutation would not produce a valid knowledge base:")?;
                for (index, diagnostic) in diagnostics.iter().enumerate() {
                    if index + 1 == diagnostics.len() {
                        write!(formatter, "{diagnostic}")?;
                    } else {
                        writeln!(formatter, "{diagnostic}")?;
                    }
                }
                Ok(())
            }
            Self::Write { path, source } => {
                write!(formatter, "cannot write {}: {source}", path.display())
            }
            Self::ConcurrentChange(path) => {
                write!(formatter, "resource changed while applying mutation: {}", path.display())
            }
            Self::Commit { message } => formatter.write_str(message),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Read { source, .. } | Self::Write { source, .. } => Some(source),
            Self::ParseStatementBatch { source, .. }
            | Self::ParseReference { source, .. }
            | Self::ParseAllocation { source, .. }
            | Self::ParseEntityType { source, .. }
            | Self::ParseProperty { source, .. }
            | Self::ParseEntity { source, .. } => Some(source),
            Self::InvalidRequest(_)
            | Self::InvalidRepository(_)
            | Self::InvalidSnapshot { .. }
            | Self::Edit { .. }
            | Self::Validation(_)
            | Self::ConcurrentChange(_)
            | Self::Commit { .. } => None,
        }
    }
}
