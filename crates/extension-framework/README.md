# Knowledge-base extension framework

This crate is the small layer used to add domain-specific rules to a knowledge base. An extension is compiled into the program; a repository then chooses which compiled extensions to use.

The main benefit is portability: extension code refers to meaningful names such as `wikidata:item_id_property`, while each repository supplies the real ontology ID (for example, `P21`).

## The modules

- `contracts` — describe an extension: its ID, dependencies, bindings, and validators.
- `registry` — check compiled extensions and activate the requested set.
- `bindings` — connect an extension's binding names to real entity-type and property IDs.
- `manifest` — strict, versioned repository configuration and activation.
- `error` — errors returned while composing extensions.

Import types from the module that owns them:

```rust
use knowledge_base_extension_framework::manifest::ExtensionManifest;
use knowledge_base_extension_framework::registry::ExtensionRegistry;
```

## A typical flow

First, register the extensions built into the program. A repository activates the compiled extensions named in its required `extensions.yaml`; dependencies must be included explicitly.

```rust
let registry = ExtensionRegistry::new(compiled_extensions)?;
let activation = ExtensionManifest::load_and_activate(repository_root, &registry)?;
```

The manifest gives each declared binding its repository-specific ID. The framework checks that every active binding is present, has the right kind, and resolves to a canonical ontology record.

```yaml
version: 1
extensions:
  wikidata:
    contract: 1
    properties:
      item_id_property: P21
```

Extensions can read their own bindings and bindings provided by a direct dependency. This keeps their dependencies obvious and prevents accidental coupling to unrelated extensions.

## What this crate does not do

It does not add CLI commands or modify ontology records. Manifest editing remains manual and reviewable.
