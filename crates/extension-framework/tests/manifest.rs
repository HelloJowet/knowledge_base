use knowledge_base_extension_framework::bindings::ResolvedBindings;
use knowledge_base_extension_framework::contracts::{
    BindingDeclaration, BindingKey, BindingKind, ContractVersion, ExtensionDependency, ExtensionId, ExtensionMetadata, KnowledgeBaseExtension, OntologyRequirements,
};
use knowledge_base_extension_framework::error::FrameworkError;
use knowledge_base_extension_framework::manifest::{ExtensionManifest, ManifestDiagnostic, ManifestError};
use knowledge_base_extension_framework::registry::ExtensionRegistry;
use knowledge_base_validation::KnowledgeBaseValidator;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use tempfile::TempDir;

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

fn extension(name: &str, dependencies: &[&str], bindings: &[(&str, BindingKind)]) -> Arc<dyn KnowledgeBaseExtension> {
    Arc::new(TestExtension(ExtensionMetadata {
        id: id(name),
        contract: ContractVersion::new(1),
        dependencies: dependencies
            .iter()
            .map(|dependency| ExtensionDependency {
                id: id(dependency),
                contract: ContractVersion::new(1),
            })
            .collect(),
        bindings: bindings.iter().map(|(name, kind)| BindingDeclaration { key: key(name), kind: *kind }).collect(),
        ontology_requirements: OntologyRequirements::default(),
    }))
}

fn registry() -> ExtensionRegistry {
    ExtensionRegistry::new([
        extension("wikidata", &[], &[("item_id_property", BindingKind::Property)]),
        extension(
            "public-transport",
            &["wikidata"],
            &[("stop_place_type", BindingKind::EntityType), ("route_line_property", BindingKind::Property)],
        ),
    ])
    .unwrap()
}

fn write_manifest(root: &Path, source: &str) {
    fs::write(root.join("extensions.yaml"), source).unwrap();
}

fn repository() -> TempDir {
    let root = tempfile::tempdir().unwrap();
    for directory in ["entities", "entity_types", "properties", "references"] {
        fs::create_dir(root.path().join(directory)).unwrap();
    }
    fs::write(
        root.path().join("id_allocation.yaml"),
        "version: 1\nnext:\n  entity: 1\n  property: 2\n  reference: 1\n  entity_type: 2\n",
    )
    .unwrap();
    fs::write(root.path().join("entity_types/T1.yaml"), "id: T1\nlabels: {}\n").unwrap();
    fs::write(
        root.path().join("properties/P1.yaml"),
        "id: P1\nlabels: {}\nsubject_types: []\nvalue_type: string\nusage: statement\n",
    )
    .unwrap();
    root
}

#[test]
fn parses_base_only_and_composed_manifests_in_stable_order() {
    let base: ExtensionManifest = serde_yaml::from_str("version: 1\nextensions: {}\n").unwrap();
    assert!(base.extensions.is_empty());

    let composed: ExtensionManifest = serde_yaml::from_str(
        "version: 1\nextensions:\n  public-transport:\n    contract: 1\n    properties:\n      route_line_property: P1\n    entity_types:\n      stop_place_type: T1\n  wikidata:\n    contract: 1\n    properties:\n      item_id_property: P1\n",
    )
    .unwrap();
    assert_eq!(composed.extensions.keys().map(ToString::to_string).collect::<Vec<_>>(), ["public-transport", "wikidata"]);
    let serialized = serde_yaml::to_string(&composed).unwrap();
    assert!(serialized.find("public-transport:").unwrap() < serialized.find("wikidata:").unwrap());
    assert!(serialized.find("entity_types:").unwrap() < serialized.find("properties:").unwrap());
}

#[test]
fn rejects_missing_and_malformed_manifests() {
    let root = tempfile::tempdir().unwrap();
    assert!(matches!(ExtensionManifest::load(root.path()), Err(ManifestError::Read { .. })));

    for source in [
        "version: 1\nunknown: true\nextensions: {}\n",
        "version: 1\nextensions:\n  PublicTransport:\n    contract: 1\n",
        "version: 1\nextensions:\n  wikidata:\n    contract: 1\n    properties:\n      item_id_property: T1\n",
        "version: 1\nextensions:\n  public-transport:\n    contract: 1\n    entity_types:\n      stop_place_type: P1\n",
    ] {
        write_manifest(root.path(), source);
        assert!(matches!(ExtensionManifest::load(root.path()), Err(ManifestError::Parse { .. })), "{source}");
    }

    write_manifest(root.path(), "version: 2\nextensions: {}\n");
    assert!(
        matches!(ExtensionManifest::load_and_activate(root.path(), &registry()), Err(ManifestError::Diagnostics { diagnostics, .. }) if diagnostics == vec![ManifestDiagnostic::UnsupportedVersion { version: 2 }])
    );
}

