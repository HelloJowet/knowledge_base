mod contexts;
mod entities;
mod records;
mod values;

use crate::diagnostic::{Diagnostics, ValidationLayer};
use crate::input::{Loaded, LoadedRepository};
use knowledge_base_models::{Entity, EntityId, EntityType, EntityTypeId, Property, PropertyId, Reference, ReferenceId};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

pub(crate) fn validate_repository(root: &Path) -> Vec<crate::Diagnostic> {
    let mut diagnostics = Diagnostics::default();
    if !root.is_dir() {
        diagnostics.push(
            ValidationLayer::Schema,
            PathBuf::from("."),
            None,
            None,
            format!("knowledge-base root is not a readable directory: {}", root.display()),
        );
        return diagnostics.finish();
    }

    let repository = LoadedRepository::load(root, &mut diagnostics);
    let indexes = RepositoryIndexes::build(&repository, &mut diagnostics);
    Validator {
        repository: &repository,
        indexes,
        diagnostics,
    }
    .run()
}

pub(super) struct RepositoryIndexes<'a> {
    pub(super) entities: BTreeMap<EntityId, &'a Loaded<Entity>>,
    pub(super) entity_types: BTreeMap<EntityTypeId, &'a Loaded<EntityType>>,
    pub(super) properties: BTreeMap<PropertyId, &'a Loaded<Property>>,
    pub(super) references: BTreeMap<ReferenceId, &'a Loaded<Reference>>,
}

impl<'a> RepositoryIndexes<'a> {
    fn build(repository: &'a LoadedRepository, diagnostics: &mut Diagnostics) -> Self {
        Self {
            entities: build_index(&repository.entities, |item| item.id.clone(), "entity", diagnostics),
            entity_types: build_index(&repository.entity_types, |item| item.id.clone(), "entity type", diagnostics),
            properties: build_index(&repository.properties, |item| item.id.clone(), "property", diagnostics),
            references: build_index(&repository.references, |item| item.id.clone(), "reference", diagnostics),
        }
    }
}

pub(super) struct Validator<'a> {
    pub(super) repository: &'a LoadedRepository,
    pub(super) indexes: RepositoryIndexes<'a>,
    pub(super) diagnostics: Diagnostics,
}

impl Validator<'_> {
    fn run(mut self) -> Vec<crate::Diagnostic> {
        records::validate(&mut self);
        entities::validate(&mut self);
        contexts::validate(&mut self);
        self.diagnostics.finish()
    }
}

fn build_index<'a, T, I, F>(items: &'a [Loaded<T>], identifier: F, kind: &str, diagnostics: &mut Diagnostics) -> BTreeMap<I, &'a Loaded<T>>
where
    I: Clone + Ord + ToString,
    F: Fn(&T) -> I,
{
    let mut index = BTreeMap::new();
    for item in items {
        let id = identifier(&item.value);
        if let Some(previous) = index.insert(id.clone(), item) {
            diagnostics.push(
                ValidationLayer::Schema,
                item.path.clone(),
                None,
                Some(id.to_string()),
                format!("duplicate {kind} identifier; also declared in {}", previous.path.display()),
            );
        }
    }
    index
}
