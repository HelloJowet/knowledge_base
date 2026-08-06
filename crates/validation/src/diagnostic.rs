use std::fmt;
use std::path::PathBuf;

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
