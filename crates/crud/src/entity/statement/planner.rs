use super::edit::append_statements;
use super::{StatementBatch, StatementResult, StatementResultStatus};
use crate::mutation::FileEdit;
use crate::{Error, resource};
use knowledge_base_models::{Entity, EntityId, PropertyId, Statement, StatementId, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;

#[derive(Debug)]
pub(super) struct StatementPlan {
    pub(super) results: Vec<StatementResult>,
    pub(super) edits: Vec<FileEdit>,
}

impl StatementPlan {
    pub(super) fn all_new(&self) -> bool {
        self.results.iter().all(|result| result.status == StatementResultStatus::WouldAdd)
    }
}

pub(super) struct StatementPlanner<'a> {
    root: &'a Path,
    batch: &'a StatementBatch,
}

struct EntityState {
    path: PathBuf,
    original: Vec<u8>,
    next_id: Option<u64>,
    known: Vec<(PropertyId, Value, StatementId)>,
    additions: Vec<Statement>,
}

impl<'a> StatementPlanner<'a> {
    pub(super) fn new(root: &'a Path, batch: &'a StatementBatch) -> Self {
        Self { root, batch }
    }

    pub(super) fn plan(self) -> Result<StatementPlan, Error> {
        let mut entities = self.load_entities()?;
        let mut results = Vec::with_capacity(self.batch.statements.len());

        for (offset, input) in self.batch.statements.iter().enumerate() {
            let state = entities.get_mut(&input.entity).expect("all statement entities were loaded");
            let existing = state
                .known
                .iter()
                .find(|(property, value, _)| property == &input.property && value == &input.value)
                .map(|(_, _, statement)| statement.clone());
            let (statement, status) = if let Some(statement) = existing {
                (statement, StatementResultStatus::AlreadyPresent)
            } else {
                let statement = allocate_statement_id(&input.entity, &mut state.next_id)?;
                state.known.push((input.property.clone(), input.value.clone(), statement.clone()));
                state.additions.push(Statement {
                    id: statement.clone(),
                    property: input.property.clone(),
                    value: input.value.clone(),
                    qualifiers: Vec::new(),
                    references: input.references.clone(),
                });
                (statement, StatementResultStatus::WouldAdd)
            };
            results.push(StatementResult {
                index: offset + 1,
                entity: input.entity.clone(),
                property: input.property.clone(),
                statement,
                status,
            });
        }

        let mut edits = entities
            .into_values()
            .filter(|state| !state.additions.is_empty())
            .map(|state| {
                let source = std::str::from_utf8(&state.original).map_err(|error| Error::Edit {
                    path: state.path.clone(),
                    message: format!("file is not UTF-8: {error}"),
                })?;
                let replacement = append_statements(source, &state.additions, &state.path)?;
                Ok(FileEdit {
                    path: state.path,
                    original: state.original,
                    replacement: replacement.into_bytes(),
                })
            })
            .collect::<Result<Vec<_>, Error>>()?;
        edits.sort_by(|left, right| left.path.cmp(&right.path));

        Ok(StatementPlan { results, edits })
    }

    fn load_entities(&self) -> Result<BTreeMap<EntityId, EntityState>, Error> {
        self.batch
            .statements
            .iter()
            .map(|input| input.entity.clone())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .map(|id| self.load_entity(id))
            .collect()
    }

    fn load_entity(&self, id: EntityId) -> Result<(EntityId, EntityState), Error> {
        let path = resource::path(self.root, "entities", id.as_str(), "yaml");
        let original = fs::read(&path).map_err(|source| Error::Read { path: path.clone(), source })?;
        let entity = serde_yaml::from_slice::<Entity>(&original).map_err(|source| Error::ParseEntity { path: path.clone(), source })?;
        let maximum = entity.statements.iter().map(|statement| statement.id.number()).max().unwrap_or(0);
        let known = entity.statements.into_iter().map(|statement| (statement.property, statement.value, statement.id)).collect();
        Ok((
            id,
            EntityState {
                path,
                original,
                next_id: maximum.checked_add(1),
                known,
                additions: Vec::new(),
            },
        ))
    }
}

