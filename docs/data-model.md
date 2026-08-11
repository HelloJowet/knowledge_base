# Data model

This page defines the knowledge-base file format. Fields are required unless they are described as optional or have a default. See [`fixtures/valid/minimal`](../fixtures/valid/minimal) for a complete example.

## Common rules

- YAML mappings are strict: unknown fields and duplicate keys are invalid.
- Required strings and collections must not be empty, except an entity's `statements`.
- File names must match the `id` inside them.
- Identifiers use an uppercase letter followed by a positive integer: entities use `Q`, properties `P`, references `R`, statements `S`, and entity types `T`.
- `Q`, `P`, `R`, and `T` identifiers are unique across the knowledge base. `S` identifiers are unique within an entity.
- Language keys are well-formed BCP 47 tags and are unique when compared case-insensitively.
- URL fields contain absolute URLs.

Labels and descriptions are maps keyed by language. Entity labels and descriptions each cite at least one reference. Entity-type and property labels and descriptions are internally authored vocabulary, so their reference lists may be empty:

```yaml
labels:
  en:
    text: City
    references: [R1]
```

## Entity types

An entity type requires `id` and `labels`. `descriptions` is optional.

```yaml
# entity_types/T1.yaml
id: T1
labels:
  en:
    text: City
    references: [R1]
```

## Properties

A property defines which entity types can use it and which type of value it accepts.

```yaml
# properties/P1.yaml
id: P1
labels:
  en:
    text: population
    references: [R1]
subject_types: [T1]
value_type: integer
usage: statement
allowed_qualifiers: [P2]
cardinality: many
```

Required fields are `id`, `labels`, `subject_types`, `value_type`, and `usage`. `usage` is `statement`, `qualifier`, or `both`; it declares whether the property may be used as a top-level statement, a qualifier, or either. `descriptions` is optional, `allowed_qualifiers` defaults to `[]`, `cardinality` defaults to `many`, and `external_ids` defaults to `{}`.

`external_ids` maps a non-empty external-system namespace to zero or more non-empty external property identifiers. Identifiers must not be repeated within a namespace:

```yaml
external_ids:
  wikidata: [P1082, P2046]
  osm: []
```

`cardinality: one` allows at most one top-level statement with that property per entity. Cardinality does not apply to qualifiers.

An entity-valued property also requires `target_types`; other properties cannot set it:

```yaml
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

A property applies to an entity when at least one of the entity's types appears in `subject_types`. An entity value is valid when at least one type of the target entity appears in `target_types`.

A qualifier property must have `usage: qualifier` or `usage: both`, apply to the entity, and appear in the main property's `allowed_qualifiers`. A top-level statement property must have `usage: statement` or `usage: both`.

## Entities

An entity requires `id`, `labels`, `entity_types`, and `statements`. `descriptions` and `images` are optional.

```yaml
# entities/Q1.yaml
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

Each classification and statement cites at least one reference. A qualifier requires `property` and `value`; `qualifiers` defaults to `[]`.

Each statement has one value. Store multiple values as separate statements so each can have its own qualifiers and references. A statement's references support the value and its qualifiers.

An image requires lossless attribution metadata: `url`, `alt`, `source_url`, `creator`, and `license`. `source_url` is the primary provenance link to the image's source or attribution page. Supplemental knowledge-base `references` are optional.

```yaml
images:
  - url: https://example.org/bilecik.jpg
    alt: Bilecik city centre
    source_url: https://example.org/bilecik-image
    creator: Example Archive
    license: CC BY 4.0
    references: [R1]
```

## Values

Every statement and qualifier value has a `type`.

| Type | Value |
| --- | --- |
| `entity` | `value` is a `Q<n>` identifier |
| `string` | `value` is a YAML string |
| `integer` | `value` is a YAML integer |
| `decimal` | `value` is a quoted decimal string |
| `boolean` | `value` is a YAML boolean |
| `date` | `value` is a quoted ISO 8601 date |
| `datetime` | `value` is a quoted RFC 3339 timestamp |
| `url` | `value` is an absolute URL |
| `coordinate` | `latitude` and `longitude` are quoted decimal strings |

Coordinates do not have a `value` field:

```yaml
value:
  type: coordinate
  latitude: "40.1419"
  longitude: "29.9793"
```

Decimals and coordinates use plain base-10 notation without plus signs, exponent notation, leading zeroes, or digit separators. Latitude is between -90 and 90; longitude is between -180 and 180.

## References

A reference requires `id`, `url`, `title`, and `retrieved_at`. `publisher`, `publication_date`, `source_language`, and `archive_url` are optional.

```yaml
# references/R1.yaml
id: R1
url: https://example.org/bilecik
title: Bilecik source
publisher: Example Publisher
publication_date: "2025-01-14"
source_language: en
retrieved_at: "2025-01-15T10:30:00Z"
archive_url: https://web.archive.org/example/bilecik
```

`title` and any supplied metadata values must not be empty. `publication_date` accepts a valid calendar year, month, or day in `YYYY`, `YYYY-MM`, or `YYYY-MM-DD` form. `source_language` is a well-formed BCP 47 language tag. `retrieved_at` is an RFC 3339 timestamp; `url` and `archive_url` are absolute URLs.

## Identifier allocation

`id_allocation.yaml` stores the next identifier in each repository-wide namespace:

```yaml
version: 1
next:
  entity: 3
  property: 4
  reference: 2
  entity_type: 3
```

`version` must be `1`. All four counters are required and must be greater than every identifier already used in their namespace. Gaps are allowed.

Statement identifiers are local to an entity and are not included in this file.

## Entity Markdown

`entity_context/Q<n>.md` contains optional unstructured data attached to entity `Q<n>`. When used, citations are GitHub Flavored Markdown footnotes:

```markdown
Bilecik is a city in Türkiye.[^R1]

[^R1]: [Reference R1](../references/R1.yaml)
```

Each footnote label must name an existing reference, and its link must be `../references/R<n>.yaml`.
