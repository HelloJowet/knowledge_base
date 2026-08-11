use knowledge_base_crud::{EntityFilter, KnowledgeBase};
use knowledge_base_models::{PropertyId, Value};
use std::fs;
use std::path::Path;

fn write_entity(root: &Path, filename: &str, id: &str, statements: &str) {
    fs::write(
        root.join("entities").join(filename),
        format!("id: {id}\nlabels: {{}}\nentity_types: []\nstatements:{statements}\n"),
    )
    .unwrap();
}

fn filter(property: &str, target: &str) -> EntityFilter {
    EntityFilter {
        property: property.parse::<PropertyId>().unwrap(),
        value: Value::Entity { value: target.parse().unwrap() },
    }
}

#[test]
fn query_matches_all_top_level_filters_then_sorts_and_pages() {
    let root = tempfile::tempdir().unwrap();
    fs::create_dir(root.path().join("entities")).unwrap();
    let both = "\n  - id: S1\n    property: P1\n    value: { type: entity, value: Q43 }\n    references: []\n  - id: S2\n    property: P2\n    value: { type: entity, value: Q44 }\n    qualifiers:\n      - property: P9\n        value: { type: entity, value: Q99 }\n    references: []";
    write_entity(root.path(), "Q10.yaml", "Q10", both);
    write_entity(root.path(), "Q2.yaml", "Q2", both);
    write_entity(
        root.path(),
        "Q3.yaml",
        "Q3",
        "\n  - id: S1\n    property: P1\n    value: { type: entity, value: Q43 }\n    references: []",
    );

    let filters = [filter("P1", "Q43"), filter("P2", "Q44")];
    let page = KnowledgeBase::new(root.path()).entities().query(&filters, 1, 0).unwrap();
    assert_eq!(page.total, 2);
    assert_eq!(page.next_offset, Some(1));
    assert_eq!(page.entities[0].id.as_str(), "Q2");

    let second = KnowledgeBase::new(root.path()).entities().query(&filters, 1, 1).unwrap();
    assert_eq!(second.entities[0].id.as_str(), "Q10");
    assert_eq!(second.next_offset, None);

    let qualifier_only = KnowledgeBase::new(root.path()).entities().query(&[filter("P9", "Q99")], 100, 0).unwrap();
    assert_eq!(qualifier_only.total, 0);
}

#[test]
fn repeated_property_filters_require_each_value_and_empty_pages_keep_the_total() {
    let root = tempfile::tempdir().unwrap();
    fs::create_dir(root.path().join("entities")).unwrap();
    write_entity(
        root.path(),
        "Q1.yaml",
        "Q1",
        "\n  - id: S1\n    property: P1\n    value: { type: entity, value: Q43 }\n    references: []\n  - id: S2\n    property: P1\n    value: { type: entity, value: Q44 }\n    references: []",
    );

    let filters = [filter("P1", "Q43"), filter("P1", "Q44")];
    let page = KnowledgeBase::new(root.path()).entities().query(&filters, 10, 99).unwrap();
    assert_eq!(page.total, 1);
    assert!(page.entities.is_empty());
    assert_eq!(page.next_offset, None);
}

#[test]
fn query_rejects_invalid_requests_and_ambiguous_repositories() {
    let root = tempfile::tempdir().unwrap();
    fs::create_dir(root.path().join("entities")).unwrap();
    write_entity(root.path(), "Q1.yaml", "Q1", " []");
    let knowledge_base = KnowledgeBase::new(root.path());
    assert!(knowledge_base.entities().query(&[], 1, 0).unwrap_err().to_string().contains("at least one"));
    assert!(knowledge_base.entities().query(&[filter("P1", "Q43")], 0, 0).unwrap_err().to_string().contains("limit"));

    write_entity(root.path(), "Q2.yaml", "Q1", " []");
    let error = KnowledgeBase::new(root.path()).entities().query(&[filter("P1", "Q43")], 1, 0).unwrap_err();
    assert!(error.to_string().contains("duplicate entity identifier Q1"));

    fs::write(root.path().join("entities/Q2.yaml"), "not: an entity\n").unwrap();
    let error = KnowledgeBase::new(root.path()).entities().query(&[filter("P1", "Q43")], 1, 0).unwrap_err();
    assert!(error.to_string().contains("cannot parse entity"));
}

#[test]
fn query_rejects_filename_identifier_mismatches() {
    let root = tempfile::tempdir().unwrap();
    fs::create_dir(root.path().join("entities")).unwrap();
    write_entity(root.path(), "Q2.yaml", "Q1", " []");
    let error = KnowledgeBase::new(root.path()).entities().query(&[filter("P1", "Q43")], 1, 0).unwrap_err();
    assert!(error.to_string().contains("declares identifier Q1 instead of Q2"));
}
