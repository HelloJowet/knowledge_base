use knowledge_base_extension_framework::bindings::{BindingValue, ResolvedBindings};
use knowledge_base_extension_framework::contracts::{
    BindingDeclaration, BindingKey, BindingKind, BindingReference, ContractVersion, EntityTypeRequirement, ExtensionDependency, ExtensionId, ExtensionMetadata,
    KnowledgeBaseExtension, OntologyRequirements,
};
use knowledge_base_extension_framework::error::FrameworkError;
use knowledge_base_extension_framework::registry::ExtensionRegistry;
use knowledge_base_validation::KnowledgeBaseValidator;
use std::collections::BTreeMap;
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
fn reference(extension: &str, binding: &str) -> BindingReference {
    BindingReference::new(id(extension), key(binding))
}

fn extension(name: &str, dependencies: &[(&str, u32)], bindings: &[(&str, BindingKind)]) -> Arc<dyn KnowledgeBaseExtension> {
    Arc::new(TestExtension(ExtensionMetadata {
        id: id(name),
        contract: ContractVersion::new(1),
        dependencies: dependencies
            .iter()
            .map(|(dependency, version)| ExtensionDependency {
                id: id(dependency),
                contract: ContractVersion::new(*version),
            })
            .collect(),
        bindings: bindings.iter().map(|(binding, kind)| BindingDeclaration { key: key(binding), kind: *kind }).collect(),
        ontology_requirements: OntologyRequirements::default(),
    }))
}

#[test]
fn identifiers_and_references_are_canonical() {
    const STATIC_EXTENSION: ExtensionId = ExtensionId::from_static("wikidata");
    const STATIC_KEY: BindingKey = BindingKey::from_static("item_id_property");
    const STATIC_REFERENCE: BindingReference = BindingReference::from_static("wikidata", "item_id_property");

    for valid in ["wikidata", "public-transport", "v1"] {
        assert!(valid.parse::<ExtensionId>().is_ok());
    }
    for invalid in ["", "Wikidata", "public_transport", "public--transport", "1transport", "-transport", "transport-"] {
        assert!(invalid.parse::<ExtensionId>().is_err(), "{invalid} accepted");
    }
    for valid in ["item_id_property", "v1"] {
        assert!(valid.parse::<BindingKey>().is_ok());
    }
    for invalid in ["item-id", "Item_id", "1_item", "item__id"] {
        assert!(invalid.parse::<BindingKey>().is_err(), "{invalid} accepted");
    }
    assert_eq!("wikidata:item_id_property".parse::<BindingReference>().unwrap().to_string(), "wikidata:item_id_property");
    assert_eq!(STATIC_EXTENSION, "wikidata".parse().unwrap());
    assert_eq!(STATIC_KEY, "item_id_property".parse().unwrap());
    assert_eq!(STATIC_REFERENCE, "wikidata:item_id_property".parse().unwrap());
}

#[test]
fn registry_validates_metadata_and_dependency_graphs() {
    let duplicate = ExtensionRegistry::new([extension("wikidata", &[], &[]), extension("wikidata", &[], &[])])
        .err()
        .expect("duplicate registration must fail");
    assert_eq!(duplicate, FrameworkError::DuplicateExtension(id("wikidata")));

    let missing = ExtensionRegistry::new([extension("public-transport", &[("wikidata", 1)], &[])])
        .err()
        .expect("missing dependency must fail");
    assert!(matches!(missing, FrameworkError::MissingDependency { .. }));
    let version = ExtensionRegistry::new([extension("public-transport", &[("wikidata", 2)], &[]), extension("wikidata", &[], &[])])
        .err()
        .expect("contract mismatch must fail");
    assert!(matches!(version, FrameworkError::UnsupportedContract { .. }));

    let cycle = ExtensionRegistry::new([extension("a", &[("b", 1)], &[]), extension("b", &[("a", 1)], &[])]).unwrap();
    assert!(matches!(cycle.resolve_active([id("a"), id("b")]), Err(FrameworkError::DependencyCycle(_))));
}

#[test]
fn registry_resolves_active_extensions_in_dependency_first_order() {
    let registry = ExtensionRegistry::new([
        extension("public-transport", &[("wikidata", 1)], &[]),
        extension("base-data", &[], &[]),
        extension("wikidata", &[("base-data", 1)], &[]),
    ])
    .unwrap();
    let active = registry.resolve_active([id("wikidata"), id("base-data"), id("public-transport")]).unwrap();
    assert_eq!(
        active.extensions().iter().map(|item| item.metadata().id.to_string()).collect::<Vec<_>>(),
        ["base-data", "wikidata", "public-transport"]
    );
    assert!(matches!(registry.resolve_active([id("public-transport")]), Err(FrameworkError::InactiveDependency { .. })));
}

#[test]
fn bindings_require_complete_matching_values_and_declared_access() {
    let registry = ExtensionRegistry::new([
        extension("wikidata", &[], &[("item_id_property", BindingKind::Property)]),
        extension("public-transport", &[("wikidata", 1)], &[("stop_place_type", BindingKind::EntityType)]),
        extension("other", &[], &[("other_type", BindingKind::EntityType)]),
    ])
    .unwrap();
    let active = registry.resolve_active([id("wikidata"), id("public-transport")]).unwrap();

    let missing = active
        .resolve_bindings(BTreeMap::from([(reference("wikidata", "item_id_property"), BindingValue::Property("P1".parse().unwrap()))]))
        .unwrap_err();
    assert!(matches!(missing, FrameworkError::MissingBinding(_)));
    let wrong_kind = active
        .resolve_bindings(BTreeMap::from([
            (reference("wikidata", "item_id_property"), BindingValue::EntityType("T1".parse().unwrap())),
            (reference("public-transport", "stop_place_type"), BindingValue::EntityType("T2".parse().unwrap())),
        ]))
        .unwrap_err();
    assert!(matches!(wrong_kind, FrameworkError::BindingKindMismatch { .. }));

    let bindings = active
        .resolve_bindings(BTreeMap::from([
            (reference("wikidata", "item_id_property"), BindingValue::Property("P1".parse().unwrap())),
            (reference("public-transport", "stop_place_type"), BindingValue::EntityType("T1".parse().unwrap())),
        ]))
        .unwrap();
    let transport = active.extensions().last().unwrap().metadata();
    assert_eq!(bindings.entity_type(transport, &reference("public-transport", "stop_place_type")).unwrap().as_str(), "T1");
    assert_eq!(bindings.property(transport, &reference("wikidata", "item_id_property")).unwrap().as_str(), "P1");
    assert!(matches!(
        bindings.get(transport, &reference("other", "other_type")),
        Err(FrameworkError::InaccessibleBinding { .. })
    ));
}

#[test]
fn requirements_must_reference_matching_declared_binding_kinds() {
    let extension: Arc<dyn KnowledgeBaseExtension> = Arc::new(TestExtension(ExtensionMetadata {
        id: id("example"),
        contract: ContractVersion::new(1),
        dependencies: vec![],
        bindings: vec![BindingDeclaration {
            key: key("property"),
            kind: BindingKind::Property,
        }],
        ontology_requirements: OntologyRequirements {
            entity_types: vec![EntityTypeRequirement {
                binding: reference("example", "property"),
            }],
            properties: vec![],
        },
    }));
    assert!(matches!(ExtensionRegistry::new([extension]), Err(FrameworkError::InvalidRequirement { .. })));
}
