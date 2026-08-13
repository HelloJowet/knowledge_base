# Command-line interface

The `knowledge-base` command validates, reads, queries, and updates a file-based knowledge base.

## Setup

Most executable commands read the knowledge-base root from `KNOWLEDGE_BASE_PATH`:

```sh
export KNOWLEDGE_BASE_PATH="$PWD/fixtures/valid/minimal"
```

Run the CLI from the workspace with Cargo:

```sh
cargo run -p knowledge-base-cli -- validate
```

Build and run the standalone executable with:

```sh
cargo build --release -p knowledge-base-cli
KNOWLEDGE_BASE_PATH=/path/to/knowledge-base target/release/knowledge-base validate
```

`ingestion retrieval fetch`, `--help`, and `--version` do not require `KNOWLEDGE_BASE_PATH`.

Every knowledge base also requires `extensions.yaml`, including a base-only knowledge base:

```yaml
version: 1
extensions: {}
```

Commands that use the knowledge base check this configuration before reading or writing records.

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
- [Inspect extension configuration](#extensions) with `extension list` and `extension check`

## Extensions

Extensions add repository-specific rules and optional commands to a custom executable. The standard `knowledge-base` executable includes no extensions. Use these commands to inspect a repository's configuration:

```sh
knowledge-base extension list
knowledge-base extension check
```

`list` reports the extensions declared by the repository and compiled into the executable. `check` also verifies their dependencies, bindings, and ontology requirements.

## Output and exit status

Read commands write stored files to standard output exactly as written. Query and mutation commands produce deterministic YAML. Validation diagnostics and command errors are written to standard error.

Commands exit unsuccessfully when `KNOWLEDGE_BASE_PATH` is unset or empty, the extension configuration is invalid, a requested file cannot be read, a query cannot parse the entity collection, validation diagnostics are found, or a mutation cannot be applied. Consult a command's page for its additional validation and failure behavior.
