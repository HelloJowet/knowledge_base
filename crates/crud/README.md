# knowledge-base-crud

Typed filesystem access for a file-based knowledge base. `knowledge-base-crud` is the filesystem-backed CRUD layer; its public API will expand to complete CRUD for every resource. This release exposes the read, query, and mutation operations documented below. Operations are grouped by resource while `KnowledgeBase` remains the entry point:

The crate operates only on the canonical 0.2.0 schema and does not migrate or parse the legacy transit-owned schema. Migrate existing repositories before using it; see the repository [migration guide](../../MIGRATION.md).

```rust
use knowledge_base_crud::{ApplyMode, EntityFilter, KnowledgeBase, ReferenceDraft, StatementBatch};
use knowledge_base_models::{EntityId, Value};

let knowledge_base = KnowledgeBase::new("/path/to/knowledge-base");
let entity_id = "Q1".parse::<EntityId>()?;
let source = knowledge_base.entities().read(&entity_id)?;

let filters = [EntityFilter {
    property: "P3".parse()?,
    value: Value::Entity { value: "Q2".parse()? },
}];
let matches = knowledge_base.entities().query(&filters, 100, 0)?;

let batch = StatementBatch::read("/tmp/statement-manifest.yaml")?;
let outcome = knowledge_base
    .entities()
    .apply_statements(&batch, ApplyMode::Preview)?;
let reference = knowledge_base.references().register(
    &ReferenceDraft {
        url: "https://example.org/source".to_owned(),
        title: "Example source".to_owned(),
        publisher: None,
        publication_date: None,
        source_language: Some("en".to_owned()),
        retrieved_at: "2026-08-11T12:00:00Z".to_owned(),
        archive_url: None,
    },
    ApplyMode::Preview,
)?;
# Ok::<(), Box<dyn std::error::Error>>(())
```

Statement manifests may include optional qualifiers (`property` plus a typed `value`). Repeating the same qualifier property/value within one statement is rejected. Duplicate detection compares the entity, property, typed value, and qualifiers without considering qualifier order or references.

Equivalent read services are available through `entity_types()`, `properties()`, `references()`, and `entity_contexts()`. Reads return files exactly as stored.

For a typed, read-only view of all structured resources, load a repository snapshot:

```rust
let snapshot = knowledge_base.snapshot()?;
let entity = snapshot.entities().get(&entity_id);
let allocation = snapshot.allocation();
# let _ = (entity, allocation);
# Ok::<(), Box<dyn std::error::Error>>(())
```

Snapshots index resources by ID in deterministic order and reject malformed repository structure. They do not run generic or domain semantic validation; call `knowledge_base.validate()` when that is required.

`entities().query()` requires at least one typed property/value filter and applies AND semantics to top-level statements. It scans the entity directory, sorts matches by numeric ID, and returns complete parsed entities with offset pagination metadata.

Mutations use a shared `.knowledge-base.lock`, validate a staged repository, detect changes made after planning, and attempt rollback if a later replacement fails. Multi-file mutations are not crash-atomic.

## Domain validators

A domain CLI can make its rules mandatory for both direct validation and every mutation by constructing `KnowledgeBase` with additional validators:

```rust
use knowledge_base_crud::KnowledgeBase;
use knowledge_base_validation::{AdditionalValidator, Diagnostic, ValidationContext, ValidationLayer};
use std::path::PathBuf;
use std::sync::Arc;

struct TransportValidator;

impl AdditionalValidator for TransportValidator {
    fn validate(&self, context: &ValidationContext<'_>) -> Vec<Diagnostic> {
        let _entity = context.snapshot().entities().get(&"Q1".parse().unwrap());
        vec![Diagnostic {
            layer: ValidationLayer::Domain,
            path: PathBuf::from("entities/Q1.yaml"),
            line: None,
            identifier: Some("Q1".to_owned()),
            message: "example transport rule".to_owned(),
        }]
    }
}

let knowledge_base = KnowledgeBase::with_additional_validators(
    "/path/to/knowledge-base",
    [Arc::new(TransportValidator)],
);
let diagnostics = knowledge_base.validate();
# Ok::<(), Box<dyn std::error::Error>>(())
```

Configured validators run in registration order against the locked baseline and the complete staged result. Each pass supplies every validator one shared immutable snapshot; validators are skipped when that snapshot cannot be loaded and generic schema diagnostics already describe the failure. Any diagnostic rejects the mutation before files change; preview mode uses the same staged validation path. The generic `knowledge-base` CLI does not configure domain validators—domain CLIs must create their own configured `KnowledgeBase` instance.

`references().register()` accepts an ID-less `ReferenceDraft`. It reuses a reference only when its stored URL exactly equals the draft URL; otherwise it allocates the next `R<n>` identifier and updates `id_allocation.yaml` in the same mutation. Draft metadata is validated before the lock is acquired, and both the baseline and fully staged repositories are validated under the lock. Preview mode performs the same checks without writing files.
