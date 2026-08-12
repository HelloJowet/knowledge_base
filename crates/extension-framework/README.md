# Knowledge-base extension framework

This crate lets an application add domain-specific rules to a knowledge base without putting those rules in the generic knowledge-base crates.

Extensions are Rust objects compiled into an application. Each repository chooses which available extensions to activate in its `extensions.yaml` file. This is static composition, not runtime plugin loading.

## Why semantic bindings exist

An extension should not depend on repository-specific ontology IDs such as `P21` or `T3`. Instead, it declares meaningful binding names such as `wikidata:item_id_property`, and each repository maps those names to its own ontology records.

This allows the same extension to work with different repositories and test fixtures.

## How it works

1. The application registers the extensions compiled into it.
2. The repository's `extensions.yaml` selects extensions and maps their bindings to ontology IDs.
3. The framework checks extension versions, dependencies, bindings, and ontology requirements.
4. The application uses the active extensions and their validators.

```rust
use knowledge_base_extension_framework::manifest::ExtensionManifest;
use knowledge_base_extension_framework::registry::ExtensionRegistry;

let registry = ExtensionRegistry::new(compiled_extensions)?;
let activation = ExtensionManifest::load_and_activate(repository_root, &registry)?;
```

An extension entry in `extensions.yaml` looks like this:

```yaml
version: 1
extensions:
  wikidata:
    contract: 1
    properties:
      item_id_property: P21
```

Dependencies are declared by extension code, but every dependency must also be enabled in `extensions.yaml`.

## Extension interface

One concrete extension object implements `KnowledgeBaseExtension`. It provides metadata and can create validators from resolved bindings. Extensions without custom validators use the default empty validator set.

Extensions that add CLI commands implement `knowledge_base_cli::KnowledgeBaseCliExtension` on the same object. Keeping this as a separate trait prevents the framework from depending on Clap.

`ExtensionMetadata` is the single source of truth for an extension's ID, contract version, dependencies, bindings, and ontology requirements.

## Main modules

- `contracts` defines extension metadata, semantic bindings, ontology requirements, and the core extension trait.
- `registry` validates compiled extensions and resolves the active set.
- `bindings` contains the repository-specific ontology IDs resolved for active extensions.
- `manifest` loads and validates `extensions.yaml`.
- `ontology` verifies active extension requirements against repository ontology records.
- `error` contains framework errors.

## Boundaries

The framework does not dynamically load extensions, edit `extensions.yaml`, or create and modify ontology records. These operations remain explicit and reviewable.
