use crate::bindings::ResolvedBindings;
use crate::error::{FrameworkError, IdentifierError};
use knowledge_base_models::{Cardinality, PropertyUsage, ValueType};
use knowledge_base_validation::KnowledgeBaseValidator;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::borrow::Cow;
use std::collections::BTreeSet;
use std::fmt;
use std::str::FromStr;
use std::sync::Arc;

/// An exact integer version of an extension contract.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ContractVersion(u32);

impl ContractVersion {
    pub const fn new(value: u32) -> Self {
        Self(value)
    }
    pub const fn get(self) -> u32 {
        self.0
    }
}

impl fmt::Display for ContractVersion {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

impl Serialize for ContractVersion {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u32(self.0)
    }
}

impl<'de> Deserialize<'de> for ContractVersion {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        u32::deserialize(deserializer).map(Self)
    }
}

/// A unique lowercase kebab-case extension identifier.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ExtensionId(Cow<'static, str>);

impl ExtensionId {
    /// Constructs a canonical extension identifier from a static literal.
    ///
    /// Invalid literals cause compilation to fail when used in a constant.
    pub const fn from_static(value: &'static str) -> Self {
        assert!(canonical_segments(value, b'-'), "extension identifier must be lowercase kebab-case");
        Self(Cow::Borrowed(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ExtensionId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl FromStr for ExtensionId {
    type Err = IdentifierError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        canonical_segments(value, b'-')
            .then(|| Self(Cow::Owned(value.to_owned())))
            .ok_or_else(|| IdentifierError::new(value, "lowercase kebab-case"))
    }
}

impl Serialize for ExtensionId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for ExtensionId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        String::deserialize(deserializer)?.parse().map_err(serde::de::Error::custom)
    }
}

/// A lowercase snake_case semantic-binding key.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct BindingKey(Cow<'static, str>);

impl BindingKey {
    /// Constructs a canonical semantic-binding key from a static literal.
    ///
    /// Invalid literals cause compilation to fail when used in a constant.
    pub const fn from_static(value: &'static str) -> Self {
        assert!(canonical_segments(value, b'_'), "binding key must be lowercase snake_case");
        Self(Cow::Borrowed(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for BindingKey {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl FromStr for BindingKey {
    type Err = IdentifierError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        canonical_segments(value, b'_')
            .then(|| Self(Cow::Owned(value.to_owned())))
            .ok_or_else(|| IdentifierError::new(value, "lowercase snake_case"))
    }
}

impl Serialize for BindingKey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for BindingKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        String::deserialize(deserializer)?.parse().map_err(serde::de::Error::custom)
    }
}

/// Validates names whose segments start with a lowercase letter and then contain
/// lowercase ASCII letters or digits. The separator joins, but never empties, segments.
const fn canonical_segments(value: &str, separator: u8) -> bool {
    let bytes = value.as_bytes();
    if bytes.is_empty() {
        return false;
    }

    let mut index = 0;
    let mut starts_segment = true;
    while index < bytes.len() {
        let byte = bytes[index];
        if byte == separator {
            if starts_segment {
                return false;
            }
            starts_segment = true;
        } else if starts_segment {
            if !(byte >= b'a' && byte <= b'z') {
                return false;
            }
            starts_segment = false;
        } else if !((byte >= b'a' && byte <= b'z') || (byte >= b'0' && byte <= b'9')) {
            return false;
        }
        index += 1;
    }
    !starts_segment
}

/// The ontology identifier kind expected by a semantic binding.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BindingKind {
    EntityType,
    Property,
}

/// A fully qualified semantic binding reference, such as `wikidata:item_id_property`.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct BindingReference {
    extension_id: ExtensionId,
    key: BindingKey,
}

impl BindingReference {
    pub fn new(extension_id: ExtensionId, key: BindingKey) -> Self {
        Self { extension_id, key }
    }
    pub fn extension_id(&self) -> &ExtensionId {
        &self.extension_id
    }
    pub fn key(&self) -> &BindingKey {
        &self.key
    }

    /// Constructs a canonical binding reference from static literals.
    ///
    /// Invalid literals cause compilation to fail when used in a constant.
    pub const fn from_static(extension_id: &'static str, key: &'static str) -> Self {
        Self {
            extension_id: ExtensionId::from_static(extension_id),
            key: BindingKey::from_static(key),
        }
    }
}

impl fmt::Display for BindingReference {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}:{}", self.extension_id, self.key)
    }
}

impl FromStr for BindingReference {
    type Err = IdentifierError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let Some((extension, key)) = value.split_once(':') else {
            return Err(IdentifierError::new(value, "<extension-id>:<binding_key>"));
        };
        if key.contains(':') {
            return Err(IdentifierError::new(value, "<extension-id>:<binding_key>"));
        }
        Ok(Self::new(extension.parse()?, key.parse()?))
    }
}

/// A dependency that must be available at one exact contract version.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExtensionDependency {
    pub id: ExtensionId,
    pub contract: ContractVersion,
}

/// A semantic binding declared by an extension.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BindingDeclaration {
    pub key: BindingKey,
    pub kind: BindingKind,
}

/// A partial requirement for an entity-type binding.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EntityTypeRequirement {
    pub binding: BindingReference,
}

/// A partial requirement for a property binding. `None` scalar fields are unconstrained.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PropertyRequirement {
    pub binding: BindingReference,
    pub value_type: Option<ValueType>,
    pub usage: Option<PropertyUsage>,
    pub cardinality: Option<Cardinality>,
    pub subject_types: BTreeSet<BindingReference>,
    pub target_types: Option<BTreeSet<BindingReference>>,
    pub allowed_qualifiers: BTreeSet<BindingReference>,
}

/// Partial ontology requirements that an extension declares for its bindings.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct OntologyRequirements {
    pub entity_types: Vec<EntityTypeRequirement>,
    pub properties: Vec<PropertyRequirement>,
}

/// Static metadata that defines one extension contract.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExtensionMetadata {
    pub id: ExtensionId,
    pub contract: ContractVersion,
    pub dependencies: Vec<ExtensionDependency>,
    pub bindings: Vec<BindingDeclaration>,
    pub ontology_requirements: OntologyRequirements,
}

/// A concrete extension implementation without CLI coupling.
pub trait KnowledgeBaseExtension: Send + Sync {
    fn metadata(&self) -> &ExtensionMetadata;
    fn validators(&self, _: &ResolvedBindings) -> Result<Vec<Arc<dyn KnowledgeBaseValidator>>, FrameworkError> {
        Ok(Vec::new())
    }
}
