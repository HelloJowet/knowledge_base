use std::{collections::HashSet, path::Path};

use knowledge_base_snapshot::RepositorySnapshot;

use super::{
    candidate,
    diagnostics::{DiagnosticFactory, ValidationReport},
    index::{InventoryIndex, has_entity, has_entity_type, has_property, has_reference},
    rules::{candidate_id, draft_id, evidence_id, is_identifier, require_unique, validate_text_list},
    summary,
};
use crate::IngestionCandidateInventory;

pub(crate) fn validate(path: &Path, inventory: &IngestionCandidateInventory, snapshot: &RepositorySnapshot, report: &mut ValidationReport) {
    if inventory.source_file != "page.html" {
        report.push(DiagnosticFactory::domain(path, "source_file", "must be page.html"));
    }
    let source_path = path.parent().unwrap_or_else(|| Path::new(".")).join(&inventory.source_file);
    if !source_path.is_file() {
        report.push(DiagnosticFactory::at_path(source_path, "source HTML file does not exist"));
    }
    if !is_identifier(&inventory.source_reference, "R") || !has_reference(snapshot, &inventory.source_reference) {
        report.push(DiagnosticFactory::domain(
            path,
            "source_reference",
            format!("unresolved production reference {}", inventory.source_reference),
        ));
    }
    validate_text_list(report, path, "review_notes", &inventory.review_notes);
    validate_coverage(path, inventory, report);
    validate_evidence(path, inventory, report);
    validate_drafts(path, inventory, snapshot, report);
    let index = InventoryIndex::new(inventory);
    for (candidate_index, candidate_value) in inventory.candidates.iter().enumerate() {
        if !candidate_id(&candidate_value.key) {
            report.push(DiagnosticFactory::domain(
                path,
                format!("candidates[{candidate_index}].key"),
                format!("invalid candidate ID {}", candidate_value.key),
            ));
        }
        candidate::validate(candidate_index, candidate_value, path, snapshot, &index, report);
    }
    let mut results = HashSet::new();
    for (result_index, result) in inventory.article_results.iter().enumerate() {
        let context = format!("article_results[{result_index}].production_entity");
        if !has_entity(snapshot, &result.production_entity) {
            report.push(DiagnosticFactory::domain(
                path,
                &context,
                format!("unresolved production entity {}", result.production_entity),
            ));
        }
        if !results.insert(result.production_entity.as_str()) {
            report.push(DiagnosticFactory::domain(path, &context, format!("duplicate result for {}", result.production_entity)));
        }
    }
    summary::validate(path, inventory, report);
}

fn validate_coverage(path: &Path, inventory: &IngestionCandidateInventory, report: &mut ValidationReport) {
    let Some(coverage) = &inventory.coverage else { return };
    for (index, reviewed) in coverage.reviewed.iter().enumerate() {
        for (field, value) in [("heading", &reviewed.heading), ("location", &reviewed.location), ("scope", &reviewed.scope)] {
            report.require_nonempty(path, &format!("coverage.reviewed[{index}].{field}"), value);
        }
    }
    for (index, excluded) in coverage.excluded.iter().enumerate() {
        for (field, value) in [("concept", &excluded.concept), ("location", &excluded.location), ("reason", &excluded.reason)] {
            report.require_nonempty(path, &format!("coverage.excluded[{index}].{field}"), value);
        }
    }
}

fn validate_evidence(path: &Path, inventory: &IngestionCandidateInventory, report: &mut ValidationReport) {
    let mut ids = HashSet::new();
    for (index, evidence) in inventory.evidence.iter().enumerate() {
        let context = format!("evidence[{index}]");
        if !evidence_id(&evidence.id) {
            report.push(DiagnosticFactory::domain(path, &context, format!("invalid evidence ID {}", evidence.id)));
        }
        if !ids.insert(evidence.id.as_str()) {
            report.push(DiagnosticFactory::domain(path, &context, format!("duplicate evidence ID {}", evidence.id)));
        }
        if evidence.reference != inventory.source_reference {
            report.push(DiagnosticFactory::domain(path, format!("{context}.reference"), "must match source_reference"));
        }
        for (field, value) in [("heading", &evidence.heading), ("location", &evidence.location), ("excerpt", &evidence.excerpt)] {
            report.require_nonempty(path, &format!("{context}.{field}"), value);
        }
    }
}

