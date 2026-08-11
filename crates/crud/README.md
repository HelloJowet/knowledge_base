# knowledge-base-crud

Typed filesystem access for a file-based knowledge base. Operations are grouped by resource while `KnowledgeBase` remains the entry point:

```rust
use knowledge_base_crud::{ApplyMode, KnowledgeBase, StatementBatch};
use knowledge_base_models::EntityId;

let knowledge_base = KnowledgeBase::new("/path/to/knowledge-base");
let entity_id = "Q1".parse::<EntityId>()?;
let source = knowledge_base.entities().read(&entity_id)?;

let batch = StatementBatch::read("/tmp/statement-manifest.yaml")?;
let outcome = knowledge_base
    .entities()
    .apply_statements(&batch, ApplyMode::Preview)?;
# Ok::<(), Box<dyn std::error::Error>>(())
```

Equivalent read services are available through `entity_types()`, `properties()`, `references()`, and `entity_contexts()`. Reads return files exactly as stored.

Mutations use a shared `.knowledge-base.lock`, validate a staged repository, detect changes made after planning, and attempt rollback if a later replacement fails. Multi-file mutations are not crash-atomic.
