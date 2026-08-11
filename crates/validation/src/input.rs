use crate::diagnostic::{Diagnostics, ValidationLayer};
use knowledge_base_models::{Entity, EntityId, EntityType, IdAllocation, Property, Reference};
use serde::de::DeserializeOwned;
use std::fs;
use std::path::{Path, PathBuf};

pub(crate) struct Loaded<T> {
    pub(crate) path: PathBuf,
    pub(crate) value: T,
}

pub(crate) struct ContextDocument {
    pub(crate) path: PathBuf,
    pub(crate) entity_id: EntityId,
    pub(crate) source: String,
}

pub(crate) struct LoadedRepository {
    pub(crate) entities: Vec<Loaded<Entity>>,
    pub(crate) entity_types: Vec<Loaded<EntityType>>,
    pub(crate) properties: Vec<Loaded<Property>>,
    pub(crate) references: Vec<Loaded<Reference>>,
    pub(crate) allocation: Option<Loaded<IdAllocation>>,
    pub(crate) contexts: Vec<ContextDocument>,
}

impl LoadedRepository {
    pub(crate) fn load(root: &Path, diagnostics: &mut Diagnostics) -> Self {
        let entities = load_yaml_directory::<Entity>(root, "entities", diagnostics);
        let entity_types = load_yaml_directory::<EntityType>(root, "entity_types", diagnostics);
        let properties = load_yaml_directory::<Property>(root, "properties", diagnostics);
        let references = load_yaml_directory::<Reference>(root, "references", diagnostics);
        let allocation = load_yaml_file::<IdAllocation>(root, Path::new("id_allocation.yaml"), diagnostics);
        let contexts = load_contexts(root, diagnostics);

        validate_filenames(&entities, "entity", |item| item.id.as_str(), diagnostics);
        validate_filenames(&entity_types, "entity type", |item| item.id.as_str(), diagnostics);
        validate_filenames(&properties, "property", |item| item.id.as_str(), diagnostics);
        validate_filenames(&references, "reference", |item| item.id.as_str(), diagnostics);

        Self {
            entities,
            entity_types,
            properties,
            references,
            allocation,
            contexts,
        }
    }
}

fn load_yaml_directory<T: DeserializeOwned>(root: &Path, directory: &str, diagnostics: &mut Diagnostics) -> Vec<Loaded<T>> {
    let relative = PathBuf::from(directory);
    let path = root.join(&relative);
    let entries = match fs::read_dir(&path) {
        Ok(entries) => entries,
        Err(error) => {
            diagnostics.push(ValidationLayer::Schema, relative, None, None, format!("required directory cannot be read: {error}"));
            return Vec::new();
        }
    };

    let mut paths = Vec::new();
    for entry in entries {
        match entry {
            Ok(entry) => paths.push(entry.path()),
            Err(error) => diagnostics.push(
                ValidationLayer::Schema,
                PathBuf::from(directory),
                None,
                None,
                format!("directory entry cannot be read: {error}"),
            ),
        }
    }
    paths.sort();

    let mut loaded = Vec::new();
    for path in paths {
        let relative_path = relative_path(root, &path);
        let is_yaml_file = path.is_file() && path.extension().and_then(|extension| extension.to_str()) == Some("yaml");
        if !is_yaml_file {
            diagnostics.push(
                ValidationLayer::Schema,
                relative_path,
                None,
                None,
                "unexpected entry; managed directories may contain only .yaml files",
            );
            continue;
        }
        if let Some(value) = load_yaml_at::<T>(&path, relative_path.clone(), diagnostics) {
            loaded.push(Loaded { path: relative_path, value });
        }
    }
    loaded
}

fn load_yaml_file<T: DeserializeOwned>(root: &Path, relative: &Path, diagnostics: &mut Diagnostics) -> Option<Loaded<T>> {
    let value = load_yaml_at::<T>(&root.join(relative), relative.to_path_buf(), diagnostics)?;
    Some(Loaded {
        path: relative.to_path_buf(),
        value,
    })
}

