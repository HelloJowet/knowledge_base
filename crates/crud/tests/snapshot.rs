use knowledge_base_crud::{KnowledgeBaseRepository, RepositoryError};
use knowledge_base_models::{EntityId, PropertyId};
use knowledge_base_snapshot::Error as SnapshotError;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

fn fixture() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..").join("fixtures/valid/minimal")
}

#[test]
fn snapshot_loads_all_structured_resources_in_identifier_order() {
    let snapshot = KnowledgeBaseRepository::new(fixture()).read().snapshot().expect("snapshot loads");

    assert_eq!(snapshot.entities().keys().map(|id| id.as_str()).collect::<Vec<_>>(), ["Q1", "Q2"]);
    assert_eq!(snapshot.entity_types().keys().map(|id| id.as_str()).collect::<Vec<_>>(), ["T1", "T2"]);
    assert_eq!(snapshot.properties().keys().map(|id| id.as_str()).collect::<Vec<_>>(), ["P1", "P2", "P3"]);
    assert_eq!(snapshot.references().keys().map(|id| id.as_str()).collect::<Vec<_>>(), ["R1"]);
    assert_eq!(snapshot.entities()[&"Q1".parse::<EntityId>().unwrap()].id.as_str(), "Q1");
    assert_eq!(snapshot.properties()[&"P1".parse::<PropertyId>().unwrap()].id.as_str(), "P1");
    assert_eq!(snapshot.allocation().next.entity, 3);
}

fn copied_fixture() -> TempDir {
    let destination = tempfile::tempdir().expect("temporary repository");
    for directory in ["entities", "entity_types", "properties", "references"] {
        let target = destination.path().join(directory);
        fs::create_dir(&target).unwrap();
        for entry in fs::read_dir(fixture().join(directory)).unwrap() {
            let entry = entry.unwrap();
            fs::copy(entry.path(), target.join(entry.file_name())).unwrap();
        }
    }
    fs::copy(fixture().join("id_allocation.yaml"), destination.path().join("id_allocation.yaml")).unwrap();
    destination
}

#[test]
fn snapshot_rejects_non_yaml_managed_resource() {
    let repository = copied_fixture();
    let path = repository.path().join("entities/notes.txt");
    fs::write(&path, "not managed").unwrap();

    let error = KnowledgeBaseRepository::new(repository.path()).read().snapshot().expect_err("non-YAML file is rejected");

    assert!(matches!(error, RepositoryError::Snapshot(SnapshotError::InvalidSnapshot { path: error_path, .. }) if error_path == path));
}

#[test]
fn snapshot_rejects_filename_identifier_mismatch() {
    let repository = copied_fixture();
    let old_path = repository.path().join("entities/Q1.yaml");
    let path = repository.path().join("entities/Q99.yaml");
    fs::rename(old_path, &path).unwrap();

    let error = KnowledgeBaseRepository::new(repository.path())
        .read()
        .snapshot()
        .expect_err("mismatched filename is rejected");

    assert!(matches!(error, RepositoryError::Snapshot(SnapshotError::InvalidSnapshot { path: error_path, .. }) if error_path == path));
}
