use crate::bindings::ResolvedBindings;
use crate::error::{FrameworkError, IdentifierError};
use knowledge_base_models::{Cardinality, PropertyUsage, ValueType};
use knowledge_base_validation::KnowledgeBaseValidator;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
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
pub struct ExtensionId(String);

impl ExtensionId {
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
        canonical_segments(value, '-')
            .then(|| Self(value.to_owned()))
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
pub struct BindingKey(String);

impl BindingKey {
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
        canonical_segments(value, '_')
            .then(|| Self(value.to_owned()))
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
fn canonical_segments(value: &str, separator: char) -> bool {
    !value.is_empty()
        && value.bytes().all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == separator as u8)
        && value.split(separator).all(|segment| segment.as_bytes().first().is_some_and(u8::is_ascii_lowercase))
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
    fn ontology_requirements(&self) -> &OntologyRequirements {
        &self.metadata().ontology_requirements
    }
    fn validators(&self, bindings: &ResolvedBindings) -> Result<Vec<Arc<dyn KnowledgeBaseValidator>>, FrameworkError>;
}
