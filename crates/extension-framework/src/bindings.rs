use crate::contracts::{BindingKind, BindingReference, ExtensionMetadata};
use crate::error::FrameworkError;
use knowledge_base_models::{EntityTypeId, PropertyId};
use std::collections::BTreeMap;

/// A resolved ontology identifier assigned to a semantic binding.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BindingValue {
    EntityType(EntityTypeId),
    Property(PropertyId),
}

impl BindingValue {
    pub const fn kind(&self) -> BindingKind {
        match self {
            Self::EntityType(_) => BindingKind::EntityType,
            Self::Property(_) => BindingKind::Property,
        }
    }
}

/// Typed bindings visible to an activated extension.
#[derive(Clone, Debug, Default)]
pub struct ResolvedBindings {
    pub(crate) values: BTreeMap<BindingReference, BindingValue>,
}

impl ResolvedBindings {
    /// Looks up a binding owned by this extension or one of its direct dependencies.
    ///
    /// Limiting access to direct dependencies makes an extension's required contracts
    /// explicit instead of allowing it to rely on unrelated active extensions.
    pub fn get(&self, extension: &ExtensionMetadata, reference: &BindingReference) -> Result<&BindingValue, FrameworkError> {
        if reference.extension_id() != &extension.id && !extension.dependencies.iter().any(|dependency| dependency.id == *reference.extension_id()) {
            return Err(FrameworkError::InaccessibleBinding {
                extension: extension.id.clone(),
                binding: reference.clone(),
            });
        }
        self.values.get(reference).ok_or_else(|| FrameworkError::MissingBinding(reference.clone()))
    }

    /// Retrieves an entity-type binding and verifies its declared kind.
    pub fn entity_type(&self, extension: &ExtensionMetadata, reference: &BindingReference) -> Result<&EntityTypeId, FrameworkError> {
        match self.get(extension, reference)? {
            BindingValue::EntityType(id) => Ok(id),
            value => Err(FrameworkError::BindingKindMismatch {
                binding: reference.clone(),
                expected: BindingKind::EntityType,
                actual: value.kind(),
            }),
        }
    }

    /// Retrieves a property binding and verifies its declared kind.
    pub fn property(&self, extension: &ExtensionMetadata, reference: &BindingReference) -> Result<&PropertyId, FrameworkError> {
        match self.get(extension, reference)? {
            BindingValue::Property(id) => Ok(id),
            value => Err(FrameworkError::BindingKindMismatch {
                binding: reference.clone(),
                expected: BindingKind::Property,
                actual: value.kind(),
            }),
        }
    }
}
