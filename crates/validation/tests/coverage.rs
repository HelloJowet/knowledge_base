use knowledge_base_validation::{Diagnostic, ValidationLayer, validate_repository};
use std::path::{Path, PathBuf};

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..").join("fixtures").join(name)
}

fn diagnostic(layer: ValidationLayer, path: &str, line: Option<usize>, identifier: Option<&str>, message: &str) -> Diagnostic {
    Diagnostic {
        layer,
        path: PathBuf::from(path),
        line,
        identifier: identifier.map(str::to_owned),
        message: message.to_owned(),
    }
}

fn assert_fixture(name: &str, expected: Vec<Diagnostic>) {
    assert_eq!(validate_repository(fixture(name)), expected, "fixture: {name}");
}

#[test]
fn valid_fixtures_cover_minimal_and_all_value_shapes() {
    for name in ["valid/minimal", "valid/all-values"] {
        assert_fixture(name, Vec::new());
    }
}

#[test]
fn schema_fixture_reports_loading_and_shape_errors() {
    assert_fixture(
        "invalid/schema",
        vec![
            diagnostic(
                ValidationLayer::Schema,
                "entities/Q2.yaml",
                Some(3),
                None,
                "invalid YAML: did not find expected node content at line 3 column 1, while parsing a flow node",
            ),
            diagnostic(ValidationLayer::Schema, "entities/Q3.yaml", Some(1), None, "invalid YAML: duplicate entry with key \"id\""),
            diagnostic(
                ValidationLayer::Schema,
                "entities/unexpected.txt",
                None,
                None,
                "unexpected entry; managed directories may contain only .yaml files",
            ),
            diagnostic(
                ValidationLayer::Schema,
                "entity_types/README.txt",
                None,
                None,
                "unexpected entry; managed directories may contain only .yaml files",
            ),
            diagnostic(
                ValidationLayer::Schema,
                "properties/README.txt",
                None,
                None,
                "unexpected entry; managed directories may contain only .yaml files",
            ),
            diagnostic(
                ValidationLayer::Schema,
                "references/README.txt",
                None,
                None,
                "unexpected entry; managed directories may contain only .yaml files",
            ),
        ],
    );
}

#[test]
fn missing_required_directories_are_reported() {
    assert_fixture(
        "invalid/schema-missing",
        ["entities", "entity_types", "properties", "references"]
            .into_iter()
            .map(|path| {
                diagnostic(
                    ValidationLayer::Schema,
                    path,
                    None,
                    None,
                    "required directory cannot be read: No such file or directory (os error 2)",
                )
            })
            .collect(),
    );
}

#[test]
fn localization_and_reference_fixture_reports_complete_diagnostics() {
    assert_fixture(
        "invalid/localization-provenance",
        vec![
            diagnostic(ValidationLayer::Schema, "entities/Q1.yaml", None, Some("Q1"), "entity_types must not be empty"),
            diagnostic(
                ValidationLayer::Schema,
                "entities/Q1.yaml",
                None,
                Some("Q1"),
                "labels contains locale \"tr\" more than once ignoring case",
            ),
            diagnostic(
                ValidationLayer::Schema,
                "entities/Q1.yaml",
                None,
                Some("Q1"),
                "labels locale \"invalid_tag\" is not a well-formed BCP 47 tag",
            ),
            diagnostic(ValidationLayer::Schema, "entities/Q1.yaml", None, Some("Q1"), "labels.TR references must not be empty"),
            diagnostic(ValidationLayer::Provenance, "entities/Q1.yaml", None, Some("Q1"), "labels.tr cites missing reference R99"),
            diagnostic(ValidationLayer::Schema, "entities/Q1.yaml", None, Some("Q1"), "labels.tr text must not be empty"),
            diagnostic(ValidationLayer::Schema, "entity_types/T1.yaml", None, Some("T1"), "labels must not be empty"),
            diagnostic(ValidationLayer::Schema, "properties/P1.yaml", None, Some("P1"), "labels must not be empty"),
            diagnostic(ValidationLayer::Schema, "properties/P1.yaml", None, Some("P1"), "subject_types must not be empty"),
            diagnostic(ValidationLayer::Schema, "references/R1.yaml", None, Some("R1"), "archive_url must be an absolute URL"),
            diagnostic(
                ValidationLayer::Schema,
                "references/R1.yaml",
                None,
                Some("R1"),
                "publication_date must be a valid YYYY, YYYY-MM, or YYYY-MM-DD date",
            ),
            diagnostic(ValidationLayer::Schema, "references/R1.yaml", None, Some("R1"), "publisher must not be empty"),
            diagnostic(
                ValidationLayer::Schema,
                "references/R1.yaml",
                None,
                Some("R1"),
                "retrieved_at must be an RFC 3339 timestamp",
            ),
            diagnostic(
                ValidationLayer::Schema,
                "references/R1.yaml",
                None,
                Some("R1"),
                "source_language must be a well-formed BCP 47 tag",
            ),
            diagnostic(ValidationLayer::Schema, "references/R1.yaml", None, Some("R1"), "title must not be empty"),
            diagnostic(ValidationLayer::Schema, "references/R1.yaml", None, Some("R1"), "url must be an absolute URL"),
        ],
    );
}

