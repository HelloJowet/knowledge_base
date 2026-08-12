use knowledge_base_extension_framework::bindings::ResolvedBindings;
use knowledge_base_extension_framework::contracts::{
    BindingDeclaration, BindingKey, BindingKind, BindingReference, ContractVersion, ExtensionId, ExtensionMetadata, KnowledgeBaseExtension, OntologyRequirements,
    PropertyRequirement,
};
use knowledge_base_extension_framework::error::FrameworkError;
use knowledge_base_extension_framework::manifest::{ExtensionManifest, ManifestError};
use knowledge_base_extension_framework::ontology::OntologyContractField;
use knowledge_base_extension_framework::registry::ExtensionRegistry;
use knowledge_base_models::{Cardinality, PropertyUsage, ValueType};
use knowledge_base_validation::KnowledgeBaseValidator;
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;
use std::sync::Arc;

struct TestExtension(ExtensionMetadata);

impl KnowledgeBaseExtension for TestExtension {
    fn metadata(&self) -> &ExtensionMetadata {
        &self.0
    }

    fn validators(&self, _: &ResolvedBindings) -> Result<Vec<Arc<dyn KnowledgeBaseValidator>>, FrameworkError> {
        Ok(Vec::new())
    }
}

fn id(value: &str) -> ExtensionId {
    value.parse().unwrap()
}

fn key(value: &str) -> BindingKey {
    value.parse().unwrap()
}

fn binding(extension: &str, name: &str) -> BindingReference {
    BindingReference::new(id(extension), key(name))
}

fn set(values: impl IntoIterator<Item = BindingReference>) -> BTreeSet<BindingReference> {
    values.into_iter().collect()
}

fn extension(name: &str, bindings: &[(&str, BindingKind)], requirements: Vec<PropertyRequirement>) -> Arc<dyn KnowledgeBaseExtension> {
    Arc::new(TestExtension(ExtensionMetadata {
        id: id(name),
        contract: ContractVersion::new(1),
        dependencies: vec![],
        bindings: bindings.iter().map(|(name, kind)| BindingDeclaration { key: key(name), kind: *kind }).collect(),
        ontology_requirements: OntologyRequirements {
            entity_types: vec![],
            properties: requirements,
        },
    }))
}

fn requirement(extension: &str, property: &str) -> PropertyRequirement {
    PropertyRequirement {
        binding: binding(extension, property),
        value_type: None,
        usage: None,
        cardinality: None,
        subject_types: BTreeSet::new(),
        target_types: None,
        allowed_qualifiers: BTreeSet::new(),
    }
}

fn repository() -> tempfile::TempDir {
    let root = tempfile::tempdir().unwrap();
    for directory in ["entities", "entity_types", "properties", "references"] {
        fs::create_dir(root.path().join(directory)).unwrap();
    }
    fs::write(
        root.path().join("id_allocation.yaml"),
        "version: 1\nnext:\n  entity: 1\n  property: 100\n  reference: 1\n  entity_type: 100\n",
    )
    .unwrap();
    for type_id in ["T2", "T3", "T8", "T9"] {
        fs::write(root.path().join(format!("entity_types/{type_id}.yaml")), format!("id: {type_id}\nlabels: {{}}\n")).unwrap();
    }
    root
}

fn write_property(root: &Path, source: &str) {
    fs::write(root.join("properties/P42.yaml"), source).unwrap();
    fs::write(
        root.join("properties/P43.yaml"),
        "id: P43\nlabels: {}\nsubject_types: [T2]\nvalue_type: string\nusage: qualifier\n",
    )
    .unwrap();
    fs::write(
        root.join("properties/P44.yaml"),
        "id: P44\nlabels: {}\nsubject_types: [T2]\nvalue_type: string\nusage: qualifier\n",
    )
    .unwrap();
}

fn manifest(root: &Path, source: &str, registry: &ExtensionRegistry) -> Result<(), ManifestError> {
    fs::write(root.join("extensions.yaml"), source).unwrap();
    ExtensionManifest::load_and_activate(root, registry).map(|_| ())
}

#[test]
fn accepts_collection_supersets_and_unions_requirements_from_extensions() {
    let mut first = requirement("first", "property");
    first.subject_types = set([binding("first", "subject")]);
    first.target_types = Some(set([binding("first", "target")]));
    first.allowed_qualifiers = set([binding("first", "qualifier")]);
    let mut second = requirement("second", "property");
    second.subject_types = set([binding("second", "subject")]);
    second.target_types = Some(set([binding("second", "target")]));
    second.allowed_qualifiers = set([binding("second", "qualifier")]);
    let registry = ExtensionRegistry::new([
        extension(
            "first",
            &[
                ("property", BindingKind::Property),
                ("subject", BindingKind::EntityType),
                ("target", BindingKind::EntityType),
                ("qualifier", BindingKind::Property),
            ],
            vec![first],
        ),
        extension(
            "second",
            &[
                ("property", BindingKind::Property),
                ("subject", BindingKind::EntityType),
                ("target", BindingKind::EntityType),
                ("qualifier", BindingKind::Property),
            ],
            vec![second],
        ),
    ])
    .unwrap();
    let root = repository();
    write_property(
        root.path(),
        "id: P42\nlabels: {}\nsubject_types: [T2, T3, T9]\nvalue_type: entity\nusage: statement\ntarget_types: [T2, T3, T8]\nallowed_qualifiers: [P43, P44]\n",
    );

    manifest(
        root.path(),
        "version: 1\nextensions:\n  first:\n    contract: 1\n    entity_types: {subject: T2, target: T3}\n    properties: {property: P42, qualifier: P43}\n  second:\n    contract: 1\n    entity_types: {subject: T3, target: T2}\n    properties: {property: P42, qualifier: P44}\n",
        &registry,
    )
    .unwrap();
}

