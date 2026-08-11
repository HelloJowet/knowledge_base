use serde::{Deserialize, Deserializer, Serialize};
use std::collections::BTreeMap;
use std::fmt;
use std::str::FromStr;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IdentifierParseError {
    value: String,
    prefix: &'static str,
}

impl fmt::Display for IdentifierParseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "invalid identifier {:?}; expected canonical {}<positive integer> syntax",
            self.value, self.prefix
        )
    }
}

impl std::error::Error for IdentifierParseError {}

macro_rules! identifier {
    ($name:ident, $prefix:literal) => {
        #[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
        #[serde(transparent)]
        pub struct $name(String);

        impl $name {
            pub fn as_str(&self) -> &str {
                &self.0
            }

            pub fn number(&self) -> u64 {
                self.0[1..].parse().expect("validated identifiers contain a u64")
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str(&self.0)
            }
        }

        impl FromStr for $name {
            type Err = IdentifierParseError;

            fn from_str(value: &str) -> Result<Self, Self::Err> {
                let digits = value.strip_prefix($prefix).ok_or_else(|| IdentifierParseError {
                    value: value.to_owned(),
                    prefix: $prefix,
                })?;
                let canonical = !digits.is_empty() && digits.bytes().all(|byte| byte.is_ascii_digit()) && !digits.starts_with('0') && digits.parse::<u64>().is_ok();
                if !canonical {
                    return Err(IdentifierParseError {
                        value: value.to_owned(),
                        prefix: $prefix,
                    });
                }
                Ok(Self(value.to_owned()))
            }
        }

        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                String::deserialize(deserializer)?.parse().map_err(serde::de::Error::custom)
            }
        }
    };
}

identifier!(EntityId, "Q");
identifier!(PropertyId, "P");
identifier!(ReferenceId, "R");
identifier!(StatementId, "S");
identifier!(EntityTypeId, "T");

pub type LocalizedMap = BTreeMap<String, LocalizedText>;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LocalizedText {
    pub text: String,
    pub references: Vec<ReferenceId>,
}

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
    pub attribution: String,
    #[serde(default, deserialize_with = "deserialize_optional_non_null")]
    pub attribution_url: Option<String>,
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

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Reference {
    pub id: ReferenceId,
    pub url: String,
    pub retrieved_at: String,
    #[serde(default, deserialize_with = "deserialize_optional_non_null")]
    pub archive_url: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct IdAllocation {
    pub version: u64,
    pub next: NextIds,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NextIds {
    pub entity: u64,
    pub property: u64,
    pub reference: u64,
    pub entity_type: u64,
}

fn deserialize_optional_non_null<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    T::deserialize(deserializer).map(Some)
}

#[cfg(test)]
mod tests {
    use super::{EntityId, EntityTypeId, PropertyId, ReferenceId, StatementId};
    use serde::de::DeserializeOwned;

    fn parses<T: DeserializeOwned>(value: &str) -> bool {
        serde_yaml::from_str::<T>(value).is_ok()
    }

    #[test]
    fn typed_identifiers_accept_their_canonical_forms() {
        assert!(parses::<EntityId>("Q1"));
        assert!(parses::<PropertyId>("P2"));
        assert!(parses::<ReferenceId>("R3"));
        assert!(parses::<StatementId>("S4"));
        assert!(parses::<EntityTypeId>("T5"));
    }

    #[test]
    fn typed_identifiers_reject_noncanonical_forms() {
        for value in ["Q0", "Q01", "Q-1", "Q", "P1", "q1", "1"] {
            assert!(!parses::<EntityId>(value), "{value} unexpectedly parsed as an entity identifier");
        }
    }

    #[test]
    fn typed_identifiers_parse_from_strings() {
        assert_eq!("Q42".parse::<EntityId>().expect("valid identifier").as_str(), "Q42");

        for value in ["Q0", "Q01", "P1", "../Q1", "Q1.yaml"] {
            assert!(value.parse::<EntityId>().is_err(), "{value} unexpectedly parsed as an entity identifier");
        }
    }
}
