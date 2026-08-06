# knowledge-base-models

Serde models and typed identifiers for knowledge-base YAML files.

```rust
use knowledge_base_models::EntityId;

let id: EntityId = serde_yaml::from_str("Q42")?;
assert_eq!(id.as_str(), "Q42");
# Ok::<(), serde_yaml::Error>(())
```

See the [data model](../../docs/data-model.md) for the complete file shapes.
