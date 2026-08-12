//! Versioned, repository-local configuration for compiled extensions.

use crate::bindings::{BindingValue, ResolvedBindings};
use crate::contracts::{BindingKey, BindingKind, BindingReference, ContractVersion, ExtensionId};
use crate::error::FrameworkError;
use crate::ontology::{OntologyContractDiagnostic, verify_ontology_contracts};
use crate::registry::{ActiveExtensions, ExtensionRegistry};
use knowledge_base_models::{EntityTypeId, PropertyId};
use knowledge_base_snapshot::{Error as SnapshotError, RepositorySnapshot};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

pub const MANIFEST_FILE_NAME: &str = "extensions.yaml";
pub const MANIFEST_VERSION: u32 = 1;

/// The versioned extension configuration stored in a repository's `extensions.yaml`.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ExtensionManifest {
    pub version: u32,
    #[serde(default)]
    pub extensions: BTreeMap<ExtensionId, ManifestExtension>,
}

/// The contract and semantic bindings activated for one extension.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ManifestExtension {
    pub contract: ContractVersion,
    #[serde(default)]
    pub entity_types: BTreeMap<BindingKey, EntityTypeId>,
    #[serde(default)]
    pub properties: BTreeMap<BindingKey, PropertyId>,
}

/// One stable semantic error found while validating a syntactically valid manifest.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ManifestDiagnostic {
    UnsupportedVersion {
        version: u32,
    },
    UnavailableExtension {
        extension: ExtensionId,
    },
    UnsupportedContract {
        extension: ExtensionId,
        declared: ContractVersion,
        available: ContractVersion,
    },
    MissingDependency {
        extension: ExtensionId,
        dependency: ExtensionId,
    },
    DuplicateBinding {
        binding: BindingReference,
    },
    UndeclaredBinding {
        binding: BindingReference,
    },
    BindingKindMismatch {
        binding: BindingReference,
        expected: BindingKind,
        actual: BindingKind,
    },
    MissingBinding {
        binding: BindingReference,
    },
    MissingEntityType {
        binding: BindingReference,
        id: EntityTypeId,
    },
    MissingProperty {
        binding: BindingReference,
        id: PropertyId,
    },
}

impl fmt::Display for ManifestDiagnostic {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedVersion { version } => write!(formatter, "unsupported extension manifest version {version}; expected {MANIFEST_VERSION}"),
            Self::UnavailableExtension { extension } => write!(formatter, "extension {extension} is not compiled into this distribution"),
            Self::UnsupportedContract { extension, declared, available } => write!(
                formatter,
                "extension {extension} declares contract {declared}, but compiled implementation supports {available}"
            ),
            Self::MissingDependency { extension, dependency } => write!(formatter, "extension {extension} requires declared dependency {dependency}"),
            Self::DuplicateBinding { binding } => write!(formatter, "binding {binding} is declared in more than one typed segment"),
            Self::UndeclaredBinding { binding } => write!(formatter, "extension does not declare binding {binding}"),
            Self::BindingKindMismatch { binding, expected, actual } => write!(formatter, "binding {binding} is declared as {actual:?}, but extension requires {expected:?}"),
            Self::MissingBinding { binding } => write!(formatter, "manifest is missing required binding {binding}"),
            Self::MissingEntityType { binding, id } => write!(formatter, "binding {binding} resolves missing entity type {id}"),
            Self::MissingProperty { binding, id } => write!(formatter, "binding {binding} resolves missing property {id}"),
        }
    }
}

/// Errors loading, activating, or resolving a repository extension manifest.
#[derive(Debug)]
pub enum ManifestError {
    Read { path: PathBuf, source: io::Error },
    Parse { path: PathBuf, source: serde_yaml::Error },
    Diagnostics { path: PathBuf, diagnostics: Vec<ManifestDiagnostic> },
    OntologyContracts { path: PathBuf, diagnostics: Vec<OntologyContractDiagnostic> },
    Framework(FrameworkError),
    Snapshot(SnapshotError),
}

impl fmt::Display for ManifestError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Read { path, source } => write!(formatter, "cannot read extension manifest {}: {source}", path.display()),
            Self::Parse { path, source } => write!(formatter, "cannot parse extension manifest {}: {source}", path.display()),
            Self::Diagnostics { path, diagnostics } => write!(
                formatter,
                "invalid extension manifest {}: {}",
                path.display(),
                diagnostics.iter().map(ToString::to_string).collect::<Vec<_>>().join("; ")
            ),
            Self::OntologyContracts { path, diagnostics } => write!(
                formatter,
                "invalid extension ontology contracts for {}: {}",
                path.display(),
                diagnostics.iter().map(ToString::to_string).collect::<Vec<_>>().join("; ")
            ),
            Self::Framework(source) => source.fmt(formatter),
            Self::Snapshot(source) => source.fmt(formatter),
        }
    }
}

impl std::error::Error for ManifestError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Read { source, .. } => Some(source),
            Self::Parse { source, .. } => Some(source),
            Self::Framework(source) => Some(source),
            Self::Snapshot(source) => Some(source),
            Self::Diagnostics { .. } | Self::OntologyContracts { .. } => None,
        }
    }
}

/// The dependency-ordered extensions and typed bindings enabled for one repository.
pub struct ManifestActivation {
    active: ActiveExtensions,
    bindings: ResolvedBindings,
}

impl fmt::Debug for ManifestActivation {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ManifestActivation")
            .field("active_extension_count", &self.active.extensions().len())
            .field("binding_count", &self.bindings.values.len())
            .finish()
    }
}

impl ManifestActivation {
    pub fn active(&self) -> &ActiveExtensions {
        &self.active
    }

