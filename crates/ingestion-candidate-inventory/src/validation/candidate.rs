use std::{collections::BTreeSet, path::Path};

use knowledge_base_models::EntityId;
use knowledge_base_snapshot::RepositorySnapshot;

use super::{
    diagnostics::{DiagnosticFactory, ValidationReport},
    index::{InventoryIndex, has_entity, property},
    rules::{candidate_id, require_unique, validate_candidate_value},
};
use crate::{Candidate, CandidateValue, RecommendedOutcome};

pub(crate) fn validate(candidate_index: usize, candidate: &Candidate, path: &Path, snapshot: &RepositorySnapshot, index: &InventoryIndex<'_>, report: &mut ValidationReport) {
    let context = format!("candidates[{candidate_index}]");
    validate_links(report, path, &format!("{context}.evidence"), &candidate.evidence, &index.evidence_ids);
    super::rules::validate_text_list(report, path, &format!("{context}.source_names"), &candidate.source_names);
    super::rules::validate_text_list(report, path, &format!("{context}.unresolved_questions"), &candidate.unresolved_questions);
    require_unique(report, path, &format!("{context}.dependencies"), &candidate.dependencies);
    validate_type_links(
        report,
        path,
        &format!("{context}.proposed_metadata.classifications"),
        &candidate.proposed_metadata.classifications,
        snapshot,
        &index.draft_type_ids,
    );
    match candidate.recommended_outcome {
        RecommendedOutcome::New => {
            if candidate.existing_entity.is_some() {
                report.push(DiagnosticFactory::domain(
                    path,
                    format!("{context}.existing_entity"),
                    "new candidate cannot declare existing_entity",
                ));
            }
            require_option_text(report, path, &context, "label", candidate.proposed_metadata.label.as_deref());
            require_option_text(report, path, &context, "description", candidate.proposed_metadata.description.as_deref());
            if candidate.proposed_metadata.classifications.is_empty() {
                report.push(DiagnosticFactory::domain(path, &context, "new candidate requires a classification"));
            }
        }
        RecommendedOutcome::Existing if candidate.existing_entity.is_none() => {
            report.push(DiagnosticFactory::domain(path, &context, "existing candidate requires existing_entity"))
        }
        _ => {}
    }
    for (field, value) in [
        ("existing_entity", candidate.existing_entity.as_deref()),
        ("production_entity", candidate.production_entity.as_deref()),
    ] {
        if let Some(id) = value
            && !has_entity(snapshot, id)
        {
            report.push(DiagnosticFactory::domain(path, format!("{context}.{field}"), format!("unresolved production entity {id}")));
        }
    }

    let mut expected_dependencies = BTreeSet::new();
    for (statement_index, statement) in candidate.statements.iter().enumerate() {
        let statement_context = format!("{context}.statements[{statement_index}]");
        validate_links(report, path, &format!("{statement_context}.evidence"), &statement.evidence, &index.evidence_ids);
        validate_candidate_value(report, path, &format!("{statement_context}.value"), &statement.value);
        collect_candidate_target(&statement.value, &mut expected_dependencies);
        let Some(definition) = property(&statement.property, snapshot, &index.draft_properties) else {
            report.push(DiagnosticFactory::domain(
                path,
                format!("{statement_context}.property"),
                format!("unresolved property {}", statement.property),
            ));
            continue;
        };
        if !definition.usage.allows_statement() {
            report.push(DiagnosticFactory::domain(
                path,
                format!("{statement_context}.property"),
                "property cannot be used for statements",
            ));
        }
        if definition.value_type != statement.value.value_type() {
            report.push(DiagnosticFactory::domain(
                path,
                format!("{statement_context}.value"),
                format!("expected {:?} value", definition.value_type),
            ));
        }
        if !definition.allowed_subject_types.is_empty()
            && !candidate
                .proposed_metadata
                .classifications
                .iter()
                .any(|type_id| definition.allowed_subject_types.contains(&type_id.as_str()))
        {
            report.push(DiagnosticFactory::domain(
                path,
                format!("{statement_context}.property"),
                "property is incompatible with candidate classifications",
            ));
        }
        validate_target(
            report,
            path,
            &format!("{statement_context}.value"),
            &statement.value,
            &definition.allowed_value_types,
            snapshot,
            index,
        );
        for (qualifier_index, qualifier) in statement.qualifiers.iter().enumerate() {
            let qualifier_context = format!("{statement_context}.qualifiers[{qualifier_index}]");
            validate_candidate_value(report, path, &format!("{qualifier_context}.value"), &qualifier.value);
            collect_candidate_target(&qualifier.value, &mut expected_dependencies);
            if !definition.allowed_qualifiers.contains(&qualifier.property.as_str()) {
                report.push(DiagnosticFactory::domain(
                    path,
                    format!("{qualifier_context}.property"),
                    format!("{} is not an allowed qualifier", qualifier.property),
                ));
            }
            let Some(qualifier_definition) = property(&qualifier.property, snapshot, &index.draft_properties) else {
                report.push(DiagnosticFactory::domain(
                    path,
                    format!("{qualifier_context}.property"),
                    format!("unresolved property {}", qualifier.property),
                ));
                continue;
            };
            if !qualifier_definition.usage.allows_qualifier() {
                report.push(DiagnosticFactory::domain(
                    path,
                    format!("{qualifier_context}.property"),
                    "property cannot be used as a qualifier",
                ));
            }
            if qualifier_definition.value_type != qualifier.value.value_type() {
                report.push(DiagnosticFactory::domain(
                    path,
                    format!("{qualifier_context}.value"),
                    format!("expected {:?} value", qualifier_definition.value_type),
                ));
            }
            validate_target(
                report,
                path,
                &format!("{qualifier_context}.value"),
                &qualifier.value,
                &qualifier_definition.allowed_value_types,
                snapshot,
                index,
            );
        }
    }
    let actual: BTreeSet<_> = candidate.dependencies.iter().map(String::as_str).collect();
    if actual != expected_dependencies {
        report.push(DiagnosticFactory::domain(
            path,
            format!("{context}.dependencies"),
            format!("must exactly match candidate entity targets: {expected_dependencies:?}"),
        ));
    }
}

