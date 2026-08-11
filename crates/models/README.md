# knowledge-base-models

Serde models and typed identifiers for knowledge-base YAML files.

The crate exposes the record types, typed identifiers, localized text, property definitions, and value types that make up the canonical 0.2.0 schema. It intentionally does not deserialize the legacy transit-owned schema; use the repository [migration guide](../../MIGRATION.md) before adopting these types for existing data.

```rust
use knowledge_base_models::EntityId;

let id: EntityId = serde_yaml::from_str("Q42")?;
assert_eq!(id.as_str(), "Q42");
# Ok::<(), serde_yaml::Error>(())
```

See the [data model](../../docs/data-model.md) for the complete file shapes.
