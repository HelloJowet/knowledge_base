use super::Validator;
use super::values::{valid_partial_date, validate_localized_map, validate_nonempty_metadata, validate_url};
use crate::diagnostic::Diagnostics;
use crate::input::Loaded;
use chrono::DateTime;
use knowledge_base_models::{EntityId, EntityType, EntityTypeId, IdAllocation, PropertyId, Reference, ReferenceId, ValueType};
use language_tags::LanguageTag;
use std::collections::BTreeMap;

pub(super) fn validate(validator: &mut Validator<'_>) {
    validate_entity_types(&validator.repository.entity_types, &validator.indexes.references, &mut validator.diagnostics);
    validate_properties(validator);
    validate_references(&validator.repository.references, &mut validator.diagnostics);

    if let Some(allocation) = validator.repository.allocation.as_ref() {
        validate_allocation(
            allocation,
            validator.indexes.entities.keys().map(EntityId::number).max(),
            validator.indexes.properties.keys().map(PropertyId::number).max(),
            validator.indexes.references.keys().map(ReferenceId::number).max(),
            validator.indexes.entity_types.keys().map(EntityTypeId::number).max(),
            &mut validator.diagnostics,
        );
    }
}

fn validate_entity_types(items: &[Loaded<EntityType>], references: &BTreeMap<ReferenceId, &Loaded<Reference>>, diagnostics: &mut Diagnostics) {
    for item in items {
        let id = item.value.id.to_string();
        validate_localized_map(&item.path, &id, "labels", &item.value.labels, true, references, diagnostics);
        validate_localized_map(&item.path, &id, "descriptions", &item.value.descriptions, false, references, diagnostics);
    }
}

fn validate_properties(validator: &mut Validator<'_>) {
    let references = &validator.indexes.references;
    let types = &validator.indexes.entity_types;
    let properties = &validator.indexes.properties;
    let diagnostics = &mut validator.diagnostics;

    for item in &validator.repository.properties {
        let property = &item.value;
        let id = property.id.to_string();
        validate_localized_map(&item.path, &id, "labels", &property.labels, true, references, diagnostics);
        validate_localized_map(&item.path, &id, "descriptions", &property.descriptions, false, references, diagnostics);

        if property.subject_types.is_empty() {
            diagnostics.schema(item, &id, "subject_types must not be empty");
        }
        for type_id in &property.subject_types {
            if !types.contains_key(type_id) {
                diagnostics.ontology(item, &id, format!("subject type {type_id} does not exist"));
            }
        }

        match (&property.value_type, &property.target_types) {
            (ValueType::Entity, Some(targets)) if targets.is_empty() => {
                diagnostics.schema(item, &id, "target_types must not be empty");
            }
            (ValueType::Entity, None) => {
                diagnostics.schema(item, &id, "entity-valued property requires target_types");
            }
            (ValueType::Entity, Some(_)) | (_, None) => {}
            (_, Some(_)) => diagnostics.schema(item, &id, "target_types is allowed only for entity-valued properties"),
        }
        if let Some(targets) = &property.target_types {
            for type_id in targets {
                if !types.contains_key(type_id) {
                    diagnostics.ontology(item, &id, format!("target type {type_id} does not exist"));
                }
            }
        }
        for qualifier in &property.allowed_qualifiers {
            if !properties.contains_key(qualifier) {
                diagnostics.ontology(item, &id, format!("allowed qualifier property {qualifier} does not exist"));
            }
        }
    }
}

fn validate_references(items: &[Loaded<Reference>], diagnostics: &mut Diagnostics) {
    for item in items {
        let reference = &item.value;
        let id = reference.id.to_string();
        validate_url(&item.path, &id, "url", &reference.url, diagnostics);
        validate_nonempty_metadata(item, &id, "title", &reference.title, diagnostics);
        if let Some(publisher) = &reference.publisher {
            validate_nonempty_metadata(item, &id, "publisher", publisher, diagnostics);
        }
        if let Some(publication_date) = &reference.publication_date
            && validate_nonempty_metadata(item, &id, "publication_date", publication_date, diagnostics)
            && !valid_partial_date(publication_date)
        {
            diagnostics.schema(item, &id, "publication_date must be a valid YYYY, YYYY-MM, or YYYY-MM-DD date");
        }
        if let Some(source_language) = &reference.source_language
            && validate_nonempty_metadata(item, &id, "source_language", source_language, diagnostics)
            && source_language.parse::<LanguageTag>().is_err()
        {
            diagnostics.schema(item, &id, "source_language must be a well-formed BCP 47 tag");
        }
        if let Some(url) = &reference.archive_url {
            validate_url(&item.path, &id, "archive_url", url, diagnostics);
        }
        if DateTime::parse_from_rfc3339(&reference.retrieved_at).is_err() {
            diagnostics.schema(item, &id, "retrieved_at must be an RFC 3339 timestamp");
        }
    }
}

fn validate_allocation(
    allocation: &Loaded<IdAllocation>,
    max_entity: Option<u64>,
    max_property: Option<u64>,
    max_reference: Option<u64>,
    max_type: Option<u64>,
    diagnostics: &mut Diagnostics,
) {
    if allocation.value.version != 1 {
        diagnostics.schema(allocation, "id_allocation", "version must be 1");
    }
    for (field, next, maximum) in [
        ("entity", allocation.value.next.entity, max_entity),
        ("property", allocation.value.next.property, max_property),
        ("reference", allocation.value.next.reference, max_reference),
        ("entity_type", allocation.value.next.entity_type, max_type),
    ] {
        if next == 0 {
            diagnostics.schema(allocation, "id_allocation", format!("next.{field} must be positive"));
        } else if maximum.is_some_and(|maximum| next <= maximum) {
            diagnostics.schema(
                allocation,
                "id_allocation",
                format!("next.{field} must be greater than the greatest used identifier number ({})", maximum.unwrap_or_default()),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::validate_references;
    use crate::diagnostic::Diagnostics;
    use crate::input::Loaded;
    use knowledge_base_models::Reference;
    use std::path::PathBuf;

    #[test]
    fn validates_reference_metadata() {
        let references = [Loaded {
            path: PathBuf::from("references/R1.yaml"),
            value: Reference {
                id: "R1".parse().expect("valid reference identifier"),
                url: "https://example.org".to_owned(),
                title: " ".to_owned(),
                publisher: Some("".to_owned()),
                publication_date: Some("2025-02-29".to_owned()),
                source_language: Some("en_US".to_owned()),
                retrieved_at: "2025-01-15T10:30:00Z".to_owned(),
                archive_url: None,
            },
        }];
        let mut diagnostics = Diagnostics::default();

        validate_references(&references, &mut diagnostics);

        let diagnostics = diagnostics.finish();
        let messages = diagnostics.iter().map(|item| item.message.as_str()).collect::<Vec<_>>();
        assert!(messages.contains(&"title must not be empty"));
        assert!(messages.contains(&"publisher must not be empty"));
        assert!(messages.contains(&"publication_date must be a valid YYYY, YYYY-MM, or YYYY-MM-DD date"));
        assert!(messages.contains(&"source_language must be a well-formed BCP 47 tag"));
    }
}
