# Ingestion workflow

Ingestion turns a webpage into reviewable material for a file-based knowledge base. It keeps retrieved source material and review work separate from production records until you decide what to apply.

## 1. Retrieve the page

```sh
knowledge-base ingestion retrieval fetch https://example.com/page
```

The command creates a uniquely named retrieval bundle under `temp/retrievals` and prints its path. A bundle contains cleaned source HTML in `page.html` and source metadata in `retrieval.yaml`. Use `--output-root PATH` to choose another location. Fetching does not need `KNOWLEDGE_BASE_PATH`.

## 2. Register the source reference

Set the production knowledge-base path, then preview registration before writing it:

```sh
export KNOWLEDGE_BASE_PATH="/absolute/path/to/knowledge-base"
knowledge-base ingestion retrieval register temp/retrievals/fetch-XXXXXX --dry-run
knowledge-base ingestion retrieval register temp/retrievals/fetch-XXXXXX
```

Registration validates the bundle and returns YAML that identifies the source reference. If the canonical URL already has a reference, the command reuses it instead of creating a duplicate.

## 3. Create the review inventory

A person or AI agent reviews `page.html` and creates `ingestion_candidate_inventory.yaml` in the same directory. The inventory records the source evidence, suggested entities and statements, draft vocabulary when needed, and review outcomes. It is a temporary handoff, not production knowledge-base data.

Set the inventory's `source_reference` to the reference returned by registration and its `source_file` to `page.html`. Keep `page.html` beside the inventory.

## 4. Validate the inventory

```sh
knowledge-base ingestion candidate-inventory validate \
  temp/retrievals/fetch-XXXXXX/ingestion_candidate_inventory.yaml
```

Validation checks the inventory against the current production knowledge base. It rejects unknown YAML fields and reports invalid identifiers, evidence links, dependencies, property and value compatibility, and inconsistent optional totals. Validation confirms the review handoff; it does not apply proposed changes.

For command details, see [retrieval](cli/ingestion-retrieval.md) and [candidate-inventory validation](cli/ingestion-candidate-inventory.md). For Rust integration, see the [retrieval crate](../crates/ingestion-retrieval/README.md) and [candidate-inventory crate](../crates/ingestion-candidate-inventory/README.md).
