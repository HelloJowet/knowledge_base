# Command-line interface

The `knowledge-base` command validates, reads, queries, and updates a file-based knowledge base.

## Setup

Most executable commands read the knowledge-base root from `KNOWLEDGE_BASE_PATH`:

```sh
export KNOWLEDGE_BASE_PATH="$PWD/fixtures/valid/minimal"
```

Run the CLI from the workspace with Cargo:

```sh
knowledge-base validate
```

Build and run the standalone executable with:

```sh
cargo build --release -p knowledge-base-cli
KNOWLEDGE_BASE_PATH=/path/to/knowledge-base target/release/knowledge-base validate
```

`ingestion retrieval fetch`, `--help`, and `--version` do not require `KNOWLEDGE_BASE_PATH`.

## Commands

- [Validate a knowledge base](validate.md) with `validate`
- [Work with ingestion artifacts](../ingestion.md)
- [Retrieve and register webpages](ingestion-retrieval.md) with `ingestion retrieval`
- [Validate an ingestion candidate inventory](ingestion-candidate-inventory.md) with `ingestion candidate-inventory validate`
- [Read stored resources](read.md) with the resource-specific `read` commands
- [Query entities](query.md) with `entity query`
- [Search entity labels](query.md#search-entity-labels) with `entity search`
- [Query direct relationships](relationships.md) with `entity relationships`
- [Apply statement manifests](statements.md) with `entity statement apply`
- [Register references](references.md) with `reference register`

## Output and exit status

Read commands write stored files to standard output exactly as written. Query and mutation commands produce deterministic YAML. Validation diagnostics and command errors are written to standard error.

Commands exit unsuccessfully when `KNOWLEDGE_BASE_PATH` is unset or empty, a requested file cannot be read, a query cannot parse the entity collection, validation diagnostics are found, or a mutation cannot be applied. Consult a command's page for its additional validation and failure behavior.