#[test]
fn ontology_fixture_reports_complete_diagnostics() {
    assert_fixture(
        "invalid/ontology",
        vec![
            diagnostic(ValidationLayer::Ontology, "entities/Q1.yaml", None, Some("Q1"), "classified entity type T99 does not exist"),
            diagnostic(ValidationLayer::Ontology, "entities/Q1.yaml", None, Some("Q1"), "statement property P99 does not exist"),
            diagnostic(
                ValidationLayer::Schema,
                "id_allocation.yaml",
                None,
                Some("id_allocation"),
                "next.entity must be greater than the greatest used identifier number (1)",
            ),
            diagnostic(
                ValidationLayer::Schema,
                "id_allocation.yaml",
                None,
                Some("id_allocation"),
                "next.entity_type must be greater than the greatest used identifier number (1)",
            ),
            diagnostic(
                ValidationLayer::Schema,
                "id_allocation.yaml",
                None,
                Some("id_allocation"),
                "next.property must be greater than the greatest used identifier number (1)",
            ),
            diagnostic(
                ValidationLayer::Schema,
                "id_allocation.yaml",
                None,
                Some("id_allocation"),
                "next.reference must be greater than the greatest used identifier number (1)",
            ),
            diagnostic(ValidationLayer::Schema, "id_allocation.yaml", None, Some("id_allocation"), "version must be 1"),
            diagnostic(
                ValidationLayer::Ontology,
                "properties/P1.yaml",
                None,
                Some("P1"),
                "allowed qualifier property P99 does not exist",
            ),
            diagnostic(ValidationLayer::Schema, "properties/P1.yaml", None, Some("P1"), "subject_types must not be empty"),
            diagnostic(ValidationLayer::Schema, "properties/P1.yaml", None, Some("P1"), "target_types must not be empty"),
        ],
    );
}

#[test]
fn entity_context_fixture_keeps_known_gaps_visible() {
    assert_fixture(
        "invalid/entity-context",
        vec![
            diagnostic(
                ValidationLayer::Provenance,
                "entity_context/Q1.md",
                Some(1),
                Some("Q1"),
                "footnote \"R2\" has no definition",
            ),
            diagnostic(
                ValidationLayer::Provenance,
                "entity_context/Q1.md",
                Some(1),
                Some("Q1"),
                "footnote \"R99\" cites a reference that does not exist",
            ),
            diagnostic(
                ValidationLayer::Provenance,
                "entity_context/Q1.md",
                Some(3),
                Some("Q1"),
                "footnote \"R99\" cites a reference that does not exist",
            ),
            diagnostic(
                ValidationLayer::Provenance,
                "entity_context/Q1.md",
                Some(3),
                Some("Q1"),
                "footnote \"R99\" must contain exactly one link to ../references/R99.yaml",
            ),
            diagnostic(ValidationLayer::Provenance, "entity_context/Q1.md", Some(5), Some("Q1"), "footnote \"R1\" is unused"),
            diagnostic(
                ValidationLayer::Provenance,
                "entity_context/Q99.md",
                None,
                Some("Q99"),
                "context document names an entity that does not exist",
            ),
            diagnostic(
                ValidationLayer::Schema,
                "id_allocation.yaml",
                None,
                Some("id_allocation"),
                "next.property must be greater than the greatest used identifier number (1)",
            ),
        ],
    );
}

#[test]
fn diagnostic_display_and_sorting_cover_all_context_combinations() {
    let diagnostics = [
        diagnostic(ValidationLayer::Schema, "a", None, None, "m"),
        diagnostic(ValidationLayer::Ontology, "a", Some(2), None, "m"),
        diagnostic(ValidationLayer::Provenance, "a", None, Some("Q1"), "m"),
        diagnostic(ValidationLayer::Domain, "a", Some(2), Some("Q1"), "m"),
    ];
    assert_eq!(diagnostics[0].to_string(), "a [schema] m");
    assert_eq!(diagnostics[1].to_string(), "a:2 [ontology] m");
    assert_eq!(diagnostics[2].to_string(), "a [provenance] [Q1] m");
    assert_eq!(diagnostics[3].to_string(), "a:2 [domain] [Q1] m");
}
