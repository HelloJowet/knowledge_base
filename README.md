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

The `knowledge-base` executable validates, reads, queries, and updates a knowledge base. Install it from crates.io with:

```sh
cargo install knowledge-base-cli
```

Set `KNOWLEDGE_BASE_PATH` to the knowledge-base directory before running commands:

```sh
export KNOWLEDGE_BASE_PATH="/absolute/path/to/knowledge-base"
knowledge-base validate
```

See the [CLI documentation](docs/cli/README.md) for all commands and examples.

## Agent skills

Reusable skills teach compatible AI coding agents how to operate the CLI and author files that follow the data model. Install skills from this repository with the [skills CLI](https://www.skills.sh/docs):

```sh
npx skills add HelloJowet/knowledge_base --skill knowledge-base-cli
npx skills add HelloJowet/knowledge_base --skill knowledge-base-data-model
```

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
