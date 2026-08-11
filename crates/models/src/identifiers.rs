use serde::{Deserialize, Deserializer, Serialize};
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
