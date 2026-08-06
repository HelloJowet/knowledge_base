mod diagnostic;

pub use diagnostic::{Diagnostic, ValidationLayer};

use chrono::{DateTime, NaiveDate};
use knowledge_base_models::{Cardinality, Entity, EntityId, EntityType, EntityTypeId, IdAllocation, LocalizedMap, Property, PropertyId, Reference, ReferenceId, Value, ValueType};
use language_tags::LanguageTag;
use pulldown_cmark::{Event, Options, Parser, Tag};
use regex::Regex;
use serde::de::DeserializeOwned;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;
use url::Url;

static DECIMAL: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^-?(0|[1-9][0-9]*)(\.[0-9]+)?$").expect("valid regex"));
static DATE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^[0-9]{4}-[0-9]{2}-[0-9]{2}$").expect("valid regex"));

struct Loaded<T> {
    path: PathBuf,
    value: T,
}

struct ContextDocument {
    path: PathBuf,
    entity_id: EntityId,
    source: String,
}

type Diagnostics = Vec<Diagnostic>;

pub fn validate_repository(root: impl AsRef<Path>) -> Vec<Diagnostic> {
    let root = root.as_ref();
    let mut report = Vec::new();

    if !root.is_dir() {
        push(
            &mut report,
            ValidationLayer::Schema,
            PathBuf::from("."),
            None,
            None,
            format!("knowledge-base root is not a readable directory: {}", root.display()),
        );
        sort_diagnostics(&mut report);
        return report;
    }

    let entities = load_yaml_directory::<Entity>(root, "entities", &mut report);
    let entity_types = load_yaml_directory::<EntityType>(root, "entity_types", &mut report);
    let properties = load_yaml_directory::<Property>(root, "properties", &mut report);
    let references = load_yaml_directory::<Reference>(root, "references", &mut report);
    let allocation = load_yaml_file::<IdAllocation>(root, Path::new("id_allocation.yaml"), &mut report);
    let contexts = load_contexts(root, &mut report);

    let entity_index = build_index(&entities, |item| item.id.clone(), "entity", &mut report);
    let type_index = build_index(&entity_types, |item| item.id.clone(), "entity type", &mut report);
    let property_index = build_index(&properties, |item| item.id.clone(), "property", &mut report);
    let reference_index = build_index(&references, |item| item.id.clone(), "reference", &mut report);

    validate_filenames(&entities, "entity", |item| item.id.as_str(), &mut report);
    validate_filenames(&entity_types, "entity type", |item| item.id.as_str(), &mut report);
    validate_filenames(&properties, "property", |item| item.id.as_str(), &mut report);
    validate_filenames(&references, "reference", |item| item.id.as_str(), &mut report);

    validate_entity_types(&entity_types, &reference_index, &mut report);
    validate_properties(&properties, &type_index, &property_index, &reference_index, &mut report);
    validate_references(&references, &mut report);
    validate_entities(&entities, &entity_index, &type_index, &property_index, &reference_index, &mut report);

    if let Some(allocation) = allocation.as_ref() {
        validate_allocation(
            allocation,
            entity_index.keys().map(EntityId::number).max(),
            property_index.keys().map(PropertyId::number).max(),
            reference_index.keys().map(ReferenceId::number).max(),
            type_index.keys().map(EntityTypeId::number).max(),
            &mut report,
        );
    }

    validate_contexts(&contexts, &entity_index, &reference_index, &mut report);
    sort_diagnostics(&mut report);
    report
}

