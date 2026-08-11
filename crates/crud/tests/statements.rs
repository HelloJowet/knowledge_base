use knowledge_base_crud::{ApplyMode, ApplyStatementsOutcome, Error, KnowledgeBase, StatementBatch, StatementResultStatus};
use knowledge_base_validation::{AdditionalValidator, Diagnostic, ValidationLayer};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

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

#[test]
fn configured_validators_run_against_the_baseline_and_staged_repository_for_preview_and_commit() {
    let root = copied_fixture();
    let calls = Arc::new(Mutex::new(Vec::new()));
    let validator_calls = Arc::clone(&calls);
    let validator: Arc<dyn AdditionalValidator> = Arc::new(move |context: &knowledge_base_validation::ValidationContext<'_>| -> Vec<Diagnostic> {
        let has_planned_statement = context.snapshot().entities()[&"Q1".parse().unwrap()]
            .statements
            .iter()
            .any(|statement| matches!(statement.value, knowledge_base_models::Value::Integer { value: 123456789 }));
        validator_calls.lock().unwrap().push((context.repository_root().to_path_buf(), has_planned_statement));
        Vec::new()
    });
    let knowledge_base = KnowledgeBase::with_additional_validators(root.path(), [validator]);

    knowledge_base.entities().apply_statements(&batch(), ApplyMode::Preview).unwrap();
    knowledge_base.entities().apply_statements(&batch(), ApplyMode::Commit).unwrap();

    let calls = calls.lock().unwrap();
    assert_eq!(calls.len(), 4);
    assert_eq!(calls.iter().filter(|(path, planned)| path == root.path() && !planned).count(), 2);
    assert_eq!(calls.iter().filter(|(path, planned)| path != root.path() && *planned).count(), 2);
}

#[test]
fn a_staged_domain_diagnostic_rejects_preview_and_commit_without_changing_files() {
    let root = copied_fixture();
    let entity_path = root.path().join("entities/Q1.yaml");
    let before = fs::read(&entity_path).unwrap();
    let validator: Arc<dyn AdditionalValidator> = Arc::new(|context: &knowledge_base_validation::ValidationContext<'_>| {
        if context.snapshot().entities()[&"Q1".parse().unwrap()]
            .statements
            .iter()
            .any(|statement| matches!(statement.value, knowledge_base_models::Value::Integer { value: 123456789 }))
        {
            vec![Diagnostic {
                layer: ValidationLayer::Domain,
                path: PathBuf::from("entities/Q1.yaml"),
                line: None,
                identifier: Some("Q1".to_owned()),
                message: "domain policy rejects this statement".to_owned(),
            }]
        } else {
            Vec::new()
        }
    });
    let knowledge_base = KnowledgeBase::with_additional_validators(root.path(), [validator]);

    for mode in [ApplyMode::Preview, ApplyMode::Commit] {
        let error = knowledge_base.entities().apply_statements(&batch(), mode).unwrap_err();
        assert!(matches!(error, Error::Validation(ref diagnostics) if diagnostics.iter().any(|diagnostic| diagnostic.message == "domain policy rejects this statement")));
        assert_eq!(fs::read(&entity_path).unwrap(), before);
    }
}

#[test]
fn every_configured_validator_runs_on_an_invalid_baseline() {
    let root = copied_fixture();
    let calls = Arc::new(Mutex::new(Vec::new()));
    let first_calls = Arc::clone(&calls);
    let first: Arc<dyn AdditionalValidator> = Arc::new(move |context: &knowledge_base_validation::ValidationContext<'_>| {
        first_calls.lock().unwrap().push(("first", context.repository_root().to_path_buf()));
        vec![Diagnostic {
            layer: ValidationLayer::Domain,
            path: PathBuf::from("domain.yaml"),
            line: None,
            identifier: None,
            message: "first failure".to_owned(),
        }]
    });
    let second_calls = Arc::clone(&calls);
    let second: Arc<dyn AdditionalValidator> = Arc::new(move |context: &knowledge_base_validation::ValidationContext<'_>| {
        second_calls.lock().unwrap().push(("second", context.repository_root().to_path_buf()));
        vec![Diagnostic {
            layer: ValidationLayer::Domain,
            path: PathBuf::from("domain.yaml"),
            line: None,
            identifier: None,
            message: "second failure".to_owned(),
        }]
    });
    let knowledge_base = KnowledgeBase::with_additional_validators(root.path(), [first, second]);
    let entity_path = root.path().join("entities/Q1.yaml");
    let before = fs::read(&entity_path).unwrap();

    assert!(matches!(knowledge_base.entities().apply_statements(&batch(), ApplyMode::Commit), Err(Error::Validation(_))));
    assert_eq!(
        calls.lock().unwrap().as_slice(),
        [("first", root.path().to_path_buf()), ("second", root.path().to_path_buf())]
    );
    assert_eq!(fs::read(entity_path).unwrap(), before);
}
