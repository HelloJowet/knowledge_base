mod diagnostic;
mod input;
mod validator;

pub use diagnostic::{Diagnostic, ValidationLayer};

use std::path::Path;

pub fn validate_repository(root: impl AsRef<Path>) -> Vec<Diagnostic> {
    validator::validate_repository(root.as_ref())
}
