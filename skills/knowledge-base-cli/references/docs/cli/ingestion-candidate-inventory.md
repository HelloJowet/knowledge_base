# Validate an ingestion candidate inventory

An ingestion candidate inventory is a temporary, reviewable YAML handoff for one source article. It can record evidence, proposed entities and statements, draft vocabulary, and review outcomes without adding those records to the production knowledge base.

Set `KNOWLEDGE_BASE_PATH` to the production repository that supplies the existing entities, properties, types, and references, then run:

```sh
knowledge-base ingestion candidate-inventory validate \
  /path/to/bundle/ingestion_candidate_inventory.yaml
```

The filename must be exactly `ingestion_candidate_inventory.yaml`. Its declared `page.html` must exist beside it.

A valid inventory prints a confirmation to standard output and exits `0`. Diagnostics and repository-loading failures are printed to standard error and exit `1`. Invalid command usage exits `2`.

See the [ingestion guide](../ingestion.md) for the complete workflow and the [retrieval command reference](ingestion-retrieval.md) for creating source bundles.
