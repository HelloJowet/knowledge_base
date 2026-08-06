# Data model

This document defines the normative MVP file shapes. Fields are required unless
they are explicitly described as optional or given a default. YAML examples use
flow sequences only to save space.

## Common rules

- Every mapping is strict: unknown fields and duplicate YAML keys are invalid.
- Strings and collections explicitly described as nonempty must contain a
  value.
- Identifiers consist of their uppercase namespace letter followed by a
  positive canonical decimal integer. For example, `Q1` is valid, while `Q0`,
  `Q01`, and `q1` are invalid.
- Entity (`Q`), property (`P`), reference (`R`), and entity-type (`T`)
  identifiers are repository-wide. Statement identifiers (`S`) are unique only
  within their entity.
- Localized maps use well-formed BCP 47 language tags as keys. Key spelling is
  preserved, but keys are compared case-insensitively for uniqueness and exact
  lookup.
- URL fields contain absolute URLs.

Localized labels and descriptions have the same shape. Both `text` and a
nonempty `references` sequence are required:

```yaml
en:
  text: City
  references: [R1]
```

## Entity types and properties

An entity type requires `id` and a nonempty `labels` map. `descriptions` is
optional and defaults to an empty map:

```yaml
# entity_types/T1.yaml
id: T1
labels:
  en:
    text: City
    references: [R1]
descriptions:
  en:
    text: An incorporated urban settlement.
    references: [R1]

---
# entity_types/T2.yaml
id: T2
labels:
  en:
    text: Country
    references: [R1]
```

Properties require `id`, a nonempty `labels` map, a nonempty `subject_types`
sequence, and `value_type`. They may include `descriptions`.
`allowed_qualifiers` defaults to an empty sequence, and `cardinality` defaults
to `many`; `one` means that an entity may contain zero or one top-level
statement using the property.

An entity-valued property requires a nonempty `target_types` sequence.
`target_types` is prohibited for every other value type.

```yaml
# properties/P1.yaml
id: P1
labels:
  en:
    text: population
    references: [R1]
subject_types: [T1]
value_type: integer
allowed_qualifiers: [P2]

---
# properties/P2.yaml
id: P2
labels:
  en:
    text: point in time
    references: [R1]
subject_types: [T1]
value_type: date

---
# properties/P3.yaml
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

A property is applicable when its `subject_types` and the entity's types have
at least one value in common. An entity-valued statement or qualifier has a
compatible target when the target entity's types and the property's
`target_types` have at least one value in common. Additional entity
classifications therefore cannot make an otherwise applicable property invalid.

A property used as a qualifier must be applicable to the entity containing the
statement and appear in the main property's `allowed_qualifiers`. Property
cardinality does not apply to qualifiers, and repeated qualifier properties are
allowed.

## Entities and statements

An entity requires `id`, a nonempty `labels` map, a nonempty `entity_types`
sequence, and `statements`. The statements sequence may be empty.
`descriptions` and `images` are optional and default to empty.

Each classification requires `value` and a nonempty `references` sequence.
Each statement requires `id`, `property`, `value`, and a nonempty `references`
sequence. Its `qualifiers` sequence is optional and defaults to empty. Each
qualifier requires `property` and `value`.

```yaml
# entities/Q1.yaml
id: Q1
labels:
  tr:
    text: BİLECİK
    references: [R1]
descriptions:
  en:
    text: A city in Türkiye.
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
  - id: S2
    property: P1
    value:
      type: integer
      value: 219427
    qualifiers:
      - property: P2
        value:
          type: date
          value: "2022-12-31"
    references: [R1]
  - id: S3
    property: P3
    value:
      type: entity
      value: Q2
    references: [R1]

---
# entities/Q2.yaml
id: Q2
labels:
  en:
    text: Türkiye
    references: [R1]
entity_types:
  - value: T2
    references: [R1]
statements: []
```

Each statement has exactly one value. Multiple values for a property are
represented by repeated statements so that each value can have independent
qualifiers and provenance. Statement references support the complete assertion,
including its qualifiers.

An image requires `url`, nonempty `attribution`, and a nonempty `references`
sequence. `attribution_url` is optional:

```yaml
images:
  - url: https://example.org/bilecik.jpg
    attribution: Example Archive, CC BY 4.0
    attribution_url: https://example.org/bilecik-image
    references: [R1]
```

## Typed values

The MVP supports these value types:

| Type         | Representation                                               |
| ------------ | ------------------------------------------------------------ |
| `entity`     | `value` is a `Q<n>` identifier                               |
| `string`     | `value` is a YAML string                                     |
| `integer`    | `value` is a YAML integer                                    |
| `decimal`    | `value` is a quoted base-10 decimal string                   |
| `boolean`    | `value` is a YAML boolean                                    |
| `date`       | `value` is a quoted ISO 8601 calendar date                   |
| `datetime`   | `value` is a quoted RFC 3339 timestamp                       |
| `url`        | `value` is an absolute URL string                            |
| `coordinate` | `latitude` and `longitude` are quoted decimal-degree strings |

All values use a `type` discriminator. Fields not listed for the selected type
are invalid. Coordinates are the only MVP values without a `value` field:

```yaml
value:
  type: coordinate
  latitude: "40.1419"
  longitude: "29.9793"
```

Dates must be real ISO 8601 calendar dates, and datetimes must be RFC 3339
timestamps. Decimal and coordinate components must be quoted strings matching
`-?(0|[1-9][0-9]*)(\.[0-9]+)?`; plus signs, exponent notation, leading zeroes,
and digit separators are invalid. Coordinate latitude must be between -90 and
90 inclusive, and longitude must be between -180 and 180 inclusive.

## References

A reference requires `id`, `url`, and `retrieved_at`. `archive_url` is optional.
`retrieved_at` is an RFC 3339 timestamp. A reference represents a particular
source version observed at a particular retrieval:

```yaml
# references/R1.yaml
id: R1
url: https://example.org/bilecik
retrieved_at: "2025-01-15T10:30:00Z"
archive_url: https://web.archive.org/example/bilecik
```

Source content hashing is not part of the MVP.

## Identifier allocation

The required `id_allocation.yaml` file is versioned and stores the next
identifier number in each repository-wide namespace:

```yaml
version: 1
next:
  entity: 3
  property: 4
  reference: 2
  entity_type: 3
```

`version` must be `1`. Every `next` field is required and must be a positive
integer. Its value is the next number to allocate, so it must be greater than
every identifier currently present in that namespace. Gaps are valid and
counters are never inferred from file counts.

Allocation returns the identifier corresponding to the current counter and then
increments it. Statement identifiers are allocated within an entity as one more
than the greatest existing statement number and do not appear in this file.

## Context documents

Context documents cite registered references with GitHub Flavored Markdown
footnotes whose labels are reference identifiers:

```markdown
<!-- entity_context/Q1.md -->

Bilecik is a city in Türkiye.[^R1]

[^R1]: [Reference R1](../references/R1.yaml)
```

Every footnote label must be a canonical `R<n>` identifier, refer to an existing
reference, and have exactly this relative target:

```text
../references/R<n>.yaml
```

Automated validation checks citation syntax, definitions, and targets. Deciding
whether every factual prose assertion has an appropriate citation remains an
editorial task.

Identifier reuse, deletion, and ontology migration are not governed by the MVP.
Validation considers the repository's current contents.
