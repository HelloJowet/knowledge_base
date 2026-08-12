use knowledge_base_models::{PropertyUsage, ValueType};
use serde::{Deserialize, Serialize};

/// A proposed entity type that may be used by candidates in this inventory.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DraftEntityType {
    pub id: String,
    pub label: String,
    pub description: String,
    pub evidence: Vec<String>,
    pub affected_candidates: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub production_id: Option<String>,
}

/// A proposed property, including the same compatibility information as a production property.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DraftProperty {
    pub id: String,
    pub label: String,
    pub description: String,
    pub value_type: ValueType,
    pub allowed_subject_types: Vec<String>,
    pub allowed_value_types: Vec<String>,
    pub allowed_qualifiers: Vec<String>,
    pub usage: PropertyUsage,
    pub evidence: Vec<String>,
    pub affected_candidates: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub production_id: Option<String>,
}
