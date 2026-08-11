use std::fmt;
use std::path::Path;
use std::path::PathBuf;

use crate::input::Loaded;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ValidationLayer {
    Schema,
    Ontology,
    Domain,
    Provenance,
}

impl fmt::Display for ValidationLayer {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Self::Schema => "schema",
            Self::Ontology => "ontology",
            Self::Domain => "domain",
            Self::Provenance => "provenance",
        };
        formatter.write_str(name)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Diagnostic {
    pub layer: ValidationLayer,
    pub path: PathBuf,
    pub line: Option<usize>,
    pub identifier: Option<String>,
    pub message: String,
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.path.display())?;
        if let Some(line) = self.line {
            write!(formatter, ":{line}")?;
        }
        write!(formatter, " [{}]", self.layer)?;
        if let Some(identifier) = &self.identifier {
            write!(formatter, " [{identifier}]")?;
        }
        write!(formatter, " {}", self.message)
    }
}

#[derive(Default)]
pub(crate) struct Diagnostics(Vec<Diagnostic>);

impl Diagnostics {
    pub(crate) fn push(&mut self, layer: ValidationLayer, path: PathBuf, line: Option<usize>, identifier: Option<String>, message: impl Into<String>) {
        self.0.push(Diagnostic {
            layer,
            path,
            line,
            identifier,
            message: message.into(),
        });
    }

    pub(crate) fn schema<T>(&mut self, item: &Loaded<T>, identifier: &str, message: impl Into<String>) {
        self.push(ValidationLayer::Schema, item.path.clone(), None, Some(identifier.to_owned()), message);
    }

    pub(crate) fn ontology<T>(&mut self, item: &Loaded<T>, identifier: &str, message: impl Into<String>) {
        self.push(ValidationLayer::Ontology, item.path.clone(), None, Some(identifier.to_owned()), message);
    }

    pub(crate) fn provenance(&mut self, path: &Path, line: Option<usize>, identifier: &str, message: impl Into<String>) {
        self.push(ValidationLayer::Provenance, path.to_path_buf(), line, Some(identifier.to_owned()), message);
    }

    pub(crate) fn finish(mut self) -> Vec<Diagnostic> {
        sort_diagnostics(&mut self.0);
        self.0
    }
}

pub(crate) fn sort_diagnostics(diagnostics: &mut [Diagnostic]) {
    diagnostics.sort_by(|left, right| {
        (&left.path, left.line.unwrap_or(usize::MAX), &left.identifier, &left.message, left.layer).cmp(&(
            &right.path,
            right.line.unwrap_or(usize::MAX),
            &right.identifier,
            &right.message,
            right.layer,
        ))
    });
}