fn require_option_text(report: &mut ValidationReport, path: &Path, context: &str, field: &str, value: Option<&str>) {
    if value.is_none_or(|value| value.trim().is_empty()) {
        report.push(DiagnosticFactory::domain(path, context, format!("new candidate requires a non-empty {field}")));
    }
}
fn collect_candidate_target<'a>(value: &'a CandidateValue, dependencies: &mut BTreeSet<&'a str>) {
    if let Some(id) = value.entity_id().filter(|id| candidate_id(id)) {
        dependencies.insert(id);
    }
}
fn validate_target(
    report: &mut ValidationReport,
    path: &Path,
    context: &str,
    value: &CandidateValue,
    allowed_types: &[&str],
    snapshot: &RepositorySnapshot,
    index: &InventoryIndex<'_>,
) {
    let Some(id) = value.entity_id() else { return };
    let classifications: Option<Vec<&str>> = if let Some(candidate) = index.candidates.get(id) {
        Some(candidate.proposed_metadata.classifications.iter().map(String::as_str).collect())
    } else {
        id.parse::<EntityId>()
            .ok()
            .and_then(|id| snapshot.entities().get(&id))
            .map(|entity| entity.entity_types.iter().map(|type_id| type_id.value.as_str()).collect())
    };
    match classifications {
        None => report.push(DiagnosticFactory::domain(path, context, format!("unresolved entity target {id}"))),
        Some(classifications) if !allowed_types.is_empty() && !classifications.iter().any(|type_id| allowed_types.contains(type_id)) => {
            report.push(DiagnosticFactory::domain(path, context, format!("{id} has no allowed classification")))
        }
        _ => {}
    }
}
fn validate_type_links(report: &mut ValidationReport, path: &Path, context: &str, ids: &[String], snapshot: &RepositorySnapshot, draft_ids: &std::collections::HashSet<&str>) {
    require_unique(report, path, context, ids);
    for id in ids {
        if !super::index::has_entity_type(snapshot, id) && !draft_ids.contains(id.as_str()) {
            report.push(DiagnosticFactory::domain(path, context, format!("unresolved entity type {id}")));
        }
    }
}
fn validate_links<T: AsRef<str> + std::fmt::Display + Eq + std::hash::Hash>(
    report: &mut ValidationReport,
    path: &Path,
    context: &str,
    ids: &[T],
    known: &std::collections::HashSet<&str>,
) {
    require_unique(report, path, context, ids);
    for id in ids {
        if !known.contains(id.as_ref()) {
            report.push(DiagnosticFactory::domain(path, context, format!("unresolved identifier {id}")));
        }
    }
}
