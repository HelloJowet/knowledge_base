# Knowledge Base Data Model Reference

## Contents

- [How the pieces fit together](#how-the-pieces-fit-together)
- [Repository layout](#repository-layout)
- [Common rules](#common-rules)
- [Entity types](#entity-types)
- [Properties](#properties)
- [Entities](#entities)
- [Typed values](#typed-values)
- [References](#references)
- [Identifier allocation](#identifier-allocation)
- [Entity Markdown](#entity-markdown)
- [Cross-resource validation](#cross-resource-validation)

## How the pieces fit together

The knowledge base represents subjects as structured records supported by explicit sources:

- An **entity** represents a distinct subject, such as a city, person, organization, or topic.
- An **entity type** classifies entities, for example as a `City` or `Country`.
- A **property** defines an attribute or relationship, such as `population` or `country`.
- A **statement** assigns a property value to an entity. For example, a population statement associates an entity with an integer value.
- A **reference** identifies the source supporting a label, description, classification, statement, image, or Markdown claim.
- **Entity context** provides optional Markdown for relevant information that is better expressed as prose than as structured statements.

Each resource has a stable ID. In a small example, `Q1` might be the entity Bilecik, `T1` the type City, `P1` the population property, and `R1` the source supporting those records. The sections below define their exact files and how they may connect.

## Repository layout

```text
<knowledge-base>/
├── entities/Q<n>.yaml
├── entity_types/T<n>.yaml
├── properties/P<n>.yaml
├── references/R<n>.yaml
├── entity_context/Q<n>.md
└── id_allocation.yaml
```

Entity context is optional per entity. The other resource collections and allocation file form the structured repository.

## Common rules

- Treat YAML mappings as strict: reject unknown and duplicate fields.
- Require all fields unless this reference marks them optional or gives a default.
- Require strings and collections to be nonempty, except an entity's `statements` list.
- Match every resource filename to its internal `id`.
- Use uppercase prefixes followed by positive integers: `Q` entity, `P` property, `R` reference, `S` statement, and `T` entity type.
- Keep `Q`, `P`, `R`, and `T` IDs unique repository-wide. Keep `S` IDs unique within an entity.
- Use well-formed BCP 47 language tags. Treat tags case-insensitively when checking uniqueness.
- Use absolute URLs in every URL field.

Represent localized labels and descriptions as language maps. Make each value cite one or more references:

```yaml
labels:
  en:
    text: City
    references: [R1]
descriptions:
  en:
    text: A large human settlement
    references: [R1]
```

`labels` are required on entities, entity types, and properties. `descriptions` are optional.

## Entity types

Store each entity type in `entity_types/T<n>.yaml`:

```yaml
id: T1
labels:
  en:
    text: City
    references: [R1]
```

Require `id` and `labels`; allow optional `descriptions`.

## Properties

Store each property in `properties/P<n>.yaml`:

```yaml
id: P1
labels:
  en:
    text: population
    references: [R1]
subject_types: [T1]
value_type: integer
allowed_qualifiers: [P2]
cardinality: many
```

Require `id`, `labels`, `subject_types`, and `value_type`. Allow optional `descriptions`. Default `allowed_qualifiers` to `[]` and `cardinality` to `many`.

Set `cardinality` to `one` to permit at most one top-level statement using the property on an entity. Do not apply cardinality to qualifier occurrences.

For `value_type: entity`, require nonempty `target_types`:

```yaml
id: P3
labels:
  en:
    text: country
    references: [R1]
subject_types: [T1]
value_type: entity
target_types: [T2]
cardinality: one
```

For every other value type, omit `target_types`.

Apply a property to an entity only when at least one entity type appears in the property's `subject_types`. For an entity-valued statement, require at least one type of the target entity to appear in `target_types`.

Use a property as a qualifier only when it applies to the subject entity and its ID appears in the main property's `allowed_qualifiers`.

## Entities

Store each entity in `entities/Q<n>.yaml`:

```yaml
id: Q1
labels:
  en:
    text: Bilecik
    references: [R1]
entity_types:
  - value: T1
    references: [R1]
statements:
  - id: S1
    property: P1
    value:
      type: integer
      value: 228334
    qualifiers:
      - property: P2
        value:
          type: date
          value: "2023-12-31"
    references: [R1]
```

Require `id`, `labels`, nonempty `entity_types`, and `statements`. Allow optional `descriptions` and `images`. Permit `statements: []`. Require one or more references for every entity-type classification and statement. Default a statement's `qualifiers` to `[]`.

Represent multiple values as separate statements so each has independent qualifiers and references. Treat a statement's references as support for both its value and qualifiers.

Represent images as:

```yaml
images:
  - url: https://example.org/bilecik.jpg
    attribution: Example Archive, CC BY 4.0
    attribution_url: https://example.org/bilecik-image
    references: [R1]
```

Require `url`, `attribution`, and nonempty `references`; allow optional `attribution_url`.

## Typed values

Use exactly one supported `type` and its corresponding shape:

| Type | Required value |
| --- | --- |
| `entity` | `value` is a `Q<n>` ID |
| `string` | `value` is a YAML string |
| `integer` | `value` is a YAML integer |
| `decimal` | `value` is a quoted canonical decimal string |
| `boolean` | `value` is a YAML boolean |
| `date` | `value` is a quoted ISO 8601 `YYYY-MM-DD` date |
| `datetime` | `value` is a quoted RFC 3339 timestamp |
| `url` | `value` is an absolute URL |
| `coordinate` | quoted `latitude` and `longitude` decimal strings; omit `value` |

Examples:

```yaml
value: { type: entity, value: Q2 }
value: { type: string, value: Example }
value: { type: integer, value: -42 }
value: { type: decimal, value: "-0.25" }
value: { type: boolean, value: true }
value: { type: date, value: "2024-02-29" }
value: { type: datetime, value: "2024-02-29T12:34:56Z" }
value: { type: url, value: https://example.org/item }
value:
  type: coordinate
  latitude: "40.1419"
  longitude: "29.9793"
```

Write decimal and coordinate components in plain base-10 form. Forbid plus signs, exponent notation, leading zeroes except zero itself, and digit separators. Keep latitude between -90 and 90 and longitude between -180 and 180.

Require every statement value to match its property's `value_type`. Apply the same rule to qualifier values and their qualifier-property definitions.

## References

Store each reference in `references/R<n>.yaml`:

```yaml
id: R1
url: https://example.org/bilecik
retrieved_at: "2025-01-15T10:30:00Z"
archive_url: https://web.archive.org/example/bilecik
```

Require `id`, absolute `url`, and RFC 3339 `retrieved_at`. Allow optional absolute `archive_url`.

Require all cited reference IDs to exist. Reject orphaned or mismatched citation links in entity Markdown.

## Identifier allocation

Store repository-wide allocation state in `id_allocation.yaml`:

```yaml
version: 1
next:
  entity: 3
  property: 4
  reference: 2
  entity_type: 3
```

Require `version: 1` and all four counters. Keep every counter greater than every ID already used in its namespace. Permit gaps, but never fill them by moving a counter backward.

To allocate a new repository-wide ID, use the current counter and increment that counter by one. Statement IDs are local to an entity and do not appear in this file; assign the next unused numeric `S` ID within the entity.

## Entity Markdown

Store optional unstructured context in `entity_context/Q<n>.md`. Require the entity to exist. Cite factual claims with GitHub Flavored Markdown footnotes:

```markdown
Bilecik is a city in Türkiye.[^R1]

[^R1]: [Reference R1](../references/R1.yaml)
```

Use an existing reference ID as each footnote label. Link it exactly to `../references/R<n>.yaml`. Ensure every defined reference is used and every citation has a matching definition.

## Cross-resource validation

Check all of the following after an edit:

- Every filename and declared ID agree, and allocation counters remain ahead of used IDs.
- Every cited reference, classified entity type, property, target entity, and qualifier property exists.
- Every property applies to the subject entity's types.
- Every entity target satisfies the property's target types.
- Every qualifier is allowed by its main property and applies to the subject entity.
- Every value matches the declared property type and canonical representation.
- Every `cardinality: one` property occurs no more than once as a top-level statement per entity.
- Every label, description, classification, statement, image, and Markdown claim has valid provenance.