fn load_yaml_directory<T: DeserializeOwned>(root: &Path, directory: &str, report: &mut Diagnostics) -> Vec<Loaded<T>> {
    let relative = PathBuf::from(directory);
    let path = root.join(&relative);
    let entries = match fs::read_dir(&path) {
        Ok(entries) => entries,
        Err(error) => {
            push(report, ValidationLayer::Schema, relative, None, None, format!("required directory cannot be read: {error}"));
            return Vec::new();
        }
    };

    let mut paths = Vec::new();
    for entry in entries {
        match entry {
            Ok(entry) => paths.push(entry.path()),
            Err(error) => push(
                report,
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
            push(
                report,
                ValidationLayer::Schema,
                relative_path,
                None,
                None,
                "unexpected entry; managed directories may contain only .yaml files",
            );
            continue;
        }
        if let Some(value) = load_yaml_at::<T>(&path, relative_path.clone(), report) {
            loaded.push(Loaded { path: relative_path, value });
        }
    }
    loaded
}

fn load_yaml_file<T: DeserializeOwned>(root: &Path, relative: &Path, report: &mut Diagnostics) -> Option<Loaded<T>> {
    let value = load_yaml_at::<T>(&root.join(relative), relative.to_path_buf(), report)?;
    Some(Loaded {
        path: relative.to_path_buf(),
        value,
    })
}

fn load_yaml_at<T: DeserializeOwned>(path: &Path, relative: PathBuf, report: &mut Diagnostics) -> Option<T> {
    let source = match fs::read_to_string(path) {
        Ok(source) => source,
        Err(error) => {
            push(report, ValidationLayer::Schema, relative, None, None, format!("file cannot be read: {error}"));
            return None;
        }
    };

    let value = match serde_yaml::from_str::<serde_yaml::Value>(&source) {
        Ok(value) => value,
        Err(error) => {
            push(
                report,
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
            push(
                report,
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

fn load_contexts(root: &Path, report: &mut Diagnostics) -> Vec<ContextDocument> {
    let directory = root.join("entity_context");
    if !directory.exists() {
        return Vec::new();
    }
    let entries = match fs::read_dir(&directory) {
        Ok(entries) => entries,
        Err(error) => {
            push(
                report,
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
            push(
                report,
                ValidationLayer::Schema,
                relative,
                None,
                None,
                "unexpected entry; entity_context may contain only .md files",
            );
            continue;
        }
        let Some(stem) = path.file_stem().and_then(|value| value.to_str()) else {
            push(report, ValidationLayer::Schema, relative, None, None, "context filename is not valid UTF-8");
            continue;
        };
        let entity_id = match serde_yaml::from_value::<EntityId>(stem.into()) {
            Ok(identifier) => identifier,
            Err(_) => {
                push(
                    report,
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
            Err(error) => push(
                report,
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

fn build_index<'a, T, I, F>(items: &'a [Loaded<T>], identifier: F, kind: &str, report: &mut Diagnostics) -> BTreeMap<I, &'a Loaded<T>>
where
    I: Clone + Ord + ToString,
    F: Fn(&T) -> I,
{
    let mut index = BTreeMap::new();
    for item in items {
        let id = identifier(&item.value);
        if let Some(previous) = index.insert(id.clone(), item) {
            push(
                report,
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

fn validate_filenames<T, F>(items: &[Loaded<T>], kind: &str, identifier: F, report: &mut Diagnostics)
where
    F: Fn(&T) -> &str,
{
    for item in items {
        let stem = item.path.file_stem().and_then(|value| value.to_str());
        let id = identifier(&item.value);
        if stem != Some(id) {
            push(
                report,
                ValidationLayer::Schema,
                item.path.clone(),
                None,
                Some(id.to_owned()),
                format!("{kind} filename must be exactly {id}.yaml"),
            );
        }
    }
}

fn validate_entity_types(items: &[Loaded<EntityType>], references: &BTreeMap<ReferenceId, &Loaded<Reference>>, report: &mut Diagnostics) {
    for item in items {
        let id = item.value.id.to_string();
        validate_localized_map(&item.path, &id, "labels", &item.value.labels, true, references, report);
        validate_localized_map(&item.path, &id, "descriptions", &item.value.descriptions, false, references, report);
    }
}

fn validate_properties(
    items: &[Loaded<Property>],
    types: &BTreeMap<EntityTypeId, &Loaded<EntityType>>,
    properties: &BTreeMap<PropertyId, &Loaded<Property>>,
    references: &BTreeMap<ReferenceId, &Loaded<Reference>>,
    report: &mut Diagnostics,
) {
    for item in items {
        let property = &item.value;
        let id = property.id.to_string();
        validate_localized_map(&item.path, &id, "labels", &property.labels, true, references, report);
        validate_localized_map(&item.path, &id, "descriptions", &property.descriptions, false, references, report);

        if property.subject_types.is_empty() {
            schema(report, item, &id, "subject_types must not be empty");
        }
        for type_id in &property.subject_types {
            if !types.contains_key(type_id) {
                ontology(report, item, &id, format!("subject type {type_id} does not exist"));
            }
        }

        match (&property.value_type, &property.target_types) {
            (ValueType::Entity, Some(targets)) if targets.is_empty() => {
                schema(report, item, &id, "target_types must not be empty");
            }
            (ValueType::Entity, None) => {
                schema(report, item, &id, "entity-valued property requires target_types");
            }
            (ValueType::Entity, Some(_)) | (_, None) => {}
            (_, Some(_)) => schema(report, item, &id, "target_types is allowed only for entity-valued properties"),
        }
        if let Some(targets) = &property.target_types {
            for type_id in targets {
                if !types.contains_key(type_id) {
                    ontology(report, item, &id, format!("target type {type_id} does not exist"));
                }
            }
        }
        for qualifier in &property.allowed_qualifiers {
            if !properties.contains_key(qualifier) {
                ontology(report, item, &id, format!("allowed qualifier property {qualifier} does not exist"));
            }
        }
    }
}

fn validate_references(items: &[Loaded<Reference>], report: &mut Diagnostics) {
    for item in items {
        let reference = &item.value;
        let id = reference.id.to_string();
        validate_url(&item.path, &id, "url", &reference.url, report);
        if let Some(url) = &reference.archive_url {
            validate_url(&item.path, &id, "archive_url", url, report);
        }
        if DateTime::parse_from_rfc3339(&reference.retrieved_at).is_err() {
            schema(report, item, &id, "retrieved_at must be an RFC 3339 timestamp");
        }
    }
}

fn validate_entities(
    items: &[Loaded<Entity>],
    entities: &BTreeMap<EntityId, &Loaded<Entity>>,
    types: &BTreeMap<EntityTypeId, &Loaded<EntityType>>,
    properties: &BTreeMap<PropertyId, &Loaded<Property>>,
    references: &BTreeMap<ReferenceId, &Loaded<Reference>>,
    report: &mut Diagnostics,
) {
    for item in items {
        let entity = &item.value;
        let id = entity.id.to_string();
        validate_localized_map(&item.path, &id, "labels", &entity.labels, true, references, report);
        validate_localized_map(&item.path, &id, "descriptions", &entity.descriptions, false, references, report);

        if entity.entity_types.is_empty() {
            schema(report, item, &id, "entity_types must not be empty");
        }
        for classification in &entity.entity_types {
            if !types.contains_key(&classification.value) {
                ontology(report, item, &id, format!("classified entity type {} does not exist", classification.value));
            }
            validate_provenance(&item.path, &id, "classification", &classification.references, references, report);
        }
        for image in &entity.images {
            validate_url(&item.path, &id, "image url", &image.url, report);
            if let Some(url) = &image.attribution_url {
                validate_url(&item.path, &id, "image attribution_url", url, report);
            }
            if image.attribution.trim().is_empty() {
                schema(report, item, &id, "image attribution must not be empty");
            }
            validate_provenance(&item.path, &id, "image", &image.references, references, report);
        }

        let mut statement_ids = BTreeSet::new();
        let mut property_counts = BTreeMap::<PropertyId, usize>::new();
        for statement in &entity.statements {
            if !statement_ids.insert(statement.id.clone()) {
                ontology(report, item, &id, format!("statement identifier {} is duplicated", statement.id));
            }
            *property_counts.entry(statement.property.clone()).or_default() += 1;
            validate_provenance(&item.path, &format!("{id}/{}", statement.id), "statement", &statement.references, references, report);
            validate_value(&item.path, &format!("{id}/{}", statement.id), &statement.value, report);

            let main_property = properties.get(&statement.property).map(|item| &item.value);
            if let Some(main_property) = main_property {
                validate_property_use(item, entity, &format!("{id}/{}", statement.id), main_property, &statement.value, entities, report);
            } else {
                ontology(report, item, &id, format!("statement property {} does not exist", statement.property));
            }

            for qualifier in &statement.qualifiers {
                validate_value(&item.path, &format!("{id}/{}/{}", statement.id, qualifier.property), &qualifier.value, report);
                if main_property.is_some_and(|property| !property.allowed_qualifiers.contains(&qualifier.property)) {
                    ontology(
                        report,
                        item,
                        &id,
                        format!(
                            "qualifier {} is not allowed by property {}",
                            qualifier.property,
                            main_property.expect("checked as present").id
                        ),
                    );
                }
                let Some(qualifier_property) = properties.get(&qualifier.property).map(|item| &item.value) else {
                    ontology(report, item, &id, format!("qualifier property {} does not exist", qualifier.property));
                    continue;
                };
                validate_property_use(
                    item,
                    entity,
                    &format!("{id}/{}/{}", statement.id, qualifier.property),
                    qualifier_property,
                    &qualifier.value,
                    entities,
                    report,
                );
            }
        }
        for (property_id, count) in property_counts {
            if count > 1 && properties.get(&property_id).is_some_and(|property| property.value.cardinality == Cardinality::One) {
                ontology(report, item, &id, format!("property {property_id} has cardinality one but occurs {count} times"));
            }
        }
    }
}

fn validate_property_use(
    item: &Loaded<Entity>,
    entity: &Entity,
    owner: &str,
    property: &Property,
    value: &Value,
    entities: &BTreeMap<EntityId, &Loaded<Entity>>,
    report: &mut Diagnostics,
) {
    let entity_types = entity.entity_types.iter().map(|classification| &classification.value).collect::<BTreeSet<_>>();
    if !property.subject_types.iter().any(|subject_type| entity_types.contains(subject_type)) {
        ontology(report, item, owner, format!("property {} is not applicable to this entity", property.id));
    }
    if value.value_type() != property.value_type {
        ontology(
            report,
            item,
            owner,
            format!(
                "property {} requires {} values but statement uses {}",
                property.id,
                value_type_name(property.value_type),
                value_type_name(value.value_type())
            ),
        );
        return;
    }
    if let Value::Entity { value: target_id } = value {
        let Some(target) = entities.get(target_id) else {
            ontology(report, item, owner, format!("target entity {target_id} does not exist"));
            return;
        };
        let target_types = target.value.entity_types.iter().map(|classification| &classification.value).collect::<BTreeSet<_>>();
        if !property
            .target_types
            .as_ref()
            .is_some_and(|allowed| allowed.iter().any(|type_id| target_types.contains(type_id)))
        {
            ontology(
                report,
                item,
                owner,
                format!("target entity {target_id} has none of property {}'s permitted target types", property.id),
            );
        }
    }
}

fn validate_value(path: &Path, owner: &str, value: &Value, report: &mut Diagnostics) {
    let message = match value {
        Value::Decimal { value } if !DECIMAL.is_match(value) => Some("decimal value must use canonical quoted base-10 syntax"),
        Value::Date { value } if !DATE.is_match(value) || NaiveDate::parse_from_str(value, "%Y-%m-%d").is_err() => Some("date value must be a real ISO 8601 calendar date"),
        Value::Datetime { value } if DateTime::parse_from_rfc3339(value).is_err() => Some("datetime value must be an RFC 3339 timestamp"),
        Value::Url { value } if Url::parse(value).is_err() => Some("url value must be an absolute URL"),
        Value::Coordinate { latitude, longitude } => {
            if !DECIMAL.is_match(latitude) || !within_absolute_bound(latitude, 90) {
                Some("coordinate latitude must be canonical decimal text between -90 and 90")
            } else if !DECIMAL.is_match(longitude) || !within_absolute_bound(longitude, 180) {
                Some("coordinate longitude must be canonical decimal text between -180 and 180")
            } else {
                None
            }
        }
        _ => None,
    };
    if let Some(message) = message {
        push(report, ValidationLayer::Schema, path.to_path_buf(), None, Some(owner.to_owned()), message);
    }
}

fn within_absolute_bound(value: &str, bound: u64) -> bool {
    let unsigned = value.strip_prefix('-').unwrap_or(value);
    let (integer, fraction) = unsigned.split_once('.').unwrap_or((unsigned, ""));
    match integer.len().cmp(&bound.to_string().len()) {
        std::cmp::Ordering::Less => true,
        std::cmp::Ordering::Greater => false,
        std::cmp::Ordering::Equal => match integer.parse::<u64>() {
            Ok(integer) if integer < bound => true,
            Ok(integer) if integer == bound => fraction.bytes().all(|byte| byte == b'0'),
            _ => false,
        },
    }
}

fn value_type_name(value_type: ValueType) -> &'static str {
    match value_type {
        ValueType::Entity => "entity",
        ValueType::String => "string",
        ValueType::Integer => "integer",
        ValueType::Decimal => "decimal",
        ValueType::Boolean => "boolean",
        ValueType::Date => "date",
        ValueType::Datetime => "datetime",
        ValueType::Url => "url",
        ValueType::Coordinate => "coordinate",
    }
}

fn validate_localized_map(
    path: &Path,
    owner: &str,
    field: &str,
    values: &LocalizedMap,
    required: bool,
    references: &BTreeMap<ReferenceId, &Loaded<Reference>>,
    report: &mut Diagnostics,
) {
    if required && values.is_empty() {
        push(
            report,
            ValidationLayer::Schema,
            path.to_path_buf(),
            None,
            Some(owner.to_owned()),
            format!("{field} must not be empty"),
        );
    }
    let mut normalized = BTreeSet::new();
    for (locale, value) in values {
        if locale.parse::<LanguageTag>().is_err() {
            push(
                report,
                ValidationLayer::Schema,
                path.to_path_buf(),
                None,
                Some(owner.to_owned()),
                format!("{field} locale {locale:?} is not a well-formed BCP 47 tag"),
            );
        }
        if !normalized.insert(locale.to_ascii_lowercase()) {
            push(
                report,
                ValidationLayer::Schema,
                path.to_path_buf(),
                None,
                Some(owner.to_owned()),
                format!("{field} contains locale {locale:?} more than once ignoring case"),
            );
        }
        validate_provenance(path, owner, &format!("{field}.{locale}"), &value.references, references, report);
    }
}

fn validate_provenance(path: &Path, owner: &str, field: &str, reference_ids: &[ReferenceId], references: &BTreeMap<ReferenceId, &Loaded<Reference>>, report: &mut Diagnostics) {
    if reference_ids.is_empty() {
        push(
            report,
            ValidationLayer::Schema,
            path.to_path_buf(),
            None,
            Some(owner.to_owned()),
            format!("{field} references must not be empty"),
        );
    }
    for reference_id in reference_ids {
        if !references.contains_key(reference_id) {
            push(
                report,
                ValidationLayer::Provenance,
                path.to_path_buf(),
                None,
                Some(owner.to_owned()),
                format!("{field} cites missing reference {reference_id}"),
            );
        }
    }
}

fn validate_url(path: &Path, owner: &str, field: &str, value: &str, report: &mut Diagnostics) {
    if Url::parse(value).is_err() {
        push(
            report,
            ValidationLayer::Schema,
            path.to_path_buf(),
            None,
            Some(owner.to_owned()),
            format!("{field} must be an absolute URL"),
        );
    }
}

fn validate_allocation(
    allocation: &Loaded<IdAllocation>,
    max_entity: Option<u64>,
    max_property: Option<u64>,
    max_reference: Option<u64>,
    max_type: Option<u64>,
    report: &mut Diagnostics,
) {
    if allocation.value.version != 1 {
        schema(report, allocation, "id_allocation", "version must be 1");
    }
    for (field, next, maximum) in [
        ("entity", allocation.value.next.entity, max_entity),
        ("property", allocation.value.next.property, max_property),
        ("reference", allocation.value.next.reference, max_reference),
        ("entity_type", allocation.value.next.entity_type, max_type),
    ] {
        if next == 0 {
            schema(report, allocation, "id_allocation", format!("next.{field} must be positive"));
        } else if maximum.is_some_and(|maximum| next <= maximum) {
            schema(
                report,
                allocation,
                "id_allocation",
                format!("next.{field} must be greater than the greatest used identifier number ({})", maximum.unwrap_or_default()),
            );
        }
    }
}

#[derive(Default)]
struct FootnoteDefinition {
    line: usize,
    targets: Vec<String>,
}

fn validate_contexts(
    contexts: &[ContextDocument],
    entities: &BTreeMap<EntityId, &Loaded<Entity>>,
    references: &BTreeMap<ReferenceId, &Loaded<Reference>>,
    report: &mut Diagnostics,
) {
    for context in contexts {
        let owner = context.entity_id.to_string();
        if !entities.contains_key(&context.entity_id) {
            push(
                report,
                ValidationLayer::Provenance,
                context.path.clone(),
                None,
                Some(owner.clone()),
                "context document names an entity that does not exist",
            );
        }

        let mut definitions = BTreeMap::<String, Vec<FootnoteDefinition>>::new();
        let mut references_used = BTreeMap::<String, usize>::new();
        let mut current_definition: Option<(String, FootnoteDefinition)> = None;
        let options = Options::ENABLE_FOOTNOTES;
        for (event, range) in Parser::new_ext(&context.source, options).into_offset_iter() {
            let line = line_at(&context.source, range.start);
            match event {
                Event::Start(Tag::FootnoteDefinition(label)) => {
                    current_definition = Some((label.to_string(), FootnoteDefinition { line, targets: Vec::new() }));
                }
                Event::End(pulldown_cmark::TagEnd::FootnoteDefinition) => {
                    if let Some((label, definition)) = current_definition.take() {
                        definitions.entry(label).or_default().push(definition);
                    }
                }
                Event::Start(Tag::Link { dest_url, .. }) => {
                    if let Some((_, definition)) = current_definition.as_mut() {
                        definition.targets.push(dest_url.to_string());
                    }
                }
                Event::FootnoteReference(label) => {
                    references_used.entry(label.to_string()).or_insert(line);
                }
                _ => {}
            }
        }

        for (label, line) in &references_used {
            parse_reference_label(label, &context.path, *line, &owner, references, report);
            match definitions.get(label).map(Vec::len).unwrap_or_default() {
                0 => push(
                    report,
                    ValidationLayer::Provenance,
                    context.path.clone(),
                    Some(*line),
                    Some(owner.clone()),
                    format!("footnote {label:?} has no definition"),
                ),
                1 => {}
                count => push(
                    report,
                    ValidationLayer::Provenance,
                    context.path.clone(),
                    Some(*line),
                    Some(owner.clone()),
                    format!("footnote {label:?} has {count} definitions"),
                ),
            }
        }

        for (label, entries) in definitions {
            if entries.len() > 1 && !references_used.contains_key(&label) {
                push(
                    report,
                    ValidationLayer::Provenance,
                    context.path.clone(),
                    Some(entries[0].line),
                    Some(owner.clone()),
                    format!("footnote {label:?} has {} definitions", entries.len()),
                );
            }
            for definition in &entries {
                let reference_id = parse_reference_label(&label, &context.path, definition.line, &owner, references, report);
                if let Some(reference_id) = reference_id {
                    let expected = format!("../references/{reference_id}.yaml");
                    if definition.targets.as_slice() != [expected.as_str()] {
                        push(
                            report,
                            ValidationLayer::Provenance,
                            context.path.clone(),
                            Some(definition.line),
                            Some(owner.clone()),
                            format!("footnote {label:?} must contain exactly one link to {expected}"),
                        );
                    }
                }
            }
        }
    }
}

fn parse_reference_label(
    label: &str,
    path: &Path,
    line: usize,
    owner: &str,
    references: &BTreeMap<ReferenceId, &Loaded<Reference>>,
    report: &mut Diagnostics,
) -> Option<ReferenceId> {
    let reference_id = match serde_yaml::from_value::<ReferenceId>(label.into()) {
        Ok(reference_id) => reference_id,
        Err(_) => {
            push(
                report,
                ValidationLayer::Provenance,
                path.to_path_buf(),
                Some(line),
                Some(owner.to_owned()),
                format!("footnote label {label:?} is not a canonical reference identifier"),
            );
            return None;
        }
    };
    if !references.contains_key(&reference_id) {
        push(
            report,
            ValidationLayer::Provenance,
            path.to_path_buf(),
            Some(line),
            Some(owner.to_owned()),
            format!("footnote {label:?} cites a reference that does not exist"),
        );
    }
    Some(reference_id)
}

fn schema<T>(report: &mut Diagnostics, item: &Loaded<T>, identifier: &str, message: impl Into<String>) {
    push(report, ValidationLayer::Schema, item.path.clone(), None, Some(identifier.to_owned()), message);
}

fn ontology<T>(report: &mut Diagnostics, item: &Loaded<T>, identifier: &str, message: impl Into<String>) {
    push(report, ValidationLayer::Ontology, item.path.clone(), None, Some(identifier.to_owned()), message);
}

fn push(report: &mut Diagnostics, layer: ValidationLayer, path: PathBuf, line: Option<usize>, identifier: Option<String>, message: impl Into<String>) {
    report.push(Diagnostic {
        layer,
        path,
        line,
        identifier,
        message: message.into(),
    });
}

fn sort_diagnostics(diagnostics: &mut [Diagnostic]) {
    diagnostics.sort_by(|left, right| {
        (&left.path, left.line.unwrap_or(usize::MAX), &left.identifier, &left.message, left.layer).cmp(&(
            &right.path,
            right.line.unwrap_or(usize::MAX),
            &right.identifier,
            &right.message,
            right.layer,
        ))
    });
}

fn relative_path(root: &Path, path: &Path) -> PathBuf {
    path.strip_prefix(root).map(Path::to_path_buf).unwrap_or_else(|_| path.to_path_buf())
}

fn line_at(source: &str, offset: usize) -> usize {
    source[..offset.min(source.len())].bytes().filter(|byte| *byte == b'\n').count() + 1
}
