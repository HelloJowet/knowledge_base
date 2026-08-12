# File-based knowledge base

This repository provides a simple, portable foundation for a knowledge base. It stores structured data in YAML and optional free-form notes in Markdown files attached to entities.

Because the data lives in ordinary files, you can inspect and edit it directly, track it with Git, and work with AI coding agents such as Codex and Claude Code.

## Trade-offs

A file-based knowledge base is easiest to use at a small or medium scale. As it grows, validation, querying, and concurrent updates become harder, so you may need an index or database.

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

## Ingestion

Ingestion turns a webpage into reviewable material for the knowledge base. First, retrieve the page into a bundle. Then register or reuse its source reference. A person or AI agent creates an inventory of the proposed entities and statements, and the CLI validates that inventory against the current knowledge base. The inventory is a temporary review artifact; it does not change production data.

See the [ingestion guide](docs/ingestion.md), [`knowledge-base-ingestion-retrieval`](crates/ingestion-retrieval/README.md), and [`knowledge-base-ingestion-candidate-inventory`](crates/ingestion-candidate-inventory/README.md).

## Agent skills

Reusable skills help compatible AI coding agents use the CLI and create files that follow the data model. Install them with the [skills CLI](https://www.skills.sh/docs):

```sh
npx skills add HelloJowet/knowledge_base --skill knowledge-base-cli
npx skills add HelloJowet/knowledge_base --skill knowledge-base-data-model
```

## Development

The workspace includes models, a filesystem-backed CRUD layer, validation, and a CLI. The CRUD crate currently exposes the documented subset of mutation operations:

- [`knowledge-base-models`](crates/models/README.md)
- [`knowledge-base-crud`](crates/crud/README.md)
- [`knowledge-base-ingestion-retrieval`](crates/ingestion-retrieval/README.md)
- [`knowledge-base-ingestion-candidate-inventory`](crates/ingestion-candidate-inventory/README.md)
- [`knowledge-base-validation`](crates/validation/README.md)
- [`knowledge-base-cli`](crates/cli/README.md)

```sh
cargo fmt --check --all
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
```
