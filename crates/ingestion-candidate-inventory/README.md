# knowledge-base-ingestion-candidate-inventory

Model and validate the review inventory created while ingesting one source article. An inventory records evidence, proposed entities and statements, draft vocabulary, and review outcomes without changing production data.

## What it provides

`IngestionCandidateInventory` is the strict YAML model for the handoff. `validate_ingestion_candidate_inventory` checks the file against a `RepositorySnapshot` of the production knowledge base. It resolves existing identifiers and verifies references, dependencies, property compatibility, values, and optional summary counts.

## Usage

Load the production knowledge base and validate an inventory against it:

```rust
use std::path::Path;

use knowledge_base_ingestion_candidate_inventory::validate_ingestion_candidate_inventory;
use knowledge_base_snapshot::RepositorySnapshot;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let snapshot = RepositorySnapshot::load(Path::new("knowledge_base"))?;

    let diagnostics = validate_ingestion_candidate_inventory(
        Path::new("temp/retrievals/fetch-XXXXXX/ingestion_candidate_inventory.yaml"),
        &snapshot,
    );
    assert!(diagnostics.is_empty());
    Ok(())
}
```

## File contract

The inventory filename must be `ingestion_candidate_inventory.yaml`. Its `source_file` must be `page.html`, and that file must be beside the inventory. Unknown YAML fields are rejected at every level.

The inventory is authored outside this crate by a person or an AI agent. Use it as a reviewable handoff between source retrieval and production updates.

For the complete workflow, see the [ingestion guide](../../docs/ingestion.md).
