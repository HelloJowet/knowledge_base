use crate::{EntityId, EntityTypeId, LocalizedMap, PropertyId, ReferenceId, StatementId, ValueType};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Classification {
    pub value: EntityTypeId,
    pub references: Vec<ReferenceId>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Statement {
    pub id: StatementId,
    pub property: PropertyId,
    pub value: Value,
    #[serde(default)]
    pub qualifiers: Vec<Qualifier>,
    pub references: Vec<ReferenceId>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Qualifier {
    pub property: PropertyId,
    pub value: Value,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "type", rename_all = "lowercase", deny_unknown_fields)]
pub enum Value {
    Entity { value: EntityId },
    String { value: String },
    Integer { value: i64 },
    Decimal { value: String },
    Boolean { value: bool },
    Date { value: String },
    Datetime { value: String },
    Url { value: String },
    Coordinate { latitude: String, longitude: String },
}

impl Value {
    pub fn value_type(&self) -> ValueType {
        match self {
            Self::Entity { .. } => ValueType::Entity,
            Self::String { .. } => ValueType::String,
            Self::Integer { .. } => ValueType::Integer,
            Self::Decimal { .. } => ValueType::Decimal,
            Self::Boolean { .. } => ValueType::Boolean,
            Self::Date { .. } => ValueType::Date,
            Self::Datetime { .. } => ValueType::Datetime,
            Self::Url { .. } => ValueType::Url,
            Self::Coordinate { .. } => ValueType::Coordinate,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Image {
    pub url: String,
    pub alt: String,
    pub source_url: String,
    pub creator: String,
    pub license: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub references: Vec<ReferenceId>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Entity {
    pub id: EntityId,
    pub labels: LocalizedMap,
    #[serde(default)]
    pub descriptions: LocalizedMap,
    pub entity_types: Vec<Classification>,
    #[serde(default)]
    pub images: Vec<Image>,
    pub statements: Vec<Statement>,
}

#[cfg(test)]
mod tests {
    use super::Image;

    #[test]
    fn images_use_lossless_metadata_and_optional_references() {
        let image: Image =
            serde_yaml::from_str("url: https://example.org/image.jpg\nalt: Example image\nsource_url: https://example.org/source\ncreator: Example creator\nlicense: CC BY 4.0\n")
                .expect("canonical image parses");

        assert!(image.references.is_empty());
        assert!(!serde_yaml::to_string(&image).expect("image serializes").contains("references:"));

        let cited: Image = serde_yaml::from_str(
            "url: https://example.org/image.jpg\nalt: Example image\nsource_url: https://example.org/source\ncreator: Example creator\nlicense: CC BY 4.0\nreferences: [R1]\n",
        )
        .expect("cited canonical image parses");
        assert_eq!(cited.references[0].as_str(), "R1");
    }

    #[test]
    fn images_reject_legacy_attribution_fields() {
        assert!(
            serde_yaml::from_str::<Image>("url: https://example.org/image.jpg\nattribution: Example Archive\nattribution_url: https://example.org/source\nreferences: [R1]\n")
                .is_err()
        );
    }
}
