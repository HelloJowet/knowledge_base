# Knowledge Base Validator

A read-only Rust workspace for a version-controlled, file-based knowledge base. Knowledge is stored as strict YAML records, with optional Markdown documents for sourced explanatory context.

The validator checks file structure, identifiers, typed values, ontology rules, cross-references, provenance, allocation counters, and Markdown citations. A knowledge-base directory can have any name and can live anywhere.

## Knowledge-base layout

The path passed to `validate` must contain:

```text
<knowledge-base-root>/
├── entities/
│   └── Q<n>.yaml
├── entity_types/
│   └── T<n>.yaml
├── properties/
│   └── P<n>.yaml
├── references/
│   └── R<n>.yaml
├── entity_context/       # optional
│   └── Q<n>.md
└── id_allocation.yaml
```

The four YAML directories and `id_allocation.yaml` are required. `entity_context/` is optional. Managed directories may contain only their documented file type.

See the complete specifications:

- [Architecture](docs/architecture.md)
- [Data model](docs/data-model.md)
- [Validation rules](docs/validation.md)
- [Deferred topics](docs/todo.md)

The small, valid data set in [`fixtures/valid/minimal`](fixtures/valid/minimal) is also a useful starting point.

## Workspace crates

- [`knowledge-base-models`](crates/models/README.md): Serde models and typed identifiers
- [`knowledge-base-validation`](crates/validation/README.md): repository validation and diagnostics
- [`knowledge-base-cli`](crates/cli/README.md): the `knowledge-base` command

## Development checks

Run the same checks used for this implementation:

```sh
cargo fmt --check --all
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
cargo run -p knowledge-base-cli -- validate fixtures/valid/minimal
```

The current milestone intentionally does not include mutation or query APIs.
