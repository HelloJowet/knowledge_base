---
name: knowledge-base-cli
description: Use the `knowledge-base` CLI to validate, read, search, query, inspect, ingest, and safely update a file-based knowledge base. Use for CLI-driven work with entities, relationships, statements, references, retrieval bundles, or ingestion candidate inventories instead of directly editing supported resources.
---

# Use the Knowledge Base CLI

Read [the CLI overview](references/docs/cli/README.md), then read the page for the command needed by the task:

- [validation](references/docs/cli/validate.md)
- [resource reads](references/docs/cli/read.md)
- [entity queries and label search](references/docs/cli/query.md)
- [direct relationships](references/docs/cli/relationships.md)
- [statement application](references/docs/cli/statements.md)
- [reference registration](references/docs/cli/references.md)
- [webpage retrieval and registration](references/docs/cli/ingestion-retrieval.md)
- [candidate-inventory validation](references/docs/cli/ingestion-candidate-inventory.md)
- [the complete ingestion workflow](references/docs/ingestion.md)

Consult [the data model](references/docs/data-model.md) when a command depends on resource structure or typed values.

## Choose the knowledge base

`KNOWLEDGE_BASE_PATH` must point to the folder containing the knowledge-base resources and `id_allocation.yaml`. Use an existing setting when it identifies the intended knowledge base. Otherwise, identify the root and pass its absolute path to the command. Ask the user only when multiple plausible roots remain.

Retrieval fetches, `--help`, and `--version` do not require `KNOWLEDGE_BASE_PATH`.

## Execute safely

- Use read-only commands freely when they help answer the request.
- Run mutating commands only when the user asked to change knowledge-base data or create ingestion artifacts.
- For statement application, reference registration, and retrieval registration, run the documented `--dry-run` form before the corresponding write. Continue only when the preview succeeds, then use the same unchanged inputs.
- Check every command's exit status. Explain errors in plain language and never present partial output as a complete result.
- Validate the knowledge base after a mutation and report the created or reused identifiers.

The CLI supports statement application and reference registration but does not directly create or edit every resource. Use `$knowledge-base-data-model` for entity, entity-type, property, entity-context, and identifier-allocation changes that have no CLI operation.
