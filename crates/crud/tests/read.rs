use knowledge_base_crud::KnowledgeBase;
use knowledge_base_models::{EntityId, EntityTypeId, PropertyId, ReferenceId};
use std::fs;
use std::path::{Path, PathBuf};

fn fixture() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..").join("fixtures/valid/minimal")
}

#[test]
fn reads_every_supported_resource_without_normalizing_it() {
    let root = fixture();
    let knowledge_base = KnowledgeBase::new(&root);

    let entity_id = "Q1".parse::<EntityId>().expect("valid entity identifier");
    let type_id = "T1".parse::<EntityTypeId>().expect("valid entity type identifier");
    let property_id = "P1".parse::<PropertyId>().expect("valid property identifier");
    let reference_id = "R1".parse::<ReferenceId>().expect("valid reference identifier");

    assert_eq!(
        knowledge_base.read_entity(&entity_id).expect("read entity"),
        fs::read_to_string(root.join("entities/Q1.yaml")).unwrap()
    );
    assert_eq!(
        knowledge_base.read_entity_type(&type_id).expect("read type"),
        fs::read_to_string(root.join("entity_types/T1.yaml")).unwrap()
    );
    assert_eq!(
        knowledge_base.read_property(&property_id).expect("read property"),
        fs::read_to_string(root.join("properties/P1.yaml")).unwrap()
    );
    assert_eq!(
        knowledge_base.read_reference(&reference_id).expect("read reference"),
        fs::read_to_string(root.join("references/R1.yaml")).unwrap()
    );
    assert_eq!(
        knowledge_base.read_entity_context(&entity_id).expect("read context"),
        fs::read_to_string(root.join("entity_context/Q1.md")).unwrap()
    );
}

#[test]
fn missing_resource_error_contains_the_resolved_path() {
    let root = fixture();
    let knowledge_base = KnowledgeBase::new(&root);
    let id = "Q999".parse::<EntityId>().expect("valid entity identifier");

    let error = knowledge_base.read_entity(&id).expect_err("resource should be missing");

    assert_eq!(error.path, root.join("entities/Q999.yaml"));
    assert!(error.to_string().contains("Q999.yaml"));
}
