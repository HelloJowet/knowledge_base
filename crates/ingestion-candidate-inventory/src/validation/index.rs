use std::collections::{BTreeMap, HashSet};

use knowledge_base_models::{EntityId, EntityTypeId, PropertyId, PropertyUsage, ReferenceId, ValueType};
use knowledge_base_snapshot::RepositorySnapshot;

use crate::{Candidate, DraftProperty, IngestionCandidateInventory};

/// Lookup tables built once so validation rules can describe relationships without rescanning YAML.
pub(crate) struct InventoryIndex<'a> {
    pub(crate) candidates: BTreeMap<&'a str, &'a Candidate>,
    pub(crate) evidence_ids: HashSet<&'a str>,
    pub(crate) draft_type_ids: HashSet<&'a str>,
    pub(crate) draft_properties: BTreeMap<&'a str, &'a DraftProperty>,
}

impl<'a> InventoryIndex<'a> {
    pub(crate) fn new(inventory: &'a IngestionCandidateInventory) -> Self {
        Self {
            candidates: inventory.candidates.iter().map(|candidate| (candidate.key.as_str(), candidate)).collect(),
            evidence_ids: inventory.evidence.iter().map(|evidence| evidence.id.as_str()).collect(),
            draft_type_ids: inventory.draft_entity_types.iter().map(|item| item.id.as_str()).collect(),
            draft_properties: inventory.draft_properties.iter().map(|item| (item.id.as_str(), item)).collect(),
        }
    }
}

pub(crate) struct PropertyDefinition<'a> {
    pub(crate) value_type: ValueType,
    pub(crate) allowed_subject_types: Vec<&'a str>,
    pub(crate) allowed_value_types: Vec<&'a str>,
    pub(crate) allowed_qualifiers: Vec<&'a str>,
    pub(crate) usage: PropertyUsage,
}

pub(crate) fn property<'a>(id: &str, snapshot: &'a RepositorySnapshot, drafts: &'a BTreeMap<&str, &DraftProperty>) -> Option<PropertyDefinition<'a>> {
    if let Some(property) = id.parse::<PropertyId>().ok().and_then(|id| snapshot.properties().get(&id)) {
        return Some(PropertyDefinition {
            value_type: property.value_type,
            allowed_subject_types: property.subject_types.iter().map(EntityTypeId::as_str).collect(),
            allowed_value_types: property.target_types.as_deref().unwrap_or_default().iter().map(EntityTypeId::as_str).collect(),
            allowed_qualifiers: property.allowed_qualifiers.iter().map(PropertyId::as_str).collect(),
            usage: property.usage,
        });
    }
    drafts.get(id).map(|property| PropertyDefinition {
        value_type: property.value_type,
        allowed_subject_types: property.allowed_subject_types.iter().map(String::as_str).collect(),
        allowed_value_types: property.allowed_value_types.iter().map(String::as_str).collect(),
        allowed_qualifiers: property.allowed_qualifiers.iter().map(String::as_str).collect(),
        usage: property.usage,
    })
}

pub(crate) fn has_entity(snapshot: &RepositorySnapshot, value: &str) -> bool {
    value.parse::<EntityId>().is_ok_and(|id| snapshot.entities().contains_key(&id))
}
pub(crate) fn has_entity_type(snapshot: &RepositorySnapshot, value: &str) -> bool {
    value.parse::<EntityTypeId>().is_ok_and(|id| snapshot.entity_types().contains_key(&id))
}
pub(crate) fn has_property(snapshot: &RepositorySnapshot, value: &str) -> bool {
    value.parse::<PropertyId>().is_ok_and(|id| snapshot.properties().contains_key(&id))
}
pub(crate) fn has_reference(snapshot: &RepositorySnapshot, value: &str) -> bool {
    value.parse::<ReferenceId>().is_ok_and(|id| snapshot.references().contains_key(&id))
}
