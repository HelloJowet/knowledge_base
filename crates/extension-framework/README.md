# Knowledge-base extension framework

This crate is the small layer used to add domain-specific rules to a knowledge base. An extension is compiled into the program; a repository then chooses which compiled extensions to use.

The main benefit is portability: extension code refers to meaningful names such as `wikidata:item_id_property`, while each repository supplies the real ontology ID (for example, `P21`).

## The four modules

- `contracts` — describe an extension: its ID, dependencies, bindings, and validators.
- `registry` — check compiled extensions and activate the requested set.
- `bindings` — connect an extension's binding names to real entity-type and property IDs.
- `error` — errors returned while parsing or composing extensions.

Import types from the module that owns them:

```rust
use knowledge_base_extension_framework::bindings::BindingValue;
use knowledge_base_extension_framework::contracts::{BindingReference, ExtensionId};
use knowledge_base_extension_framework::registry::ExtensionRegistry;
```

## A typical flow

First, register the extensions built into the program and activate the ones requested by a repository. Dependencies must be included explicitly.

```rust
let registry = ExtensionRegistry::new(compiled_extensions)?;
let active = registry.resolve_active([
    "wikidata".parse::<ExtensionId>()?,
    "public-transport".parse::<ExtensionId>()?,
])?;
```

Next, give each declared binding its repository-specific ID. The framework checks that every active binding is present and has the right kind.

```rust
use std::collections::BTreeMap;

let bindings = active.resolve_bindings(BTreeMap::from([(
    "wikidata:item_id_property".parse::<BindingReference>()?,
    BindingValue::Property("P21".parse()?),
)]))?;
```

Extensions can read their own bindings and bindings provided by a direct dependency. This keeps their dependencies obvious and prevents accidental coupling to unrelated extensions.

## What this crate does not do

It does not read `extensions.yaml`, add CLI commands, or modify ontology records. Those responsibilities belong to the application layer that uses this crate.
