use super::Entities;
use crate::Error;
use knowledge_base_models::{Entity, PropertyId, Value};
use serde::Serialize;
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct EntityFilter {
    pub property: PropertyId,
    pub value: Value,
}

#[derive(Clone, Debug, Serialize)]
pub struct EntitiesPage {
    pub filters: Vec<EntityFilter>,
    pub offset: usize,
    pub limit: usize,
    pub total: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_offset: Option<usize>,
    pub entities: Vec<Entity>,
}

impl Entities<'_> {
    pub fn query(&self, filters: &[EntityFilter], limit: usize, offset: usize) -> Result<EntitiesPage, Error> {
        if filters.is_empty() {
            return Err(Error::InvalidRequest("at least one entity filter is required".to_owned()));
        }
        if limit == 0 {
            return Err(Error::InvalidRequest("entity query limit must be greater than zero".to_owned()));
        }

        let mut entities = load_entities(self.knowledge_base.root())?;
        entities.retain(|entity| {
            filters.iter().all(|filter| {
                entity
                    .statements
                    .iter()
                    .any(|statement| statement.property == filter.property && statement.value == filter.value)
            })
        });
        entities.sort_by_key(|entity| entity.id.number());

        let total = entities.len();
        let entities = entities.into_iter().skip(offset).take(limit).collect::<Vec<_>>();
        let returned_end = offset.saturating_add(entities.len());
        let next_offset = (returned_end < total).then_some(returned_end);

        Ok(EntitiesPage {
            filters: filters.to_vec(),
            offset,
            limit,
            total,
            next_offset,
            entities,
        })
    }
}

pub(super) fn load_entities(root: &Path) -> Result<Vec<Entity>, Error> {
    let directory = root.join("entities");
    let entries = fs::read_dir(&directory).map_err(|source| Error::Read { path: directory.clone(), source })?;
    let mut paths = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|source| Error::Read { path: directory.clone(), source })?;
        let path = entry.path();
        let file_type = entry.file_type().map_err(|source| Error::Read { path: path.clone(), source })?;
        if file_type.is_file() && path.extension().and_then(|extension| extension.to_str()) == Some("yaml") {
            paths.push(path);
        }
    }
    paths.sort();

    let mut loaded = Vec::with_capacity(paths.len());
    for path in paths {
        let source = fs::read_to_string(&path).map_err(|source| Error::Read { path: path.clone(), source })?;
        let entity: Entity = serde_yaml::from_str(&source).map_err(|source| Error::ParseEntity { path: path.clone(), source })?;
        loaded.push((path, entity));
    }

    let mut ids = BTreeSet::new();
    for (_, entity) in &loaded {
        if !ids.insert(entity.id.clone()) {
            return Err(Error::InvalidRepository(format!("duplicate entity identifier {}", entity.id)));
        }
    }

    let mut entities = Vec::with_capacity(loaded.len());
    for (path, entity) in loaded {
        let file_id = path.file_stem().and_then(|name| name.to_str());
        if file_id != Some(entity.id.as_str()) {
            return Err(Error::InvalidRepository(format!(
                "entity file {} declares identifier {} instead of {}",
                path.display(),
                entity.id,
                file_id.unwrap_or("<non-UTF-8 filename>")
            )));
        }
        entities.push(entity);
    }
    Ok(entities)
}