#[test]
fn reports_ordered_activation_problems() {
    let root = tempfile::tempdir().unwrap();
    write_manifest(
        root.path(),
        "version: 1\nextensions:\n  aaa:\n    contract: 1\n  public-transport:\n    contract: 2\n    entity_types:\n      route_line_property: T1\n      unknown_type: T1\n    properties:\n      stop_place_type: P1\n  wikidata:\n    contract: 1\n    properties: {}\n",
    );
    let error = ExtensionManifest::load_and_activate(root.path(), &registry()).unwrap_err();
    let ManifestError::Diagnostics { diagnostics, .. } = error else {
        panic!("expected aggregate diagnostics");
    };
    assert_eq!(
        diagnostics,
        vec![
            ManifestDiagnostic::UnavailableExtension { extension: id("aaa") },
            ManifestDiagnostic::UnsupportedContract {
                extension: id("public-transport"),
                declared: ContractVersion::new(2),
                available: ContractVersion::new(1),
            },
            ManifestDiagnostic::BindingKindMismatch {
                binding: "public-transport:route_line_property".parse().unwrap(),
                expected: BindingKind::Property,
                actual: BindingKind::EntityType,
            },
            ManifestDiagnostic::UndeclaredBinding {
                binding: "public-transport:unknown_type".parse().unwrap()
            },
            ManifestDiagnostic::BindingKindMismatch {
                binding: "public-transport:stop_place_type".parse().unwrap(),
                expected: BindingKind::EntityType,
                actual: BindingKind::Property,
            },
            ManifestDiagnostic::MissingBinding {
                binding: "public-transport:route_line_property".parse().unwrap()
            },
            ManifestDiagnostic::MissingBinding {
                binding: "public-transport:stop_place_type".parse().unwrap()
            },
            ManifestDiagnostic::MissingBinding {
                binding: "wikidata:item_id_property".parse().unwrap()
            },
        ]
    );
}

#[test]
fn requires_declared_dependencies_and_resolves_existing_ontology_records() {
    let root = repository();
    write_manifest(
        root.path(),
        "version: 1\nextensions:\n  public-transport:\n    contract: 1\n    entity_types:\n      stop_place_type: T1\n    properties:\n      route_line_property: P1\n",
    );
    let error = ExtensionManifest::load_and_activate(root.path(), &registry()).unwrap_err();
    assert!(
        matches!(error, ManifestError::Diagnostics { diagnostics, .. } if diagnostics == vec![ManifestDiagnostic::MissingDependency { extension: id("public-transport"), dependency: id("wikidata") }])
    );

    write_manifest(
        root.path(),
        "version: 1\nextensions:\n  public-transport:\n    contract: 1\n    entity_types:\n      stop_place_type: T1\n    properties:\n      route_line_property: P1\n  wikidata:\n    contract: 1\n    properties:\n      item_id_property: P1\n",
    );
    let activation = ExtensionManifest::load_and_activate(root.path(), &registry()).unwrap();
    assert_eq!(
        activation
            .active()
            .extensions()
            .iter()
            .map(|extension| extension.metadata().id.to_string())
            .collect::<Vec<_>>(),
        ["wikidata", "public-transport"]
    );

    write_manifest(
        root.path(),
        "version: 1\nextensions:\n  wikidata:\n    contract: 1\n    properties:\n      item_id_property: P99\n",
    );
    let error = ExtensionManifest::load_and_activate(root.path(), &registry()).unwrap_err();
    let ManifestError::Diagnostics { diagnostics, .. } = error else {
        panic!("expected missing ontology binding diagnostic");
    };
    assert!(matches!(
        diagnostics.as_slice(),
        [ManifestDiagnostic::MissingProperty { binding, id }]
            if binding.to_string() == "wikidata:item_id_property" && id.to_string() == "P99"
    ));
}
