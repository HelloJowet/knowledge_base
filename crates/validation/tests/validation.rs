use knowledge_base_validation::{AdditionalValidator, Diagnostic, ValidationLayer, validate_repository, validate_repository_with};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

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
fn additional_validators_all_run_and_their_diagnostics_are_sorted_with_built_ins() {
    let root = tempfile::tempdir().expect("temporary directory");
    let calls = Arc::new(Mutex::new(Vec::new()));
    let first_calls = Arc::clone(&calls);
    let first = move |path: &Path| {
        first_calls.lock().unwrap().push(("first", path.to_path_buf()));
        vec![Diagnostic {
            layer: ValidationLayer::Domain,
            path: PathBuf::from("z-domain.yaml"),
            line: None,
            identifier: None,
            message: "last".to_owned(),
        }]
    };
    let second_calls = Arc::clone(&calls);
    let second = move |path: &Path| {
        second_calls.lock().unwrap().push(("second", path.to_path_buf()));
        vec![Diagnostic {
            layer: ValidationLayer::Domain,
            path: PathBuf::from("a-domain.yaml"),
            line: None,
            identifier: None,
            message: "first".to_owned(),
        }]
    };
    let validators: [&dyn AdditionalValidator; 2] = [&first, &second];

    let diagnostics = validate_repository_with(root.path(), validators);

    assert_eq!(
        calls.lock().unwrap().as_slice(),
        [("first", root.path().to_path_buf()), ("second", root.path().to_path_buf())]
    );
    assert!(diagnostics.iter().any(|diagnostic| diagnostic.layer == ValidationLayer::Schema));
    let domain_paths = diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.layer == ValidationLayer::Domain)
        .map(|diagnostic| diagnostic.path.clone())
        .collect::<Vec<_>>();
    assert_eq!(domain_paths, [PathBuf::from("a-domain.yaml"), PathBuf::from("z-domain.yaml")]);
}

#[test]
fn validation_without_additional_validators_is_unchanged() {
    let root = fixture("valid/minimal");
    assert_eq!(validate_repository(&root), validate_repository_with(&root, std::iter::empty()));
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

#[test]
fn property_usage_restricts_statement_and_qualifier_positions() {
    for (property, replacement, expected_message) in [
        ("P1.yaml", "usage: qualifier", "property P1 cannot be used as a statement"),
        ("P2.yaml", "usage: statement", "property P2 cannot be used as a qualifier"),
    ] {
        let root = copied_minimal_fixture();
        let path = root.path().join("properties").join(property);
        let source = fs::read_to_string(&path).expect("property fixture");
        fs::write(&path, source.replacen("usage: statement", replacement, 1).replacen("usage: qualifier", replacement, 1)).expect("updated property fixture");

        assert!(
            validate_repository(root.path()).iter().any(|diagnostic| diagnostic.message == expected_message),
            "missing diagnostic: {expected_message}"
        );
    }

    let root = copied_minimal_fixture();
    let path = root.path().join("properties/P2.yaml");
    let source = fs::read_to_string(&path).expect("property fixture");
    fs::write(&path, source.replace("usage: qualifier", "usage: both")).expect("updated property fixture");
    assert!(validate_repository(root.path()).is_empty());
}

#[test]
fn external_ids_require_meaningful_unique_identifiers() {
    for (block, expected_message) in [
        ("external_ids:\n  '   ': [P1082]\n", "external_ids namespace must not be empty"),
        ("external_ids:\n  wikidata: ['   ']\n", "external_ids.wikidata identifier must not be empty"),
        (
            "external_ids:\n  wikidata: [P1082, P1082]\n",
            "external_ids.wikidata contains duplicate identifier \"P1082\"",
        ),
    ] {
        let root = copied_minimal_fixture();
        let path = root.path().join("properties/P1.yaml");
        let source = fs::read_to_string(&path).expect("property fixture");
        fs::write(&path, format!("{source}{block}")).expect("updated property fixture");

        assert!(
            validate_repository(root.path()).iter().any(|diagnostic| diagnostic.message == expected_message),
            "missing diagnostic: {expected_message}"
        );
    }

    let root = copied_minimal_fixture();
    let path = root.path().join("properties/P1.yaml");
    let source = fs::read_to_string(&path).expect("property fixture");
    fs::write(&path, format!("{source}external_ids:\n  wikidata: [P1082, P2046]\n  osm: []\n")).expect("updated property fixture");
    assert!(validate_repository(root.path()).is_empty());
}

#[test]
fn ontology_text_may_be_uncited_but_entity_text_must_be_cited() {
    let root = copied_minimal_fixture();
    for path in [root.path().join("properties/P1.yaml"), root.path().join("entity_types/T1.yaml")] {
        let source = fs::read_to_string(&path).expect("ontology fixture");
        fs::write(&path, source.replace("references: [R1]", "references: []")).expect("updated ontology fixture");
    }
    assert!(validate_repository(root.path()).is_empty());

    let path = root.path().join("entities/Q1.yaml");
    let source = fs::read_to_string(&path).expect("entity fixture");
    fs::write(&path, source.replacen("references: [R1]", "references: []", 1)).expect("updated entity fixture");
    assert!(
        validate_repository(root.path())
            .iter()
            .any(|diagnostic| diagnostic.message == "labels.tr references must not be empty")
    );
}
