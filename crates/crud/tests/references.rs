use knowledge_base_crud::{ApplyMode, Error, KnowledgeBase, ReferenceDraft, ReferenceRegistrationStatus};
use std::fs;
use std::path::{Path, PathBuf};

fn fixture() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..").join("fixtures/valid/minimal")
}

fn copied_fixture() -> tempfile::TempDir {
    let destination = tempfile::tempdir().unwrap();
    for directory in ["entities", "entity_types", "properties", "references", "entity_context"] {
        let source = fixture().join(directory);
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

fn draft() -> ReferenceDraft {
    ReferenceDraft {
        url: "https://example.org/new-source".to_owned(),
        title: "New source".to_owned(),
        publisher: Some("Example Publisher".to_owned()),
        publication_date: Some("2026-08".to_owned()),
        source_language: Some("en".to_owned()),
        retrieved_at: "2026-08-11T12:00:00Z".to_owned(),
        archive_url: Some("https://archive.example.org/new-source".to_owned()),
    }
}

#[test]
fn previews_and_registers_a_reference_with_all_metadata() {
    let root = copied_fixture();
    let knowledge_base = KnowledgeBase::new(root.path());
    let reference_path = root.path().join("references/R2.yaml");
    let allocation_path = root.path().join("id_allocation.yaml");
    let allocation_before = fs::read(&allocation_path).unwrap();

    let preview = knowledge_base.references().register(&draft(), ApplyMode::Preview).unwrap();
    assert_eq!(preview.status, ReferenceRegistrationStatus::Previewed);
    assert_eq!(preview.reference.as_str(), "R2");
    assert!(!reference_path.exists());
    assert_eq!(fs::read(&allocation_path).unwrap(), allocation_before);

    let registered = knowledge_base.references().register(&draft(), ApplyMode::Commit).unwrap();
    assert_eq!(registered.status, ReferenceRegistrationStatus::Registered);
    assert_eq!(registered.reference.as_str(), "R2");
    let stored = fs::read_to_string(&reference_path).unwrap();
    assert!(stored.contains("id: R2\n"));
    assert!(stored.contains("url: https://example.org/new-source\n"));
    assert!(stored.contains("archive_url: https://archive.example.org/new-source\n"));
    assert!(fs::read_to_string(allocation_path).unwrap().contains("reference: 3"));
}

#[test]
fn duplicate_url_reuses_the_lowest_reference_identifier_without_writes() {
    let root = copied_fixture();
    fs::write(
        root.path().join("references/R2.yaml"),
        "id: R2\nurl: https://example.org/bilecik\ntitle: Duplicate source\nretrieved_at: 2026-08-11T12:00:00Z\n",
    )
    .unwrap();
    fs::write(
        root.path().join("id_allocation.yaml"),
        "version: 1\nnext:\n  entity: 3\n  property: 4\n  reference: 3\n  entity_type: 3\n",
    )
    .unwrap();
    let knowledge_base = KnowledgeBase::new(root.path());
    let mut duplicate = draft();
    duplicate.url = "https://example.org/bilecik".to_owned();
    let allocation = root.path().join("id_allocation.yaml");
    let before = fs::read(&allocation).unwrap();

    for mode in [ApplyMode::Preview, ApplyMode::Commit] {
        let result = knowledge_base.references().register(&duplicate, mode).unwrap();
        assert_eq!(result.status, ReferenceRegistrationStatus::Existing);
        assert_eq!(result.reference.as_str(), "R1");
    }
    assert!(root.path().join("references/R2.yaml").exists());
    assert_eq!(fs::read(allocation).unwrap(), before);
}

#[test]
fn rejects_invalid_requests_and_invalid_repositories_without_writes() {
    let root = copied_fixture();
    let knowledge_base = KnowledgeBase::new(root.path());
    let reference_path = root.path().join("references/R2.yaml");
    let allocation_path = root.path().join("id_allocation.yaml");
    let allocation_before = fs::read(&allocation_path).unwrap();
    let mut invalid = draft();
    invalid.title = " ".to_owned();

    assert!(matches!(knowledge_base.references().register(&invalid, ApplyMode::Commit), Err(Error::InvalidRequest(_))));
    assert!(!reference_path.exists());
    assert_eq!(fs::read(&allocation_path).unwrap(), allocation_before);

    fs::write(root.path().join("references/R1.yaml"), "not: a reference\n").unwrap();
    assert!(matches!(knowledge_base.references().register(&draft(), ApplyMode::Commit), Err(Error::Validation(_))));
    assert!(!reference_path.exists());
    assert_eq!(fs::read(&allocation_path).unwrap(), allocation_before);
}

#[test]
fn reports_reference_allocation_exhaustion_without_writes() {
    let root = copied_fixture();
    fs::write(
        root.path().join("id_allocation.yaml"),
        format!("version: 1\nnext:\n  entity: 3\n  property: 4\n  reference: {}\n  entity_type: 3\n", u64::MAX),
    )
    .unwrap();
    let knowledge_base = KnowledgeBase::new(root.path());
    let allocation_path = root.path().join("id_allocation.yaml");
    let before = fs::read(&allocation_path).unwrap();

    let error = knowledge_base.references().register(&draft(), ApplyMode::Commit).unwrap_err();

    assert!(error.to_string().contains("cannot allocate another reference identifier"));
    assert!(!root.path().join(format!("references/R{}.yaml", u64::MAX)).exists());
    assert_eq!(fs::read(allocation_path).unwrap(), before);
}
