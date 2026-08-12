use crate::contracts::{BindingKey, BindingKind, BindingReference, ContractVersion, ExtensionId};
use std::fmt;

/// An error returned when a canonical extension identifier or binding key is invalid.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IdentifierError {
    value: String,
    expected: &'static str,
}

impl IdentifierError {
    pub(crate) fn new(value: &str, expected: &'static str) -> Self {
        Self {
            value: value.to_owned(),
            expected,
        }
    }
}

impl fmt::Display for IdentifierError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "invalid identifier {:?}; expected {}", self.value, self.expected)
    }
}

impl std::error::Error for IdentifierError {}

/// Errors returned while registering, activating, or resolving extensions.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FrameworkError {
    DuplicateExtension(ExtensionId),
    DuplicateDependency {
        extension: ExtensionId,
        dependency: ExtensionId,
    },
    DuplicateBinding {
        extension: ExtensionId,
        key: BindingKey,
    },
    MissingDependency {
        extension: ExtensionId,
        dependency: ExtensionId,
    },
    UnsupportedContract {
        extension: ExtensionId,
        required: ContractVersion,
        available: ContractVersion,
    },
    DependencyCycle(Vec<ExtensionId>),
    InactiveDependency {
        extension: ExtensionId,
        dependency: ExtensionId,
    },
    InvalidRequirement {
        extension: ExtensionId,
        binding: BindingReference,
        expected: BindingKind,
    },
    UndeclaredBinding {
        extension: ExtensionId,
        binding: BindingReference,
    },
    InaccessibleBinding {
        extension: ExtensionId,
        binding: BindingReference,
    },
    MissingBinding(BindingReference),
    BindingKindMismatch {
        binding: BindingReference,
        expected: BindingKind,
        actual: BindingKind,
    },
}

impl fmt::Display for FrameworkError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateExtension(id) => write!(formatter, "extension {id} is registered more than once"),
            Self::DuplicateDependency { extension, dependency } => write!(formatter, "extension {extension} declares dependency {dependency} more than once"),
            Self::DuplicateBinding { extension, key } => write!(formatter, "extension {extension} declares binding {key} more than once"),
            Self::MissingDependency { extension, dependency } => write!(formatter, "extension {extension} requires unavailable dependency {dependency}"),
            Self::UnsupportedContract { extension, required, available } => write!(formatter, "extension {extension} requires contract {required}, but {available} is available"),
            Self::DependencyCycle(ids) => write!(
                formatter,
                "extension dependency cycle: {}",
                ids.iter().map(ToString::to_string).collect::<Vec<_>>().join(" -> ")
            ),
            Self::InactiveDependency { extension, dependency } => write!(formatter, "active extension {extension} requires inactive dependency {dependency}"),
            Self::InvalidRequirement { extension, binding, expected } => write!(formatter, "extension {extension} requirement {binding} must reference a {expected:?} binding"),
            Self::UndeclaredBinding { extension, binding } => write!(formatter, "extension {extension} does not declare binding {binding}"),
            Self::InaccessibleBinding { extension, binding } => write!(formatter, "extension {extension} may not consume binding {binding}"),
            Self::MissingBinding(binding) => write!(formatter, "missing binding {binding}"),
            Self::BindingKindMismatch { binding, expected, actual } => write!(formatter, "binding {binding} has kind {actual:?}; expected {expected:?}"),
        }
    }
}

impl std::error::Error for FrameworkError {}
