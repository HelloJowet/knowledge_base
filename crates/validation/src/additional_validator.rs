use crate::{Diagnostic, ValidationLayer, diagnostic, validator};
use knowledge_base_snapshot::RepositorySnapshot;
use std::path::{Path, PathBuf};

/// The single immutable repository view shared by composed domain validators.
///
/// The snapshot contains canonical structured resources. Entity-context Markdown
/// remains part of generic validation and is deliberately not included here.
#[derive(Clone, Copy, Debug)]
pub struct ValidationContext<'a> {
    repository_root: &'a Path,
    snapshot: &'a RepositorySnapshot,
}

impl<'a> ValidationContext<'a> {
    fn new(repository_root: &'a Path, snapshot: &'a RepositorySnapshot) -> Self {
        Self { repository_root, snapshot }
    }

    /// The root used for this validation pass.
    pub fn repository_root(&self) -> &'a Path {
        self.repository_root
    }

    /// The shared, read-only structured repository snapshot.
    pub fn snapshot(&self) -> &'a RepositorySnapshot {
        self.snapshot
    }
}

/// A domain-specific, read-only validator for a complete knowledge-base repository.
///
/// Diagnostics must use paths relative to `repository`, so they remain meaningful when
/// mutations validate a temporary staged copy.
pub trait AdditionalValidator: Send + Sync {
    fn validate(&self, context: &ValidationContext<'_>) -> Vec<Diagnostic>;
}

impl<F> AdditionalValidator for F
where
    F: for<'a> Fn(&ValidationContext<'a>) -> Vec<Diagnostic> + Send + Sync,
{
    fn validate(&self, context: &ValidationContext<'_>) -> Vec<Diagnostic> {
        self(context)
    }
}

/// Validates a repository with the built-in rules followed by every supplied domain validator.
///
/// All validators run even when another validator reports diagnostics. The returned diagnostics
/// are sorted deterministically by path, line, identifier, message, and validation layer.
pub fn validate_repository_with<'a>(root: impl AsRef<Path>, validators: impl IntoIterator<Item = &'a dyn AdditionalValidator>) -> Vec<Diagnostic> {
    let root = root.as_ref();
    let mut diagnostics = validator::validate_repository(root);
    let validators = validators.into_iter().collect::<Vec<_>>();
    if !validators.is_empty() {
        match RepositorySnapshot::load(root) {
            Ok(snapshot) => {
                let context = ValidationContext::new(root, &snapshot);
                for validator in validators {
                    diagnostics.extend(validator.validate(&context));
                }
            }
            Err(error) if diagnostics.is_empty() => {
                let path = error.path().strip_prefix(root).map(PathBuf::from).unwrap_or_else(|_| error.path().to_path_buf());
                diagnostics.push(Diagnostic {
                    layer: ValidationLayer::Schema,
                    path,
                    line: None,
                    identifier: None,
                    message: format!("cannot load shared repository snapshot: {error}"),
                });
            }
            Err(_) => {}
        }
    }
    diagnostic::sort_diagnostics(&mut diagnostics);
    diagnostics
}
