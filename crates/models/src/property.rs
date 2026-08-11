use crate::{EntityTypeId, LocalizedMap, PropertyId, deserialize_optional_non_null};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

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

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum PropertyUsage {
    Statement,
    Qualifier,
    Both,
}

impl PropertyUsage {
    pub fn allows_statement(self) -> bool {
        matches!(self, Self::Statement | Self::Both)
    }

    pub fn allows_qualifier(self) -> bool {
        matches!(self, Self::Qualifier | Self::Both)
    }
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
    pub usage: PropertyUsage,
    #[serde(default, deserialize_with = "deserialize_optional_non_null")]
    pub target_types: Option<Vec<EntityTypeId>>,
    #[serde(default)]
    pub allowed_qualifiers: Vec<PropertyId>,
    #[serde(default)]
    pub cardinality: Cardinality,
    #[serde(default)]
    pub external_ids: BTreeMap<String, Vec<String>>,
}

#[cfg(test)]
mod tests {
    use super::{Property, PropertyUsage};

    fn property(usage: &str) -> String {
        format!("id: P1\nlabels:\n  en:\n    text: population\n    references: [R1]\nsubject_types: [T1]\nvalue_type: integer\nusage: {usage}\n")
    }

    #[test]
    fn property_usage_serializes_and_deserializes_all_variants() {
        for (source, usage) in [
            ("statement", PropertyUsage::Statement),
            ("qualifier", PropertyUsage::Qualifier),
            ("both", PropertyUsage::Both),
        ] {
            let parsed: Property = serde_yaml::from_str(&property(source)).expect("property parses");
            assert_eq!(parsed.usage, usage);
            assert_eq!(serde_yaml::to_string(&usage).expect("usage serializes"), format!("{source}\n"));
        }
    }

    #[test]
    fn property_usage_is_required_and_rejects_unknown_values() {
        let missing = property("statement").replace("usage: statement\n", "");
        assert!(serde_yaml::from_str::<Property>(&missing).is_err());
        assert!(serde_yaml::from_str::<Property>(&property("context")).is_err());
    }

    #[test]
    fn external_ids_default_to_empty_and_preserve_namespaces_and_values() {
        let parsed: Property = serde_yaml::from_str(&property("statement")).expect("property parses");
        assert!(parsed.external_ids.is_empty());

        let source = format!("{}external_ids:\n  wikidata: [P1082, P2046]\n  osm: []\n", property("statement"));
        let parsed: Property = serde_yaml::from_str(&source).expect("property parses");
        assert_eq!(parsed.external_ids["wikidata"], ["P1082", "P2046"]);
        assert!(parsed.external_ids["osm"].is_empty());
        let serialized = serde_yaml::to_string(&parsed).expect("property serializes");
        assert!(serialized.contains("wikidata:\n  - P1082\n  - P2046"));
        assert!(serialized.contains("osm: []"));
    }
}
