# Knowledge base architecture

## Purpose

The knowledge base is a version-controlled, file-based system for structured
claims and explanatory entity context. Its architecture is independent of the
subject matter stored in it.

The directory supplied as `<knowledge-base-root>` is the source of truth. Its
name and location are not prescribed. The `knowledge-base-models` and
`knowledge-base-validation` libraries model and validate its contents, while
the `knowledge-base-cli` application exposes validation to people and
repository automation.

The system combines:

- structured YAML for explicit claims; and
- optional English Markdown for sourced explanation, synthesis, nuance, and
  conflicts that do not fit the structured model.

External data is input and evidence. It becomes production knowledge only after
identity matching, provenance capture, modeling, and validation.

## Production layout

| Path | Content |
| --- | --- |
| `<knowledge-base-root>/entities/Q<n>.yaml` | One required structured record per entity |
| `<knowledge-base-root>/entity_context/Q<n>.md` | Zero or one English context document for an entity |
| `<knowledge-base-root>/references/R<n>.yaml` | Evidence from one source version and retrieval |
| `<knowledge-base-root>/entity_types/T<n>.yaml` | Authoritative entity-type vocabulary |
| `<knowledge-base-root>/properties/P<n>.yaml` | Authoritative property and qualifier vocabulary |
| `<knowledge-base-root>/id_allocation.yaml` | Monotonic allocation of `Q`, `P`, `R`, and `T` identifiers |

An entity is an identified subject. A claim is an assertion about an entity,
and a statement is its typed representation in an entity record. Statement
identifiers (`S<n>`) are unique within their entity.

## Core principles

- Property definitions are authoritative for applicability, direction, value
  type, target types, and permitted qualifiers.
- Relationships are stored once in the property's declared direction. Reverse
  relationships are derived by the query layer.
- Available properties are derived from their permitted subject types rather
  than duplicated in entity-type records.
- Production labels, descriptions, classifications, images, and statements
  cite registered references.
- Context deserving independent treatment, such as an event, dispute, period,
  or relationship, is modeled as another entity.

## Runtime and query model

The initial validator loads the complete knowledge base into memory. The query
interface described below is a later read-only library milestone and will
expose a `KnowledgeBase` only after every enabled validation layer succeeds.

The planned query library will provide:

- direct lookup of an entity, entity type, property, or reference by its typed
  identifier;
- an entity's outgoing statements, optionally filtered by property;
- incoming entity relationships, optionally filtered by property;
- the properties available to an entity based on its types; and
- exact-locale lookup of labels and descriptions.

A failed direct lookup returns a structured not-found error containing the
identifier kind and value. Collection queries return an empty result when
nothing matches, including when their entity does not exist. Locale lookup is
case-insensitive but performs no language fallback.

Outgoing statements retain their order in the entity file. Incoming
relationships are ordered by numeric subject-entity identifier and then numeric
statement identifier. Available properties are ordered by numeric property
identifier.

Incoming relationships are derived from entity-valued statements. The loader
may build an index equivalent to:

```text
target entity identifier -> [(subject entity identifier, statement identifier)]
```

Reverse statements are never stored in production files.

## Initial implementation scope

The initial implementation is read-only. It includes strict loading, validation,
and a `validate` CLI command. The immutable query interface, mutation,
transactional replacement, and identifier-allocation concurrency are outside
this milestone.

The Rust workspace keeps the dependency direction explicit:

```text
knowledge-base-models <- knowledge-base-validation <- knowledge-base-cli
```

Models contain only the serializable data model and typed identifiers.
Validation owns repository loading, validation rules, and diagnostics. The CLI
owns argument parsing, output, and process exit behavior.

The normative file shapes are defined in the [data model](data-model.md).
Repository correctness is defined in [validation](validation.md). Deferred,
non-MVP work is listed in [TODOs](todo.md).
