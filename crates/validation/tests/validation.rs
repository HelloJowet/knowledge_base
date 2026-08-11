use knowledge_base_validation::{Diagnostic, ValidationLayer, validate_repository};
use std::fs;
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

fn copied_minimal_fixture() -> tempfile::TempDir {
    let root = tempfile::tempdir().expect("temporary directory");
    let source = fixture("valid/minimal");
    for directory in ["entities", "entity_types", "properties", "references", "entity_context"] {
        let target = root.path().join(directory);
        fs::create_dir(&target).expect("fixture directory");
        for entry in fs::read_dir(source.join(directory)).expect("fixture entries") {
            let entry = entry.expect("fixture entry");
            fs::copy(entry.path(), target.join(entry.file_name())).expect("copy fixture file");
        }
    }
    fs::copy(source.join("id_allocation.yaml"), root.path().join("id_allocation.yaml")).expect("copy allocation");
    root
}

fn image_fixture_block() -> &'static str {
    "images:\n  - url: https://example.org/bilecik.jpg\n    alt: Bilecik city centre\n    source_url: https://example.org/bilecik-image\n    creator: Example Archive\n    license: CC BY 4.0\n    references: [R1]"
}

#[test]
fn images_validate_metadata_urls_and_optional_references() {
    for (original, replacement, expected_message) in [
        ("url: https://example.org/bilecik.jpg", "url: relative.jpg", "image url must be an absolute URL"),
        (
            "source_url: https://example.org/bilecik-image",
            "source_url: relative.jpg",
            "image source_url must be an absolute URL",
        ),
        ("alt: Bilecik city centre", "alt: '   '", "image alt must not be empty"),
        ("creator: Example Archive", "creator: '   '", "image creator must not be empty"),
        ("license: CC BY 4.0", "license: '   '", "image license must not be empty"),
        ("references: [R1]", "references: [R2]", "image cites missing reference R2"),
    ] {
        let root = copied_minimal_fixture();
        let entity_path = root.path().join("entities/Q1.yaml");
        let entity = fs::read_to_string(&entity_path).expect("entity fixture");
        let updated_image = image_fixture_block().replacen(original, replacement, 1);
        fs::write(&entity_path, entity.replace(image_fixture_block(), &updated_image)).expect("updated entity fixture");

        assert!(
            validate_repository(root.path()).iter().any(|diagnostic| diagnostic.message == expected_message),
            "missing diagnostic: {expected_message}"
        );
    }

    let root = copied_minimal_fixture();
    let entity_path = root.path().join("entities/Q1.yaml");
    let entity = fs::read_to_string(&entity_path).expect("entity fixture");
    let uncited_image = image_fixture_block().replace("\n    references: [R1]", "");
    fs::write(&entity_path, entity.replace(image_fixture_block(), &uncited_image)).expect("entity without image references");
    assert!(validate_repository(root.path()).is_empty());
}
