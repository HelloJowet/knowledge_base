mod apply;
mod edit;
mod planner;

use crate::Error;
use knowledge_base_models::{EntityId, PropertyId, Qualifier, ReferenceId, StatementId, Value};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ApplyMode {
    Preview,
    Commit,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct StatementBatch {
    pub statements: Vec<StatementInput>,
}

impl StatementBatch {
    pub fn read(path: impl AsRef<Path>) -> Result<Self, Error> {
        let path = path.as_ref();
        let source = fs::read_to_string(path).map_err(|source| Error::Read { path: path.to_path_buf(), source })?;
        serde_yaml::from_str(&source).map_err(|source| Error::ParseStatementBatch { path: path.to_path_buf(), source })
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct StatementInput {
    pub entity: EntityId,
    pub property: PropertyId,
    pub value: Value,
    #[serde(default)]
    pub qualifiers: Vec<Qualifier>,
    pub references: Vec<ReferenceId>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum StatementResultStatus {
    WouldAdd,
    Added,
    AlreadyPresent,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct StatementResult {
    pub index: usize,
    pub entity: EntityId,
    pub property: PropertyId,
    pub statement: StatementId,
    pub status: StatementResultStatus,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "outcome", content = "results", rename_all = "snake_case")]
pub enum ApplyStatementsOutcome {
    Previewed(Vec<StatementResult>),
    Applied(Vec<StatementResult>),
    NotApplied(Vec<StatementResult>),
}

impl ApplyStatementsOutcome {
    pub fn results(&self) -> &[StatementResult] {
        match self {
            Self::Previewed(results) | Self::Applied(results) | Self::NotApplied(results) => results,
        }
    }

    pub fn was_applied(&self) -> bool {
        matches!(self, Self::Applied(_))
    }

    pub fn was_rejected(&self) -> bool {
        matches!(self, Self::NotApplied(_))
    }
}

fn validate_batch(batch: &StatementBatch) -> Result<(), Error> {
    if batch.statements.is_empty() {
        return Err(Error::InvalidRequest("statements must not be empty".to_owned()));
    }
    for (offset, statement) in batch.statements.iter().enumerate() {
        if statement.references.is_empty() {
            return Err(Error::InvalidRequest(format!("statements[{}].references must not be empty", offset + 1)));
        }
        for (qualifier_offset, qualifier) in statement.qualifiers.iter().enumerate() {
            if statement.qualifiers[..qualifier_offset].contains(qualifier) {
                return Err(Error::InvalidRequest(format!(
                    "statements[{}].qualifiers contains duplicate property/value entry",
                    offset + 1
                )));
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{StatementBatch, validate_batch};

    fn parses(value: &str) -> bool {
        serde_yaml::from_str::<StatementBatch>(value).is_ok()
    }

    #[test]
    fn statement_batches_are_strict() {
        let valid = r#"
statements:
  - entity: Q1
    property: P2
    value: { type: string, value: Q99 }
    references: [R3]
"#;
        assert!(parses(valid));

        for invalid in [
            valid.replace("statements:\n", "statements:\nunknown: true\n"),
            valid.replace("    property: P2\n", "    property: P2\n    property: P3\n"),
            valid.replace("entity: Q1", "entity: P1"),
            valid.replace("type: string", "type: unsupported"),
        ] {
            assert!(!parses(&invalid), "invalid batch unexpectedly parsed:\n{invalid}");
        }

        assert!(parses(&valid.replace("    references: [R3]\n", "    qualifiers: []\n    references: [R3]\n")));
    }

    #[test]
    fn statement_batches_reject_duplicate_qualifiers() {
        let source = r#"
statements:
  - entity: Q1
    property: P1
    value: { type: integer, value: 7 }
    qualifiers:
      - property: P2
        value: { type: date, value: "2024-01-01" }
      - property: P2
        value: { type: date, value: "2024-01-01" }
    references: [R1]
"#;
        let batch: StatementBatch = serde_yaml::from_str(source).expect("manifest parsing is separate from request validation");
        assert!(validate_batch(&batch).is_err());
    }
}
