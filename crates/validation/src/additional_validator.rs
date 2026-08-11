use crate::{Diagnostic, diagnostic, validator};
use std::path::Path;

/// A domain-specific, read-only validator for a complete knowledge-base repository.
///
/// Diagnostics must use paths relative to `repository`, so they remain meaningful when
/// mutations validate a temporary staged copy.
pub trait AdditionalValidator: Send + Sync {
    fn validate(&self, repository: &Path) -> Vec<Diagnostic>;
}

impl<F> AdditionalValidator for F
where
    F: Fn(&Path) -> Vec<Diagnostic> + Send + Sync,
{
    fn validate(&self, repository: &Path) -> Vec<Diagnostic> {
        self(repository)
    }
}

/// Validates a repository with the built-in rules followed by every supplied domain validator.
///
/// All validators run even when another validator reports diagnostics. The returned diagnostics
/// are sorted deterministically by path, line, identifier, message, and validation layer.
pub fn validate_repository_with<'a>(root: impl AsRef<Path>, validators: impl IntoIterator<Item = &'a dyn AdditionalValidator>) -> Vec<Diagnostic> {
    let root = root.as_ref();
    let mut diagnostics = validator::validate_repository(root);
    for validator in validators {
        diagnostics.extend(validator.validate(root));
    }
    diagnostic::sort_diagnostics(&mut diagnostics);
    diagnostics
}