fn load_yaml_at<T: DeserializeOwned>(path: &Path, relative: PathBuf, diagnostics: &mut Diagnostics) -> Option<T> {
    let source = match fs::read_to_string(path) {
        Ok(source) => source,
        Err(error) => {
            diagnostics.push(ValidationLayer::Schema, relative, None, None, format!("file cannot be read: {error}"));
            return None;
        }
    };

    let value = match serde_yaml::from_str::<serde_yaml::Value>(&source) {
        Ok(value) => value,
        Err(error) => {
            diagnostics.push(
                ValidationLayer::Schema,
                relative,
                error.location().map(|location| location.line()),
                None,
                format!("invalid YAML: {error}"),
            );
            return None;
        }
    };
    match serde_yaml::from_value(value) {
        Ok(value) => Some(value),
        Err(error) => {
            diagnostics.push(
                ValidationLayer::Schema,
                relative,
                error.location().map(|location| location.line()),
                None,
                format!("invalid file shape: {error}"),
            );
            None
        }
    }
}

fn load_contexts(root: &Path, diagnostics: &mut Diagnostics) -> Vec<ContextDocument> {
    let directory = root.join("entity_context");
    if !directory.exists() {
        return Vec::new();
    }
    let entries = match fs::read_dir(&directory) {
        Ok(entries) => entries,
        Err(error) => {
            diagnostics.push(
                ValidationLayer::Schema,
                PathBuf::from("entity_context"),
                None,
                None,
                format!("optional context directory cannot be read: {error}"),
            );
            return Vec::new();
        }
    };

    let mut paths = entries.filter_map(Result::ok).map(|entry| entry.path()).collect::<Vec<_>>();
    paths.sort();
    let mut contexts = Vec::new();
    for path in paths {
        let relative = relative_path(root, &path);
        if !path.is_file() || path.extension().and_then(|value| value.to_str()) != Some("md") {
            diagnostics.push(ValidationLayer::Schema, relative, None, None, "unexpected entry; entity_context may contain only .md files");
            continue;
        }
        let Some(stem) = path.file_stem().and_then(|value| value.to_str()) else {
            diagnostics.push(ValidationLayer::Schema, relative, None, None, "context filename is not valid UTF-8");
            continue;
        };
        let entity_id = match serde_yaml::from_value::<EntityId>(stem.into()) {
            Ok(identifier) => identifier,
            Err(_) => {
                diagnostics.push(
                    ValidationLayer::Schema,
                    relative,
                    None,
                    Some(stem.to_owned()),
                    "context filename must be a canonical entity identifier",
                );
                continue;
            }
        };
        match fs::read_to_string(&path) {
            Ok(source) => contexts.push(ContextDocument {
                path: relative,
                entity_id,
                source,
            }),
            Err(error) => diagnostics.push(
                ValidationLayer::Schema,
                relative,
                None,
                Some(entity_id.to_string()),
                format!("context document cannot be read: {error}"),
            ),
        }
    }
    contexts
}

fn validate_filenames<T, F>(items: &[Loaded<T>], kind: &str, identifier: F, diagnostics: &mut Diagnostics)
where
    F: Fn(&T) -> &str,
{
    for item in items {
        let stem = item.path.file_stem().and_then(|value| value.to_str());
        let id = identifier(&item.value);
        if stem != Some(id) {
            diagnostics.push(
                ValidationLayer::Schema,
                item.path.clone(),
                None,
                Some(id.to_owned()),
                format!("{kind} filename must be exactly {id}.yaml"),
            );
        }
    }
}

fn relative_path(root: &Path, path: &Path) -> PathBuf {
    path.strip_prefix(root).map(Path::to_path_buf).unwrap_or_else(|_| path.to_path_buf())
}
