# knowledge-base-crud

`knowledge-base-crud` is the Rust API for reading and updating a file-based knowledge base. Start with `KnowledgeBaseRepository`, then use `read()` for data access and `write()` for changes.

## Read data

Read individual resource files when you need their stored contents, or load a snapshot for a typed view of the whole repository.

```rust
use knowledge_base_crud::KnowledgeBaseRepository;
use knowledge_base_models::EntityId;

let repository = KnowledgeBaseRepository::new("/path/to/knowledge-base");
let id: EntityId = "Q1".parse()?;

let source = repository.read().entities().read(&id)?;
let snapshot = repository.read().snapshot()?;
let entity = snapshot.entities().get(&id);
# let _ = (source, entity);
# Ok::<(), Box<dyn std::error::Error>>(())
```

The other resource collections are available through `entity_types()`, `properties()`, `references()`, and `entity_contexts()`. Entity queries use one or more typed property/value filters and return entities that match every filter.

## Make changes

Use a `StatementBatch` to add statements from a YAML manifest, or register a new reference with `ReferenceDraft`. Choose `WriteMode::Preview` to check a change without writing it; use `WriteMode::Commit` to apply it.

```rust
use knowledge_base_crud::write::{StatementBatch, WriteMode};

let repository = knowledge_base_crud::KnowledgeBaseRepository::new("/path/to/knowledge-base");
let batch = StatementBatch::read("/tmp/statements.yaml")?;
let result = repository.write().statements().apply(&batch, WriteMode::Preview)?;
# let _ = result;
# Ok::<(), Box<dyn std::error::Error>>(())
```

Changes are validated before they are written. The crate also uses a repository lock to prevent concurrent updates. For domain-specific rules, create the repository with validators from [`knowledge-base-validation`](../validation/README.md).
