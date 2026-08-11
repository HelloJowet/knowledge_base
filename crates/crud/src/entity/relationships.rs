use super::Entities;
use crate::{Error, resource};
use knowledge_base_models::{Entity, EntityId, PropertyId, StatementId, Value};
use serde::Serialize;
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct EntityRelationshipsPage {
    pub entity: EntityId,
    pub offset: usize,
    pub limit: usize,
    pub total: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_offset: Option<usize>,
    pub relationships: Vec<EntityRelationship>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct EntityRelationship {
    pub direction: RelationshipDirection,
    pub entity: RelatedEntity,
    pub property: PropertyId,
    pub statement: StatementId,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct RelatedEntity {
    pub id: EntityId,
    pub labels: BTreeMap<String, String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum RelationshipDirection {
    Incoming,
    Outgoing,
    #[serde(rename = "self")]
    SelfReference,
}

#[derive(Clone, Debug)]
struct Edge {
    source: EntityId,
    property: PropertyId,
    statement: StatementId,
    target: EntityId,
}

impl Entities<'_> {
    pub fn relationships(&self, id: &EntityId, limit: usize, offset: usize) -> Result<EntityRelationshipsPage, Error> {
        if limit == 0 {
            return Err(Error::InvalidRequest("relationship limit must be greater than zero".to_owned()));
        }

        // Resolve the canonical file first so a missing requested entity remains a
        // normal resource-read error, even if another file declares the same ID.
        let requested_path = resource::path(self.knowledge_base.root(), "entities", id.as_str(), "yaml");
        let requested = parse_entity(&requested_path)?;
        if &requested.id != id {
            return Err(Error::InvalidRepository(format!(
                "entity file {} declares identifier {} instead of {}",
                requested_path.display(),
                requested.id,
                id
            )));
        }

        let entities = load_entities(self.knowledge_base.root())?;
        let mut index = BTreeMap::new();
        for entity in &entities {
            if index.insert(entity.id.clone(), entity).is_some() {
                return Err(Error::InvalidRepository(format!("duplicate entity identifier {}", entity.id)));
            }
        }

        let mut edges = Vec::new();
        for entity in &entities {
            for statement in &entity.statements {
                let Value::Entity { value: target } = &statement.value else {
                    continue;
                };
                if &entity.id == id || target == id {
                    edges.push(Edge {
                        source: entity.id.clone(),
                        property: statement.property.clone(),
                        statement: statement.id.clone(),
                        target: target.clone(),
                    });
                }
            }
        }
        edges.sort_by_key(|edge| (edge.source.number(), edge.property.number(), edge.statement.number(), edge.target.number()));

        let total = edges.len();
        let relationships = edges
            .into_iter()
            .skip(offset)
            .take(limit)
            .map(|edge| relationship(id, &index, edge))
            .collect::<Result<Vec<_>, _>>()?;
        let returned_end = offset.saturating_add(relationships.len());
        let next_offset = (returned_end < total).then_some(returned_end);

        Ok(EntityRelationshipsPage {
            entity: id.clone(),
            offset,
            limit,
            total,
            next_offset,
            relationships,
        })
    }
}

fn load_entities(root: &Path) -> Result<Vec<Entity>, Error> {
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
    paths.iter().map(|path| parse_entity(path.as_path())).collect()
}

fn parse_entity(path: &Path) -> Result<Entity, Error> {
    let source = fs::read_to_string(path).map_err(|source| Error::Read { path: path.to_path_buf(), source })?;
    serde_yaml::from_str(&source).map_err(|source| Error::ParseEntity { path: path.to_path_buf(), source })
}

fn relationship(id: &EntityId, index: &BTreeMap<EntityId, &Entity>, edge: Edge) -> Result<EntityRelationship, Error> {
    let (direction, related_id) = if &edge.source == id && &edge.target == id {
        (RelationshipDirection::SelfReference, id)
    } else if &edge.source == id {
        (RelationshipDirection::Outgoing, &edge.target)
    } else {
        (RelationshipDirection::Incoming, &edge.source)
    };
    let related = index
        .get(related_id)
        .ok_or_else(|| Error::InvalidRepository(format!("relationship {} on {} targets missing entity {}", edge.statement, edge.source, edge.target)))?;
    let labels = related.labels.iter().map(|(language, label)| (language.clone(), label.text.clone())).collect();

    Ok(EntityRelationship {
        direction,
        entity: RelatedEntity { id: related_id.clone(), labels },
        property: edge.property,
        statement: edge.statement,
    })
}
