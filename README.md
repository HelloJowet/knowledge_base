# File-based knowledge base

A simple foundation for building your own knowledge base. Structured data is stored in YAML, with optional Markdown for unstructured data attached to an entity.

Using files instead of a database makes the knowledge base easy to inspect, edit, version with Git and use with AI coding agents such as Codex and Claude Code. The project intentionally favors simplicity and portability.

## Trade-offs

A file-based knowledge base becomes harder to validate, query, and update concurrently as it grows. This approach is best suited to small or moderately sized knowledge bases; larger deployments may need an index or database.

## Layout

```text
<knowledge-base>/
├── entities/             # entities and their statements
│   └── Q<n>.yaml
├── entity_types/         # entity type definitions
│   └── T<n>.yaml
├── properties/           # property definitions
│   └── P<n>.yaml
├── references/           # sources cited by the knowledge base
│   └── R<n>.yaml
├── entity_context/       # optional Markdown attached to entities
│   └── Q<n>.md
└── id_allocation.yaml    # next available identifiers
```

See the [data model](docs/data-model.md) for the file format. The example in [`fixtures/valid/minimal`](fixtures/valid/minimal) is a small knowledge base you can copy and adapt.

## Command-line interface

Set `KNOWLEDGE_BASE_PATH` to the root directory containing the knowledge-base files:

```sh
export KNOWLEDGE_BASE_PATH="$PWD/fixtures/valid/minimal"
```

The included validator checks file structure, values, ontology rules, cross-references, provenance, and Markdown citations. It reports errors without changing files:

```sh
cargo run -p knowledge-base-cli -- validate
```

Read individual records or an entity's Markdown context with resource-specific commands:

```sh
cargo run -p knowledge-base-cli -- entity read Q1
cargo run -p knowledge-base-cli -- entity query --filter P3=Q2
cargo run -p knowledge-base-cli -- entity relationships Q1
cargo run -p knowledge-base-cli -- entity-type read T1
cargo run -p knowledge-base-cli -- property read P1
cargo run -p knowledge-base-cli -- reference read R1
cargo run -p knowledge-base-cli -- entity-context read Q1
```

Read commands print files exactly as stored and do not validate the rest of the knowledge base. See the [`knowledge-base-validation`](crates/validation/README.md) crate for the checks performed by `validate`.

### Query entities by statement values

Use `entity query` to find entities containing a top-level statement with a given property and value:

```sh
cargo run --quiet --bin knowledge-base -- entity query --filter P3=Q2
```

Repeat `--filter` to require every property/value pair. Filters use `P<n>=value` syntax and are interpreted using the property's declared `value_type`; the first `=` separates the property from the value, so string and URL values may contain additional equals signs. Coordinates use `latitude,longitude` syntax.

```sh
cargo run --quiet --bin knowledge-base -- \
  entity query --filter P3=Q2 --filter P1=228334
```

Only top-level statements satisfy filters; qualifiers are ignored. Matching uses exact typed values, including repeated filters for the same property. The command sorts complete parsed entities by numeric entity ID and returns deterministic YAML with the filters, `total`, pagination metadata, and an `entities` list. It defaults to `--limit 100 --offset 0` and requires at least one filter.

This query reads and parses every entity file. A malformed entity, duplicate identifier, or mismatch between a filename and its declared identifier fails the complete query rather than returning partial results.

### Query direct entity relationships

Entity-valued statements create directed relationships between entities. Query both the statements stored on an entity and backlinks stored on other entities with:

```sh
cargo run --quiet --bin knowledge-base -- entity relationships Q2
```

The command scans all entity files and returns one-hop incoming, outgoing, and self-referential relationships as deterministic YAML. Each result includes the related entity's ID and labels along with the property and statement IDs. It does not follow relationships recursively.

Results are paginated after sorting by source entity, property, statement, and target. The default page contains at most 100 relationships. Use `--limit` and `--offset` for other pages:

```sh
cargo run --quiet --bin knowledge-base -- entity relationships Q2 --limit 25 --offset 25
```

The response includes the total number of direct relationships and a `next_offset` when another page exists. Unlike `entity read`, this query must read and parse the complete entity collection so that incoming relationships are not missed.

### Apply statement manifests

Use a statement manifest to add one or more facts to existing entities in a single operation. A manifest is a temporary YAML file with a top-level `statements` list:

```yaml
statements:
  - entity: Q1
    property: P1
    value: { type: integer, value: 123456789 }
    references: [R1]
```

Each item says:

- `entity`: which entity receives the statement, such as `Q1`
- `property`: what the value means, such as the property `P1`
- `value`: the value and its type; the type must match the property's `value_type`
- `references`: one or more existing reference IDs that support the value

See [Values](docs/data-model.md#values) for the supported value shapes. Do not include a statement ID: the command assigns the next `S<n>` ID available within that entity. It processes items in manifest order, and each entity has its own statement ID sequence.

The manifest format is strict. All four fields above are required, `references` cannot be empty, and unknown fields are rejected. In particular, this command does not currently support qualifiers.

Keep temporary manifests outside the knowledge-base directory when practical, because they are input to the command rather than part of the knowledge base. Preview the batch first with `--dry-run`:

```sh
cargo run --quiet --bin knowledge-base -- \
  entity statement apply /tmp/statement-manifest.yaml --dry-run
```

A successful preview prints `outcome: previewed` and one `would_add` result for each manifest item. The `index` is the item's one-based position in the manifest, and `statement` is the ID that will be assigned:

```yaml
outcome: previewed
results:
- index: 1
  entity: Q1
  property: P1
  statement: S4
  status: would_add
```

If the preview looks correct, apply the same unchanged manifest without `--dry-run`:

```sh
cargo run --quiet --bin knowledge-base -- \
  entity statement apply /tmp/statement-manifest.yaml
```

The apply command reports `outcome: applied` and changes each item from `would_add` to `added`. It appends the new statements to the affected entity files while preserving their existing text and comments.

Both preview and apply validate the current knowledge base and the proposed result. The batch is all-or-nothing: if any item is invalid or already exists, no statements are written and the command exits unsuccessfully. A statement counts as already present when the same entity has the same property and typed value; changing only its references does not make it a new statement. Duplicate results use `outcome: not_applied` and `status: already_present`.

The command uses a `.knowledge-base.lock` file to coordinate with other mutations; a dry run can create this lock file but does not change entity data. Writes are staged and rollback is attempted if replacement fails. A multi-file update is not crash-atomic, so an abrupt process or machine failure can still require manual recovery.

## Development

The workspace contains models, filesystem CRUD, validation, and CLI crates:

- [`knowledge-base-models`](crates/models/README.md)
- [`knowledge-base-crud`](crates/crud/README.md)
- [`knowledge-base-validation`](crates/validation/README.md)
- [`knowledge-base-cli`](crates/cli/README.md)

```sh
cargo fmt --check --all
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
```
