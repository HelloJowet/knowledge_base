use std::path::{Path, PathBuf};

use knowledge_base_validation::{Diagnostic, ValidationLayer};

pub(crate) struct DiagnosticFactory;

impl DiagnosticFactory {
    pub(crate) fn domain(path: impl Into<PathBuf>, context: impl Into<String>, message: impl Into<String>) -> Diagnostic {
        let context = context.into();
        Diagnostic {
            layer: ValidationLayer::Domain,
            path: path.into(),
            line: None,
            identifier: (!context.is_empty()).then_some(context),
            message: message.into(),
        }
    }
    pub(crate) fn at_path(path: impl Into<PathBuf>, message: impl Into<String>) -> Diagnostic {
        Diagnostic {
            layer: ValidationLayer::Domain,
            path: path.into(),
            line: None,
            identifier: None,
            message: message.into(),
        }
    }
}

#[derive(Debug, Default)]
pub(crate) struct ValidationReport {
    diagnostics: Vec<Diagnostic>,
}

impl ValidationReport {
    pub(crate) fn push(&mut self, diagnostic: Diagnostic) {
        self.diagnostics.push(diagnostic);
    }
    pub(crate) fn into_diagnostics(mut self) -> Vec<Diagnostic> {
        self.diagnostics
            .sort_by(|left, right| (&left.path, &left.identifier, &left.message).cmp(&(&right.path, &right.identifier, &right.message)));
        self.diagnostics.dedup();
        self.diagnostics
    }
    pub(crate) fn require_nonempty(&mut self, path: &Path, context: &str, value: &str) {
        if value.trim().is_empty() {
            self.push(DiagnosticFactory::domain(path, context, "must not be empty"));
        }
    }
}
