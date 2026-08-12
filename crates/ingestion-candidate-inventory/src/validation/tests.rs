use std::{
    fs,
    path::{Path, PathBuf},
};

use knowledge_base_snapshot::RepositorySnapshot;
use tempfile::TempDir;

use super::validate_ingestion_candidate_inventory;

fn snapshot(root: &Path) -> RepositorySnapshot {
    for directory in ["entities", "entity_types", "properties", "references"] {
        fs::create_dir_all(root.join(directory)).unwrap();
    }
    fs::write(
        root.join("id_allocation.yaml"),
        "version: 1\nnext: {entity: 2, property: 1, reference: 2, entity_type: 2}\n",
    )
    .unwrap();
    fs::write(
        root.join("references/R1.yaml"),
        "id: R1\nurl: https://example.com\ntitle: Example\nretrieved_at: '2026-01-01T00:00:00Z'\n",
    )
    .unwrap();
    RepositorySnapshot::load(root).unwrap()
}

fn bundle() -> (TempDir, RepositorySnapshot, PathBuf) {
    let root = TempDir::new().unwrap();
    let snapshot = snapshot(&root.path().join("knowledge_base"));
    let bundle = root.path().join("bundle");
    fs::create_dir(&bundle).unwrap();
    fs::write(bundle.join("page.html"), "page").unwrap();
    let path = bundle.join("ingestion_candidate_inventory.yaml");
    (root, snapshot, path)
}

fn minimal() -> &'static str {
    "source_reference: R1\nsource_file: page.html\nevidence: []\ndraft_entity_types: []\ndraft_properties: []\narticle_results: []\ncandidates: []\n"
}

#[test]
fn accepts_a_minimal_strict_inventory() {
    let (_root, snapshot, path) = bundle();
    fs::write(&path, minimal()).unwrap();
    assert!(validate_ingestion_candidate_inventory(&path, &snapshot).is_empty());
}

#[test]
fn rejects_unknown_and_removed_fields() {
    let (_root, snapshot, path) = bundle();
    fs::write(&path, format!("{}unknown: true\n", minimal())).unwrap();
    assert!(validate_ingestion_candidate_inventory(&path, &snapshot)[0].message.contains("unknown field"));
    fs::write(
        &path,
        format!(
            "{}candidates: [{{key: C001, proposed_metadata: {{}}, article_context: [], evidence: [], recommended_outcome: omit}}]\n",
            minimal().replace("candidates: []\n", "")
        ),
    )
    .unwrap();
    assert!(validate_ingestion_candidate_inventory(&path, &snapshot)[0].message.contains("article_context"));
}

#[test]
fn validates_supplied_summary_and_statement_counts() {
    let (_root, snapshot, path) = bundle();
    fs::write(&path, "source_reference: R1\nsource_file: page.html\nevidence: []\ndraft_entity_types: []\ndraft_properties: []\narticle_results: []\ncandidates: [{key: C001, proposed_metadata: {}, evidence: [], recommended_outcome: omit, statements: [], dependencies: []}]\nstatement_counts: {total: 1, entity_valued: 0, candidates_with_statements: 0, by_property: {}}\nsummary: {candidate_count: 0, evidence_count: 0, draft_entity_type_count: 0, outcome_counts: {}}\n").unwrap();
    let diagnostics = validate_ingestion_candidate_inventory(&path, &snapshot);
    assert!(diagnostics.iter().any(|diagnostic| diagnostic.identifier.as_deref() == Some("statement_counts.total")));
    assert!(diagnostics.iter().any(|diagnostic| diagnostic.identifier.as_deref() == Some("summary.candidate_count")));
}

#[test]
fn accepts_a_candidate_using_a_compatible_draft_property() {
    let (_root, snapshot, path) = bundle();
    fs::write(&path, "source_reference: R1\nsource_file: page.html\nevidence: [{id: E001, reference: R1, heading: Overview, location: p1, excerpt: Example}]\ndraft_entity_types: [{id: DT1, label: Place, description: A place, evidence: [E001], affected_candidates: [C001]}]\ndraft_properties: [{id: DP1, label: Name, description: A name, value_type: string, allowed_subject_types: [DT1], allowed_value_types: [], allowed_qualifiers: [], usage: statement, evidence: [E001], affected_candidates: [C001]}]\narticle_results: []\ncandidates: [{key: C001, proposed_metadata: {label: Example, description: An example, classifications: [DT1]}, evidence: [E001], recommended_outcome: new, statements: [{property: DP1, value: {type: string, value: Example}, evidence: [E001]}], dependencies: []}]\n").unwrap();
    assert!(validate_ingestion_candidate_inventory(&path, &snapshot).is_empty());
}

#[test]
fn reports_empty_coverage_invalid_values_and_contradictory_outcomes() {
    let (_root, snapshot, path) = bundle();
    fs::write(&path, "source_reference: R1\nsource_file: page.html\ncoverage: {reviewed: [{heading: '', location: p1, scope: all}]}\nevidence: []\ndraft_entity_types: []\ndraft_properties: []\narticle_results: []\ncandidates: [{key: C001, proposed_metadata: {label: Example, description: Example, classifications: [DT1]}, evidence: [], recommended_outcome: new, existing_entity: Q1, statements: [{property: P1, value: {type: url, value: http://example.com}, evidence: []}], dependencies: []}]\n").unwrap();
    let diagnostics = validate_ingestion_candidate_inventory(&path, &snapshot);
    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| diagnostic.identifier.as_deref() == Some("coverage.reviewed[0].heading"))
    );
    assert!(diagnostics.iter().any(|diagnostic| diagnostic.message.contains("absolute HTTPS URL")));
    assert!(diagnostics.iter().any(|diagnostic| diagnostic.message.contains("cannot declare existing_entity")));
}

#[test]
fn reports_invalid_links_and_keeps_diagnostics_deterministic() {
    let (_root, snapshot, path) = bundle();
    fs::write(&path, "source_reference: R999\nsource_file: page.html\nevidence: []\ndraft_entity_types: [{id: DT1, label: Draft, description: Draft type, evidence: [], affected_candidates: [C999]}]\ndraft_properties: []\narticle_results: []\ncandidates: [{key: bad, proposed_metadata: {classifications: [DT999]}, evidence: [E001], recommended_outcome: new, statements: [], dependencies: []}]\n").unwrap();
    let diagnostics = validate_ingestion_candidate_inventory(&path, &snapshot);
    assert!(diagnostics.iter().any(|item| item.message.contains("R999")));
    assert!(diagnostics.iter().any(|item| item.message.contains("E001")));
    let mut sorted = diagnostics.clone();
    sorted.sort_by(|left, right| (&left.path, &left.identifier, &left.message).cmp(&(&right.path, &right.identifier, &right.message)));
    sorted.dedup();
    assert_eq!(diagnostics, sorted);
}
