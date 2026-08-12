# knowledge-base-cli

`knowledge-base-cli` provides the `knowledge-base` command for checking, reading, searching, and updating a file-based knowledge base.

## Installation and setup

Install the executable from crates.io:

```sh
cargo install knowledge-base-cli
```

Set `KNOWLEDGE_BASE_PATH` to the directory that contains your knowledge base:

```sh
export KNOWLEDGE_BASE_PATH="/absolute/path/to/knowledge-base"
```

Every knowledge base needs an `extensions.yaml` file, even when it uses no extensions:

```yaml
version: 1
extensions: {}
```

Commands that read or change the knowledge base check this file before they run. `--help`, `--version`, and `knowledge-base ingestion retrieval fetch` do not need `KNOWLEDGE_BASE_PATH` or an extension manifest.

## Extensions

Use these commands to inspect the configured extensions:

```sh
knowledge-base extension list
knowledge-base extension check
```

`extension list` prints a deterministic YAML summary of extensions declared by the repository and extensions included in the executable. `extension check` verifies the manifest, extension dependencies, configured ontology bindings, and ontology contracts.

## Common commands

Validate the complete knowledge base:

```sh
knowledge-base validate
```

Read stored resources by identifier:

```sh
knowledge-base entity read Q1
knowledge-base entity-type read T1
knowledge-base property read P1
knowledge-base reference read R1
knowledge-base entity-context read Q1
```

Search entities or query statement values:

```sh
knowledge-base entity search Türkiye
knowledge-base entity query --filter 'P3=Q2'
knowledge-base entity query --filter 'P3=Q2' --filter 'P1=228334' --limit 25 --offset 25
knowledge-base entity relationships Q2
```

Apply statements from a YAML manifest, using `--dry-run` to preview the result first:

```sh
knowledge-base entity statement apply /tmp/statements.yaml --dry-run
knowledge-base entity statement apply /tmp/statements.yaml
```

Each statement needs an entity, property, typed value, and at least one reference. A batch is applied completely or not at all.

Register a source reference or preview the result:

```sh
knowledge-base reference register --url https://example.org/source --title 'Example source'
knowledge-base reference register --url https://example.org/source --title 'Example source' --dry-run
```

Fetch a webpage into a retrieval bundle, then register its source reference:

```sh
knowledge-base ingestion retrieval fetch https://example.com/page
knowledge-base ingestion retrieval register /tmp/retrievals/fetch-XXXXXX
```

Validate an ingestion candidate inventory:

```sh
knowledge-base ingestion candidate-inventory validate /path/to/bundle/ingestion_candidate_inventory.yaml
```

The inventory must use that exact filename and its declared `page.html` file must be beside it.

## Output and exit status

Read commands write the stored file exactly as written. Query, mutation, and extension inspection commands write deterministic YAML. Diagnostics and command errors are written to standard error.

Commands exit unsuccessfully when configuration is missing, the manifest or extension set is invalid, input is invalid, a resource cannot be read, validation finds a problem, or a mutation cannot be applied.
