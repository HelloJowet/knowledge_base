use super::Validator;
use super::values::{validate_localized_map, validate_optional_provenance, validate_provenance, validate_url, validate_value, value_type_name};
use crate::diagnostic::Diagnostics;
use crate::input::Loaded;
use knowledge_base_models::{Cardinality, Entity, EntityId, Property, PropertyId, Value};
use std::collections::{BTreeMap, BTreeSet};

pub(super) fn validate(validator: &mut Validator<'_>) {
    let entities = &validator.indexes.entities;
    let types = &validator.indexes.entity_types;
    let properties = &validator.indexes.properties;
    let references = &validator.indexes.references;
    let diagnostics = &mut validator.diagnostics;

    for item in &validator.repository.entities {
        let entity = &item.value;
        let id = entity.id.to_string();
        validate_localized_map(&item.path, &id, "labels", &entity.labels, true, true, references, diagnostics);
        validate_localized_map(&item.path, &id, "descriptions", &entity.descriptions, false, true, references, diagnostics);

        if entity.entity_types.is_empty() {
            diagnostics.schema(item, &id, "entity_types must not be empty");
        }
        for classification in &entity.entity_types {
            if !types.contains_key(&classification.value) {
                diagnostics.ontology(item, &id, format!("classified entity type {} does not exist", classification.value));
            }
            validate_provenance(&item.path, &id, "classification", &classification.references, references, diagnostics);
        }
        for image in &entity.images {
            validate_url(&item.path, &id, "image url", &image.url, diagnostics);
            validate_url(&item.path, &id, "image source_url", &image.source_url, diagnostics);
            for (field, value) in [("alt", &image.alt), ("creator", &image.creator), ("license", &image.license)] {
                if value.trim().is_empty() {
                    diagnostics.schema(item, &id, format!("image {field} must not be empty"));
                }
            }
            validate_optional_provenance(&item.path, &id, "image", &image.references, references, diagnostics);
        }

        let mut statement_ids = BTreeSet::new();
        let mut property_counts = BTreeMap::<PropertyId, usize>::new();
        for statement in &entity.statements {
            if !statement_ids.insert(statement.id.clone()) {
                diagnostics.ontology(item, &id, format!("statement identifier {} is duplicated", statement.id));
            }
            *property_counts.entry(statement.property.clone()).or_default() += 1;
            validate_provenance(&item.path, &format!("{id}/{}", statement.id), "statement", &statement.references, references, diagnostics);
            validate_value(&item.path, &format!("{id}/{}", statement.id), &statement.value, diagnostics);

            let main_property = properties.get(&statement.property).map(|item| &item.value);
            if let Some(main_property) = main_property {
                if !main_property.usage.allows_statement() {
                    diagnostics.ontology(
                        item,
                        &format!("{id}/{}", statement.id),
                        format!("property {} cannot be used as a statement", main_property.id),
                    );
                }
                validate_property_use(item, entity, &format!("{id}/{}", statement.id), main_property, &statement.value, entities, diagnostics);
            } else {
                diagnostics.ontology(item, &id, format!("statement property {} does not exist", statement.property));
            }

            for qualifier in &statement.qualifiers {
                validate_value(&item.path, &format!("{id}/{}/{}", statement.id, qualifier.property), &qualifier.value, diagnostics);
                if main_property.is_some_and(|property| !property.allowed_qualifiers.contains(&qualifier.property)) {
                    diagnostics.ontology(
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
                    diagnostics.ontology(item, &id, format!("qualifier property {} does not exist", qualifier.property));
                    continue;
                };
                if !qualifier_property.usage.allows_qualifier() {
                    diagnostics.ontology(
                        item,
                        &format!("{id}/{}", statement.id),
                        format!("property {} cannot be used as a qualifier", qualifier_property.id),
                    );
                }
                validate_property_use(
                    item,
                    entity,
                    &format!("{id}/{}/{}", statement.id, qualifier.property),
                    qualifier_property,
                    &qualifier.value,
                    entities,
                    diagnostics,
                );
            }
        }
        for (property_id, count) in property_counts {
            if count > 1 && properties.get(&property_id).is_some_and(|property| property.value.cardinality == Cardinality::One) {
                diagnostics.ontology(item, &id, format!("property {property_id} has cardinality one but occurs {count} times"));
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
    diagnostics: &mut Diagnostics,
) {
    let entity_types = entity.entity_types.iter().map(|classification| &classification.value).collect::<BTreeSet<_>>();
    if !property.subject_types.iter().any(|subject_type| entity_types.contains(subject_type)) {
        diagnostics.ontology(item, owner, format!("property {} is not applicable to this entity", property.id));
    }
    if value.value_type() != property.value_type {
        diagnostics.ontology(
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
            diagnostics.ontology(item, owner, format!("target entity {target_id} does not exist"));
            return;
        };
        let target_types = target.value.entity_types.iter().map(|classification| &classification.value).collect::<BTreeSet<_>>();
        if !property
            .target_types
            .as_ref()
            .is_some_and(|allowed| allowed.iter().any(|type_id| target_types.contains(type_id)))
        {
            diagnostics.ontology(
                item,
                owner,
                format!("target entity {target_id} has none of property {}'s permitted target types", property.id),
            );
        }
    }
}