fn allocate_statement_id(entity: &EntityId, next_id: &mut Option<u64>) -> Result<StatementId, Error> {
    let next = next_id
        .take()
        .ok_or_else(|| Error::InvalidRequest(format!("cannot allocate another statement identifier for {entity}")))?;
    let statement = StatementId::from_str(&format!("S{next}")).expect("positive u64 values form valid statement identifiers");
    *next_id = next.checked_add(1);
    Ok(statement)
}

#[cfg(test)]
mod tests {
    use super::StatementPlanner;
    use crate::{StatementBatch, StatementResultStatus};
    use std::fs;

    fn repository(entity_source: &str) -> tempfile::TempDir {
        let root = tempfile::tempdir().unwrap();
        fs::create_dir(root.path().join("entities")).unwrap();
        fs::write(root.path().join("entities/Q1.yaml"), entity_source).unwrap();
        root
    }

    fn batch(source: &str) -> StatementBatch {
        serde_yaml::from_str(source).unwrap()
    }

    #[test]
    fn duplicate_rows_share_the_planned_identifier() {
        let root = repository("id: Q1\nlabels: {}\nentity_types: []\nstatements: []\n");
        let batch = batch(
            "statements:\n  - entity: Q1\n    property: P1\n    value: { type: integer, value: 7 }\n    references: [R1]\n  - entity: Q1\n    property: P1\n    value: { type: integer, value: 7 }\n    references: [R2]\n",
        );

        let plan = StatementPlanner::new(root.path(), &batch).plan().unwrap();

        assert_eq!(plan.results[0].statement, plan.results[1].statement);
        assert_eq!(plan.results[0].status, StatementResultStatus::WouldAdd);
        assert_eq!(plan.results[1].status, StatementResultStatus::AlreadyPresent);
        assert_eq!(plan.edits.len(), 1);
    }

    #[test]
    fn existing_statements_are_detected_without_an_edit() {
        let root = repository("id: Q1\nlabels: {}\nentity_types: []\nstatements:\n  - id: S3\n    property: P1\n    value: { type: integer, value: 7 }\n    references: [R1]\n");
        let batch = batch("statements:\n  - entity: Q1\n    property: P1\n    value: { type: integer, value: 7 }\n    references: [R2]\n");

        let plan = StatementPlanner::new(root.path(), &batch).plan().unwrap();

        assert_eq!(plan.results[0].status, StatementResultStatus::AlreadyPresent);
        assert_eq!(plan.results[0].statement.as_str(), "S3");
        assert!(plan.edits.is_empty());
    }

    #[test]
    fn identifiers_are_allocated_independently_for_each_entity() {
        let root = repository("id: Q1\nlabels: {}\nentity_types: []\nstatements:\n  - id: S2\n    property: P1\n    value: { type: integer, value: 1 }\n    references: [R1]\n");
        fs::write(root.path().join("entities/Q2.yaml"), "id: Q2\nlabels: {}\nentity_types: []\nstatements: []\n").unwrap();
        let batch = batch(
            "statements:\n  - entity: Q1\n    property: P1\n    value: { type: integer, value: 2 }\n    references: [R1]\n  - entity: Q2\n    property: P1\n    value: { type: integer, value: 3 }\n    references: [R1]\n",
        );

        let plan = StatementPlanner::new(root.path(), &batch).plan().unwrap();

        assert_eq!(plan.results[0].statement.as_str(), "S3");
        assert_eq!(plan.results[1].statement.as_str(), "S1");
        assert_eq!(plan.edits.len(), 2);
        assert!(plan.edits[0].path.ends_with("Q1.yaml"));
        assert!(plan.edits[1].path.ends_with("Q2.yaml"));
    }

    #[test]
    fn identifier_exhaustion_is_reported() {
        let root = repository(
            "id: Q1\nlabels: {}\nentity_types: []\nstatements:\n  - id: S18446744073709551615\n    property: P1\n    value: { type: integer, value: 1 }\n    references: [R1]\n",
        );
        let batch = batch("statements:\n  - entity: Q1\n    property: P1\n    value: { type: integer, value: 2 }\n    references: [R1]\n");

        let error = StatementPlanner::new(root.path(), &batch).plan().unwrap_err();

        assert!(error.to_string().contains("cannot allocate another statement identifier"));
    }
}
