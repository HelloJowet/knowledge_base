use knowledge_base_crud::{KnowledgeBase, RelationshipDirection};
use knowledge_base_models::EntityId;
use std::fs;

fn write_entity(root: &std::path::Path, id: &str, label: &str, statements: &str) {
    fs::write(
        root.join("entities").join(format!("{id}.yaml")),
        format!("id: {id}\nlabels:\n  en:\n    text: {label}\n    references: [R1]\nentity_types:\n  - value: T1\n    references: [R1]\nstatements:{statements}\n"),
    )
    .unwrap();
}

#[test]
fn relationship_service_sorts_then_pages_all_direct_edge_directions() {
    let root = tempfile::tempdir().unwrap();
    fs::create_dir(root.path().join("entities")).unwrap();
    write_entity(
        root.path(),
        "Q1",
        "Root",
        "\n  - id: S10\n    property: P2\n    value: { type: entity, value: Q2 }\n    references: [R1]\n  - id: S2\n    property: P1\n    value: { type: entity, value: Q1 }\n    references: [R1]",
    );
    write_entity(root.path(), "Q2", "Parent", " []");
    write_entity(
        root.path(),
        "Q3",
        "Child",
        "\n  - id: S1\n    property: P3\n    value: { type: entity, value: Q1 }\n    references: [R1]",
    );

    let knowledge_base = KnowledgeBase::new(root.path());
    let id = "Q1".parse::<EntityId>().unwrap();
    let page = knowledge_base.entities().relationships(&id, 2, 1).unwrap();

    assert_eq!(page.entity, id);
    assert_eq!(page.offset, 1);
    assert_eq!(page.limit, 2);
    assert_eq!(page.total, 3);
    assert_eq!(page.next_offset, None);
    assert_eq!(page.relationships.len(), 2);
    assert_eq!(page.relationships[0].direction, RelationshipDirection::Outgoing);
    assert_eq!(page.relationships[0].entity.id.as_str(), "Q2");
    assert_eq!(page.relationships[0].entity.labels.get("en").map(String::as_str), Some("Parent"));
    assert_eq!(page.relationships[1].direction, RelationshipDirection::Incoming);
    assert_eq!(page.relationships[1].entity.id.as_str(), "Q3");

    let first = knowledge_base.entities().relationships(&id, 1, 0).unwrap();
    assert_eq!(first.next_offset, Some(1));
    assert_eq!(first.relationships[0].direction, RelationshipDirection::SelfReference);
    assert_eq!(serde_yaml::to_string(&first.relationships[0].direction).unwrap(), "self\n");

    let empty = knowledge_base.entities().relationships(&id, 5, 99).unwrap();
    assert_eq!(empty.total, 3);
    assert!(empty.relationships.is_empty());
    assert_eq!(empty.next_offset, None);
}

#[test]
fn relationship_service_rejects_a_zero_limit() {
    let root = tempfile::tempdir().unwrap();
    fs::create_dir(root.path().join("entities")).unwrap();
    write_entity(root.path(), "Q1", "Root", " []");
    let id = "Q1".parse::<EntityId>().unwrap();

    let error = KnowledgeBase::new(root.path()).entities().relationships(&id, 0, 0).unwrap_err();

    assert!(error.to_string().contains("limit must be greater than zero"));
}
