//! Read-only snapshots of canonical knowledge-base repository resources.
//!
//! Loading verifies repository structure and YAML deserialization, but does not
//! perform generic or domain semantic validation.

#![forbid(unsafe_code)]

use knowledge_base_models::{Entity, EntityId, EntityType, EntityTypeId, IdAllocation, Property, PropertyId, Reference, ReferenceId};
use std::collections::BTreeMap;
use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// An error loading a structured repository resource.
#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    Read { path: PathBuf, source: io::Error },
    ParseEntity { path: PathBuf, source: serde_yaml::Error },
    ParseEntityType { path: PathBuf, source: serde_yaml::Error },
    ParseProperty { path: PathBuf, source: serde_yaml::Error },
    ParseReference { path: PathBuf, source: serde_yaml::Error },
    ParseAllocation { path: PathBuf, source: serde_yaml::Error },
    InvalidSnapshot { path: PathBuf, message: String },
}

impl Error {
    /// The absolute or caller-provided path associated with this failure.
    pub fn path(&self) -> &Path {
        match self {
            Self::Read { path, .. }
            | Self::ParseEntity { path, .. }
            | Self::ParseEntityType { path, .. }
            | Self::ParseProperty { path, .. }
            | Self::ParseReference { path, .. }
            | Self::ParseAllocation { path, .. }
            | Self::InvalidSnapshot { path, .. } => path,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Read { path, source } => write!(formatter, "cannot read {}: {source}", path.display()),
            Self::ParseEntity { path, source } => write!(formatter, "cannot parse entity {}: {source}", path.display()),
            Self::ParseEntityType { path, source } => write!(formatter, "cannot parse entity type {}: {source}", path.display()),
            Self::ParseProperty { path, source } => write!(formatter, "cannot parse property {}: {source}", path.display()),
            Self::ParseReference { path, source } => write!(formatter, "cannot parse reference {}: {source}", path.display()),
            Self::ParseAllocation { path, source } => write!(formatter, "cannot parse identifier allocation {}: {source}", path.display()),
            Self::InvalidSnapshot { path, message } => write!(formatter, "cannot load repository snapshot at {}: {message}", path.display()),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Read { source, .. } => Some(source),
            Self::ParseEntity { source, .. }
            | Self::ParseEntityType { source, .. }
            | Self::ParseProperty { source, .. }
            | Self::ParseReference { source, .. }
            | Self::ParseAllocation { source, .. } => Some(source),
            Self::InvalidSnapshot { .. } => None,
        }
    }
}

/// A complete, immutable, structurally valid view of a knowledge-base repository.
///
/// Resources are indexed by identifiers and iterate in identifier order.
#[derive(Clone, Debug)]
pub struct RepositorySnapshot {
    entities: BTreeMap<EntityId, Entity>,
    entity_types: BTreeMap<EntityTypeId, EntityType>,
    properties: BTreeMap<PropertyId, Property>,
    references: BTreeMap<ReferenceId, Reference>,
    allocation: IdAllocation,
}

impl RepositorySnapshot {
    /// Loads all managed structured resources from `root`.
    pub fn load(root: impl AsRef<Path>) -> Result<Self, Error> {
        let root = root.as_ref();
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
    type Id: Ord + Clone + fmt::Display;
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
        if path.file_stem().and_then(|stem| stem.to_str()) != Some(resource.id().to_string().as_str()) {
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