#[test]
fn reports_scalar_conflicts_against_the_resolved_fixture_property() {
    let mut first = requirement("first", "property");
    first.value_type = Some(ValueType::String);
    let mut second = requirement("second", "property");
    second.value_type = Some(ValueType::Integer);
    let registry = ExtensionRegistry::new([
        extension("first", &[("property", BindingKind::Property)], vec![first]),
        extension("second", &[("property", BindingKind::Property)], vec![second]),
    ])
    .unwrap();
    let root = repository();
    write_property(root.path(), "id: P42\nlabels: {}\nsubject_types: [T2]\nvalue_type: string\nusage: statement\n");

    let ManifestError::OntologyContracts { diagnostics, .. } = manifest(
        root.path(),
        "version: 1\nextensions:\n  first: {contract: 1, properties: {property: P42}}\n  second: {contract: 1, properties: {property: P42}}\n",
        &registry,
    )
    .unwrap_err() else {
        panic!("expected scalar conflict");
    };
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].binding.to_string(), "second:property");
    assert_eq!(diagnostics[0].id.to_string(), "P42");
    assert_eq!(diagnostics[0].field, OntologyContractField::ValueType);
}

#[test]
fn reports_deterministic_property_contract_mismatches() {
    let mut contract = requirement("fixture", "property");
    contract.value_type = Some(ValueType::Entity);
    contract.usage = Some(PropertyUsage::Qualifier);
    contract.cardinality = Some(Cardinality::One);
    contract.subject_types = set([binding("fixture", "subject")]);
    contract.target_types = Some(set([binding("fixture", "target")]));
    contract.allowed_qualifiers = set([binding("fixture", "qualifier")]);
    let registry = ExtensionRegistry::new([extension(
        "fixture",
        &[
            ("property", BindingKind::Property),
            ("subject", BindingKind::EntityType),
            ("target", BindingKind::EntityType),
            ("qualifier", BindingKind::Property),
        ],
        vec![contract],
    )])
    .unwrap();
    let root = repository();
    write_property(
        root.path(),
        "id: P42\nlabels: {}\nsubject_types: [T9]\nvalue_type: string\nusage: statement\nallowed_qualifiers: []\ncardinality: many\n",
    );

    let ManifestError::OntologyContracts { diagnostics, .. } = manifest(
        root.path(),
        "version: 1\nextensions:\n  fixture:\n    contract: 1\n    entity_types: {subject: T2, target: T3}\n    properties: {property: P42, qualifier: P43}\n",
        &registry,
    )
    .unwrap_err() else {
        panic!("expected ontology mismatches");
    };
    assert_eq!(
        diagnostics.iter().map(|diagnostic| diagnostic.field).collect::<Vec<_>>(),
        [
            OntologyContractField::ValueType,
            OntologyContractField::Usage,
            OntologyContractField::Cardinality,
            OntologyContractField::SubjectTypes,
            OntologyContractField::TargetTypes,
            OntologyContractField::AllowedQualifiers,
        ]
    );
    assert!(
        diagnostics
            .iter()
            .all(|diagnostic| diagnostic.binding.to_string() == "fixture:property" && diagnostic.id.to_string() == "P42")
    );
}

#[test]
fn rejects_a_missing_entity_target_type() {
    let mut contract = requirement("fixture", "property");
    contract.target_types = Some(set([binding("fixture", "target")]));
    let registry = ExtensionRegistry::new([extension(
        "fixture",
        &[("property", BindingKind::Property), ("target", BindingKind::EntityType)],
        vec![contract],
    )])
    .unwrap();
    let root = repository();
    write_property(
        root.path(),
        "id: P42\nlabels: {}\nsubject_types: [T2]\nvalue_type: entity\nusage: statement\ntarget_types: [T2]\n",
    );

    let ManifestError::OntologyContracts { diagnostics, .. } = manifest(
        root.path(),
        "version: 1\nextensions:\n  fixture:\n    contract: 1\n    entity_types: {target: T3}\n    properties: {property: P42}\n",
        &registry,
    )
    .unwrap_err() else {
        panic!("expected target type mismatch");
    };
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].field, OntologyContractField::TargetTypes);
    assert_eq!(diagnostics[0].message, "missing required entry T3");
}
