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

## What this repository owns

`knowledge-base` defines the generic repository format and common operations: validation, filesystem CRUD, resource reads, entity search and queries, relationship inspection, statement application, and reference registration.

The sibling `knowledge_base_public_transport` workspace supplies public-transport rules and reports through `knowledge-base-public-transport`. The sibling `geo_lake_02` workspace retains web retrieval, source-specific reference acquisition, candidate-inventory validation, legacy migration, and route-element export through `geo-lake-knowledge-base`.

## Temporary sibling integration

During consolidation, the workspaces use local sibling-path dependencies instead of released packages. Consumers pin generic crates to `=0.3.0` and public-transport crates to `=0.1.0`:

```text
parent/
├── geo_lake_02/
├── knowledge_base/
└── knowledge_base_public_transport/
```

Replace every sibling-path dependency with an immutable released dependency before publishing or distributing a consumer workspace independently.

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
- [`knowledge-base-validation`](crates/validation/README.md)
- [`knowledge-base-cli`](crates/cli/README.md)

```sh
cargo fmt --check --all
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
```
