# knowledge-base-cli

This crate provides the `knowledge-base` executable for validating, reading, querying, and updating a file-based knowledge base.

## Installation

Install the executable from crates.io:

```sh
cargo install knowledge-base-cli
```

Every executable command reads the knowledge-base root from `KNOWLEDGE_BASE_PATH`:

```sh
export KNOWLEDGE_BASE_PATH="/absolute/path/to/knowledge-base"
```

`ingestion retrieval fetch`, `--help`, and `--version` do not require
`KNOWLEDGE_BASE_PATH`.

## Commands

Validate the complete knowledge base:

```sh
knowledge-base validate
```

Validate an ingestion candidate inventory against the configured knowledge
base:

```sh
knowledge-base ingestion candidate-inventory validate \
  /path/to/bundle/ingestion_candidate_inventory.yaml
```

The inventory must use that exact filename and have its declared `page.html`
beside it. Existing `candidate_inventory.yaml` files must be renamed.

Fetch a webpage into a uniquely named retrieval bundle, then register or reuse
its canonical source reference:

```sh
knowledge-base ingestion retrieval fetch https://example.com/page
knowledge-base ingestion retrieval fetch https://example.com/page --output-root /tmp/retrievals
knowledge-base ingestion retrieval register /tmp/retrievals/fetch-XXXXXX
knowledge-base ingestion retrieval register /tmp/retrievals/fetch-XXXXXX --dry-run
```

A retrieval bundle contains cleaned `page.html` and versioned `retrieval.yaml`
metadata. Registration validates the bundle and produces the standard reference
registration YAML result.

Read stored resources by their typed identifiers:

```sh
knowledge-base entity read Q1
knowledge-base entity-type read T1
knowledge-base property read P1
knowledge-base reference read R1
knowledge-base entity-context read Q1
```

Query entities using one or more typed property-value filters:

```sh
knowledge-base entity query --filter 'P3=Q2'
knowledge-base entity query --filter 'P3=Q2' --filter 'P1=228334'
knowledge-base entity query --filter 'P3=Q2' --limit 25 --offset 25
```

Multiple filters must all match. Results are deterministic YAML and default to 100 items from offset 0.

Search every localized entity label with case-insensitive substring matching:

```sh
knowledge-base entity search Türkiye
knowledge-base entity search Türkiye --limit 25 --offset 25
```

Search results contain complete canonical entities and pagination metadata. Exact label matches sort before substring matches, with numeric entity-ID ordering within each group.

Inspect direct incoming, outgoing, and self-referential relationships:

```sh
knowledge-base entity relationships Q2
knowledge-base entity relationships Q2 --limit 25 --offset 25
```

Preview and apply a YAML statement manifest:

```sh
knowledge-base entity statement apply /tmp/statements.yaml --dry-run
knowledge-base entity statement apply /tmp/statements.yaml
```

An example manifest is:

```yaml
statements:
  - entity: Q1
    property: P1
    value: { type: integer, value: 123456789 }
    references: [R1]
```

The CLI assigns statement identifiers. Each item must contain `entity`, `property`, `value`, and at least one reference. A batch is applied completely or not at all.

Register a reference or reuse one with the exact same canonical URL:

```sh
knowledge-base reference register \
  --url https://example.org/source \
  --title 'Example source' \
  --publisher 'Example Publisher' \
  --publication-date 2026-08 \
  --source-language en \
  --archive-url https://archive.example.org/source
knowledge-base reference register --url https://example.org/source --title 'Example source' --dry-run
```

The command records its current UTC time as `retrieved_at`. It never fetches a source; callers supply the canonical URL and metadata. Exact URL matches return an existing-reference outcome without changing metadata or identifier allocation. Registration and dry runs validate both the baseline and staged repository; their YAML output contains `status` (`previewed`, `registered`, or `existing`) and the `reference` identifier.

## Output and exit status

Read commands write stored files to standard output exactly as written. Query and mutation commands produce deterministic YAML. Validation diagnostics and command errors are written to standard error.

Commands exit unsuccessfully when configuration is missing, input is invalid, a resource cannot be read, validation diagnostics are found, or a mutation cannot be applied.
