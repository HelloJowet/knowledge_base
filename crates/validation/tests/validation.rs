use knowledge_base_validation::{Diagnostic, ValidationLayer, validate_repository};
use std::path::{Path, PathBuf};

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..").join("fixtures").join(name)
}

#[test]
fn minimal_fixture_is_valid() {
    assert!(validate_repository(fixture("valid/minimal")).is_empty());
}

#[test]
fn missing_root_returns_a_schema_diagnostic() {
    let diagnostics = validate_repository(fixture("does-not-exist"));

    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].layer, ValidationLayer::Schema);
    assert!(diagnostics[0].message.contains("not a readable directory"));
}

#[test]
fn multiple_diagnostics_are_complete_and_deterministic() {
    let root = tempfile::tempdir().expect("temporary directory");
    let first = validate_repository(root.path());
    let second = validate_repository(root.path());

    assert!(first.len() > 1);
    assert_eq!(first, second);
    assert!(first.windows(2).all(|pair| {
        (&pair[0].path, pair[0].line.unwrap_or(usize::MAX), &pair[0].identifier, &pair[0].message, pair[0].layer)
            <= (&pair[1].path, pair[1].line.unwrap_or(usize::MAX), &pair[1].identifier, &pair[1].message, pair[1].layer)
    }));
}

#[test]
fn diagnostic_display_includes_all_available_context() {
    let diagnostic = Diagnostic {
        layer: ValidationLayer::Ontology,
        path: PathBuf::from("entities/Q1.yaml"),
        line: Some(12),
        identifier: Some("Q1/S3".to_owned()),
        message: "target entity Q99 does not exist".to_owned(),
    };

    assert_eq!(diagnostic.to_string(), "entities/Q1.yaml:12 [ontology] [Q1/S3] target entity Q99 does not exist");
}