fn validate_drafts(path: &Path, inventory: &IngestionCandidateInventory, snapshot: &RepositorySnapshot, report: &mut ValidationReport) {
    let candidate_ids: HashSet<_> = inventory.candidates.iter().map(|item| item.key.as_str()).collect();
    let evidence_ids: HashSet<_> = inventory.evidence.iter().map(|item| item.id.as_str()).collect();
    let type_ids: HashSet<_> = inventory.draft_entity_types.iter().map(|item| item.id.as_str()).collect();
    let property_ids: HashSet<_> = inventory.draft_properties.iter().map(|item| item.id.as_str()).collect();
    if type_ids.len() != inventory.draft_entity_types.len() {
        report.push(DiagnosticFactory::domain(path, "draft_entity_types", "contains duplicate draft IDs"));
    }
    if property_ids.len() != inventory.draft_properties.len() {
        report.push(DiagnosticFactory::domain(path, "draft_properties", "contains duplicate draft IDs"));
    }
    for (index, draft) in inventory.draft_entity_types.iter().enumerate() {
        let context = format!("draft_entity_types[{index}]");
        if !draft_id(&draft.id, "DT") {
            report.push(DiagnosticFactory::domain(path, &context, format!("invalid draft type ID {}", draft.id)));
        }
        report.require_nonempty(path, &format!("{context}.label"), &draft.label);
        report.require_nonempty(path, &format!("{context}.description"), &draft.description);
        validate_links(report, path, &format!("{context}.evidence"), &draft.evidence, &evidence_ids);
        validate_links(report, path, &format!("{context}.affected_candidates"), &draft.affected_candidates, &candidate_ids);
        if let Some(id) = &draft.production_id
            && !has_entity_type(snapshot, id)
        {
            report.push(DiagnosticFactory::domain(path, format!("{context}.production_id"), format!("unresolved identifier {id}")));
        }
    }
    for (index, draft) in inventory.draft_properties.iter().enumerate() {
        let context = format!("draft_properties[{index}]");
        if !draft_id(&draft.id, "DP") {
            report.push(DiagnosticFactory::domain(path, &context, format!("invalid draft property ID {}", draft.id)));
        }
        report.require_nonempty(path, &format!("{context}.label"), &draft.label);
        report.require_nonempty(path, &format!("{context}.description"), &draft.description);
        validate_links(report, path, &format!("{context}.evidence"), &draft.evidence, &evidence_ids);
        validate_links(report, path, &format!("{context}.affected_candidates"), &draft.affected_candidates, &candidate_ids);
        validate_type_links(report, path, &format!("{context}.allowed_subject_types"), &draft.allowed_subject_types, snapshot, &type_ids);
        validate_type_links(report, path, &format!("{context}.allowed_value_types"), &draft.allowed_value_types, snapshot, &type_ids);
        for qualifier in &draft.allowed_qualifiers {
            if !has_property(snapshot, qualifier) && !property_ids.contains(qualifier.as_str()) {
                report.push(DiagnosticFactory::domain(
                    path,
                    format!("{context}.allowed_qualifiers"),
                    format!("unresolved property {qualifier}"),
                ));
            }
        }
        if let Some(id) = &draft.production_id
            && !has_property(snapshot, id)
        {
            report.push(DiagnosticFactory::domain(path, format!("{context}.production_id"), format!("unresolved identifier {id}")));
        }
    }
}

fn validate_type_links(report: &mut ValidationReport, path: &Path, context: &str, ids: &[String], snapshot: &RepositorySnapshot, drafts: &HashSet<&str>) {
    require_unique(report, path, context, ids);
    for id in ids {
        if !has_entity_type(snapshot, id) && !drafts.contains(id.as_str()) {
            report.push(DiagnosticFactory::domain(path, context, format!("unresolved entity type {id}")));
        }
    }
}
fn validate_links<T: AsRef<str> + std::fmt::Display + Eq + std::hash::Hash>(report: &mut ValidationReport, path: &Path, context: &str, ids: &[T], known: &HashSet<&str>) {
    require_unique(report, path, context, ids);
    for id in ids {
        if !known.contains(id.as_ref()) {
            report.push(DiagnosticFactory::domain(path, context, format!("unresolved identifier {id}")));
        }
    }
}
