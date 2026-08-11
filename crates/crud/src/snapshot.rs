use crate::Error;
use knowledge_base_models::{Entity, EntityId, EntityType, EntityTypeId, IdAllocation, Property, PropertyId, Reference, ReferenceId};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

/// A complete, immutable, structurally valid view of a knowledge-base repository.
///
/// Resources are indexed by their identifiers and iterate in identifier order.
/// Loading a snapshot does not perform generic or domain semantic validation.
#[derive(Clone, Debug)]
pub struct RepositorySnapshot {
    entities: BTreeMap<EntityId, Entity>,
    entity_types: BTreeMap<EntityTypeId, EntityType>,
    properties: BTreeMap<PropertyId, Property>,
    references: BTreeMap<ReferenceId, Reference>,
    allocation: IdAllocation,
}

impl RepositorySnapshot {
    pub(crate) fn load(root: &Path) -> Result<Self, Error> {
        Ok(Self {
            entities: load_resources(root, "entities", |path, source| {
                serde_yaml::from_slice(source).map_err(|source| Error::ParseEntity { path, source })
            })?,
            entity_types: load_resources(root, "entity_types", |path, source| {
                serde_yaml::from_slice(source).map_err(|source| Error::ParseEntityType { path, source })
            })?,
            properties: load_resources(root, "properties", |path, source| {
                serde_yaml::from_slice(source).map_err(|source| Error::ParseProperty { path, source })
            })?,
            references: load_resources(root, "references", |path, source| {
                serde_yaml::from_slice(source).map_err(|source| Error::ParseReference { path, source })
            })?,
            allocation: load_allocation(root)?,
        })
    }

    pub fn entities(&self) -> &BTreeMap<EntityId, Entity> {
        &self.entities
    }
    pub fn entity_types(&self) -> &BTreeMap<EntityTypeId, EntityType> {
        &self.entity_types
    }
    pub fn properties(&self) -> &BTreeMap<PropertyId, Property> {
        &self.properties
    }
    pub fn references(&self) -> &BTreeMap<ReferenceId, Reference> {
        &self.references
    }
    pub fn allocation(&self) -> &IdAllocation {
        &self.allocation
    }
}

trait Identified {
    type Id: Ord + Clone + std::fmt::Display;
    fn id(&self) -> &Self::Id;
}

impl Identified for Entity {
    type Id = EntityId;
    fn id(&self) -> &Self::Id {
        &self.id
    }
}
impl Identified for EntityType {
    type Id = EntityTypeId;
    fn id(&self) -> &Self::Id {
        &self.id
    }
}
impl Identified for Property {
    type Id = PropertyId;
    fn id(&self) -> &Self::Id {
        &self.id
    }
}
impl Identified for Reference {
    type Id = ReferenceId;
    fn id(&self) -> &Self::Id {
        &self.id
    }
}

fn load_resources<T: Identified>(root: &Path, directory: &str, parse: impl Fn(PathBuf, &[u8]) -> Result<T, Error>) -> Result<BTreeMap<T::Id, T>, Error> {
    let directory_path = root.join(directory);
    let entries = fs::read_dir(&directory_path).map_err(|source| Error::Read {
        path: directory_path.clone(),
        source,
    })?;
    let mut paths = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|source| Error::Read {
            path: directory_path.clone(),
            source,
        })?;
        let path = entry.path();
        let file_type = entry.file_type().map_err(|source| Error::Read { path: path.clone(), source })?;
        if !file_type.is_file() || path.extension().and_then(|extension| extension.to_str()) != Some("yaml") {
            return Err(Error::InvalidSnapshot {
                path,
                message: format!("managed directory {directory} may contain only YAML files"),
            });
        }
        paths.push(path);
    }
    paths.sort();

    let mut resources = BTreeMap::new();
    for path in paths {
        let source = fs::read(&path).map_err(|source| Error::Read { path: path.clone(), source })?;
        let resource = parse(path.clone(), &source)?;
        let file_id = path.file_stem().and_then(|stem| stem.to_str());
        if file_id != Some(resource.id().to_string().as_str()) {
            return Err(Error::InvalidSnapshot {
                path,
                message: format!("filename must match declared identifier {}", resource.id()),
            });
        }
        if resources.insert(resource.id().clone(), resource).is_some() {
            return Err(Error::InvalidSnapshot {
                path,
                message: "duplicate resource identifier".to_owned(),
            });
        }
    }
    Ok(resources)
}

fn load_allocation(root: &Path) -> Result<IdAllocation, Error> {
    let path = root.join("id_allocation.yaml");
    let source = fs::read(&path).map_err(|source| Error::Read { path: path.clone(), source })?;
    serde_yaml::from_slice(&source).map_err(|source| Error::ParseAllocation { path, source })
}
