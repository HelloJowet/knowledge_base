mod additional_validator;
mod diagnostic;
mod input;
mod validator;

pub use additional_validator::{KnowledgeBaseValidator, ValidationContext, validate_repository_with};
pub use diagnostic::{Diagnostic, ValidationLayer};

use std::path::Path;

pub fn validate_repository(root: impl AsRef<Path>) -> Vec<Diagnostic> {
    validate_repository_with(root, std::iter::empty())
}
