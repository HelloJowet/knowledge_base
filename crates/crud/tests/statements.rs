use knowledge_base_crud::{ApplyMode, ApplyStatementsOutcome, Error, KnowledgeBase, StatementBatch, StatementResultStatus};
use std::fs;
use std::path::{Path, PathBuf};

fn fixture() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..").join("fixtures/valid/minimal")
}

fn copied_fixture() -> tempfile::TempDir {
    let destination = tempfile::tempdir().unwrap();
    for directory in ["entities", "entity_types", "properties", "references", "entity_context"] {
        let source = fixture().join(directory);
        if !source.exists() {
            continue;
        }
        let target = destination.path().join(directory);
        fs::create_dir(&target).unwrap();
        for entry in fs::read_dir(source).unwrap() {
            let entry = entry.unwrap();
            fs::copy(entry.path(), target.join(entry.file_name())).unwrap();
        }
    }
    fs::copy(fixture().join("id_allocation.yaml"), destination.path().join("id_allocation.yaml")).unwrap();
    destination
}

fn batch() -> StatementBatch {
    serde_yaml::from_str("statements:\n  - entity: Q1\n    property: P1\n    value: { type: integer, value: 123456789 }\n    references: [R1]\n").unwrap()
}

#[test]
fn resource_service_previews_commits_and_rejects_repeated_batches() {
    let root = copied_fixture();
    let knowledge_base = KnowledgeBase::new(root.path());
    let entity_path = root.path().join("entities/Q1.yaml");
    let before = fs::read(&entity_path).unwrap();

    let preview = knowledge_base.entities().apply_statements(&batch(), ApplyMode::Preview).unwrap();
    assert!(matches!(preview, ApplyStatementsOutcome::Previewed(_)));
    assert_eq!(preview.results()[0].status, StatementResultStatus::WouldAdd);
    assert_eq!(fs::read(&entity_path).unwrap(), before);

    let applied = knowledge_base.entities().apply_statements(&batch(), ApplyMode::Commit).unwrap();
    assert!(applied.was_applied());
    assert_eq!(applied.results()[0].status, StatementResultStatus::Added);
    let after = fs::read(&entity_path).unwrap();
    assert_ne!(after, before);

    let repeated = knowledge_base.entities().apply_statements(&batch(), ApplyMode::Commit).unwrap();
    assert!(matches!(repeated, ApplyStatementsOutcome::NotApplied(_)));
    assert_eq!(repeated.results()[0].status, StatementResultStatus::AlreadyPresent);
    assert_eq!(fs::read(entity_path).unwrap(), after);
}

#[test]
fn invalid_repository_is_rejected_before_entity_files_change() {
    let root = copied_fixture();
    fs::write(root.path().join("references/R1.yaml"), "not: a reference\n").unwrap();
    let knowledge_base = KnowledgeBase::new(root.path());
    let entity_path = root.path().join("entities/Q1.yaml");
    let before = fs::read(&entity_path).unwrap();

    let error = knowledge_base.entities().apply_statements(&batch(), ApplyMode::Commit).unwrap_err();

    assert!(matches!(error, Error::Validation(_)));
    assert_eq!(fs::read(entity_path).unwrap(), before);
}