    pub fn bindings(&self) -> &ResolvedBindings {
        &self.bindings
    }
}

impl ExtensionManifest {
    pub fn path(root: impl AsRef<Path>) -> PathBuf {
        root.as_ref().join(MANIFEST_FILE_NAME)
    }

    /// Loads `extensions.yaml` before any repository resources are opened.
    pub fn load(root: impl AsRef<Path>) -> Result<Self, ManifestError> {
        let path = Self::path(root);
        let source = fs::read_to_string(&path).map_err(|source| ManifestError::Read { path: path.clone(), source })?;
        serde_yaml::from_str(&source).map_err(|source| ManifestError::Parse { path, source })
    }

    /// Loads, validates, activates, and resolves bindings against repository ontology records.
    pub fn load_and_activate(root: impl AsRef<Path>, registry: &ExtensionRegistry) -> Result<ManifestActivation, ManifestError> {
        let root = root.as_ref();
        Self::load(root)?.activate(root, registry)
    }

    /// Activates this manifest, loading repository resources only after manifest checks succeed.
    pub fn activate(&self, root: impl AsRef<Path>, registry: &ExtensionRegistry) -> Result<ManifestActivation, ManifestError> {
        let root = root.as_ref();
        let path = Self::path(root);
        let mut diagnostics = Vec::new();
        if self.version != MANIFEST_VERSION {
            diagnostics.push(ManifestDiagnostic::UnsupportedVersion { version: self.version });
        }

        let mut available = BTreeSet::new();
        for (id, extension) in &self.extensions {
            match registry.metadata(id) {
                None => diagnostics.push(ManifestDiagnostic::UnavailableExtension { extension: id.clone() }),
                Some(metadata) if metadata.contract != extension.contract => diagnostics.push(ManifestDiagnostic::UnsupportedContract {
                    extension: id.clone(),
                    declared: extension.contract,
                    available: metadata.contract,
                }),
                Some(_) => {
                    available.insert(id.clone());
                }
            }
        }

        for id in &available {
            let metadata = registry.metadata(id).expect("available extensions have metadata");
            for dependency in &metadata.dependencies {
                if !self.extensions.contains_key(&dependency.id) {
                    diagnostics.push(ManifestDiagnostic::MissingDependency {
                        extension: id.clone(),
                        dependency: dependency.id.clone(),
                    });
                }
            }
        }

        let mut values = BTreeMap::new();
        for (id, extension) in &self.extensions {
            let Some(metadata) = registry.metadata(id) else {
                continue;
            };
            for (key, value) in &extension.entity_types {
                let reference = BindingReference::new(id.clone(), key.clone());
                validate_binding(metadata, &reference, BindingValue::EntityType(value.clone()), &mut values, &mut diagnostics);
            }
            for (key, value) in &extension.properties {
                let reference = BindingReference::new(id.clone(), key.clone());
                validate_binding(metadata, &reference, BindingValue::Property(value.clone()), &mut values, &mut diagnostics);
            }
            let mut declarations = metadata.bindings.iter().collect::<Vec<_>>();
            declarations.sort_by(|left, right| left.key.cmp(&right.key));
            for declaration in declarations {
                let reference = BindingReference::new(id.clone(), declaration.key.clone());
                if !values.contains_key(&reference) {
                    diagnostics.push(ManifestDiagnostic::MissingBinding { binding: reference });
                }
            }
        }

        if !diagnostics.is_empty() {
            return Err(ManifestError::Diagnostics { path, diagnostics });
        }

        let active = registry.resolve_active(self.extensions.keys().cloned()).map_err(ManifestError::Framework)?;
        let bindings = active.resolve_bindings(values).map_err(ManifestError::Framework)?;
        let snapshot = RepositorySnapshot::load(root).map_err(ManifestError::Snapshot)?;
        let mut diagnostics = Vec::new();
        for (binding, value) in &bindings.values {
            match value {
                BindingValue::EntityType(id) if !snapshot.entity_types().contains_key(id) => {
                    diagnostics.push(ManifestDiagnostic::MissingEntityType {
                        binding: binding.clone(),
                        id: id.clone(),
                    });
                }
                BindingValue::Property(id) if !snapshot.properties().contains_key(id) => {
                    diagnostics.push(ManifestDiagnostic::MissingProperty {
                        binding: binding.clone(),
                        id: id.clone(),
                    });
                }
                _ => {}
            }
        }
        if !diagnostics.is_empty() {
            return Err(ManifestError::Diagnostics {
                path: Self::path(root),
                diagnostics,
            });
        }
        let diagnostics = verify_ontology_contracts(&snapshot, &active, &bindings);
        if !diagnostics.is_empty() {
            return Err(ManifestError::OntologyContracts {
                path: Self::path(root),
                diagnostics,
            });
        }
        Ok(ManifestActivation { active, bindings })
    }
}

fn validate_binding(
    metadata: &crate::contracts::ExtensionMetadata,
    reference: &BindingReference,
    value: BindingValue,
    values: &mut BTreeMap<BindingReference, BindingValue>,
    diagnostics: &mut Vec<ManifestDiagnostic>,
) {
    let Some(declaration) = metadata.bindings.iter().find(|declaration| declaration.key == *reference.key()) else {
        diagnostics.push(ManifestDiagnostic::UndeclaredBinding { binding: reference.clone() });
        return;
    };
    if declaration.kind != value.kind() {
        diagnostics.push(ManifestDiagnostic::BindingKindMismatch {
            binding: reference.clone(),
            expected: declaration.kind,
            actual: value.kind(),
        });
        return;
    }
    if values.insert(reference.clone(), value).is_some() {
        diagnostics.push(ManifestDiagnostic::DuplicateBinding { binding: reference.clone() });
    }
}
