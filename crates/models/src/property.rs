use crate::{EntityTypeId, LocalizedMap, PropertyId, deserialize_optional_non_null};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct EntityType {
    pub id: EntityTypeId,
    pub labels: LocalizedMap,
    #[serde(default)]
    pub descriptions: LocalizedMap,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ValueType {
    Entity,
    String,
    Integer,
    Decimal,
    Boolean,
    Date,
    Datetime,
    Url,
    Coordinate,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Cardinality {
    One,
    #[default]
    Many,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Property {
    pub id: PropertyId,
    pub labels: LocalizedMap,
    #[serde(default)]
    pub descriptions: LocalizedMap,
    pub subject_types: Vec<EntityTypeId>,
    pub value_type: ValueType,
    #[serde(default, deserialize_with = "deserialize_optional_non_null")]
    pub target_types: Option<Vec<EntityTypeId>>,
    #[serde(default)]
    pub allowed_qualifiers: Vec<PropertyId>,
    #[serde(default)]
    pub cardinality: Cardinality,
}
