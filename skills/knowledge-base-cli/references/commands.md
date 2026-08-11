# Knowledge Base CLI Commands

## Set the knowledge-base folder

Every command needs `KNOWLEDGE_BASE_PATH`, the folder containing the knowledge-base files:

```sh
export KNOWLEDGE_BASE_PATH="/absolute/path/to/knowledge-base"
```

The examples below assume it is set.

## Validate

```sh
knowledge-base validate
```

Check the complete knowledge base, including its file structure, values, resource links, supporting references, and Markdown citations. This command does not change knowledge-base files.

## Read a resource

```sh
knowledge-base entity read Q1
knowledge-base entity-type read T1
knowledge-base property read P1
knowledge-base reference read R1
knowledge-base entity-context read Q1
```

The prefixes identify each resource: `Q` for entity, `T` for entity type, `P` for property, and `R` for reference. A read fails when the identifier is invalid or the resource cannot be read.

## Find entities

Use at least one property-value filter:

```sh
knowledge-base entity query --filter 'P3=Q2'
knowledge-base entity query --filter 'P3=Q2' --filter 'P1=228334'
```

The first example means “find entities with property `P3` equal to entity `Q2`.” Multiple filters must all match. Only top-level statements match; qualifiers do not.

The property definition determines the filter's value type:

| Value type | Filter value |
| --- | --- |
| `entity` | Entity ID such as `Q2` |
| `string` | Exact text |
| `integer` | Whole number such as `-42` |
| `decimal` | Decimal such as `-0.25` |
| `boolean` | `true` or `false` |
| `date` | Date such as `2024-02-29` |
| `datetime` | RFC 3339 timestamp |
| `url` | Absolute URL |
| `coordinate` | `latitude,longitude`, such as `40.1,-29.2` |

Results are YAML and default to 100 items from offset 0. Use `--limit` and `--offset` for another page; the limit must be greater than zero:

```sh
knowledge-base entity query --filter 'P3=Q2' --limit 25 --offset 25
```

An invalid filter or malformed knowledge-base file causes the complete query to fail.

## Inspect direct relationships

```sh
knowledge-base entity relationships Q2
knowledge-base entity relationships Q2 --limit 25 --offset 25
```

Return direct incoming, outgoing, and self-referential links. The command does not follow links recursively. Each YAML result identifies the related entity, direction, property, and statement, with the same pagination options as entity queries.

## Add statements

Create a YAML manifest outside the knowledge-base folder when practical:

```yaml
statements:
  - entity: Q1
    property: P1
    value: { type: integer, value: 123456789 }
    references: [R1]
```

Each item needs `entity`, `property`, `value`, and at least one entry in `references`. Do not add a statement ID; the CLI assigns it. Qualifiers and extra fields are not supported.

Match the property's `value_type`:

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

Use plain decimal text for decimals and coordinates. Latitude must be between -90 and 90; longitude must be between -180 and 180.

Preview first:

```sh
knowledge-base entity statement apply /tmp/statements.yaml --dry-run
```

Continue only when the preview succeeds with `outcome: previewed` and every statement has `status: would_add`. Then, if the user requested the change, apply the same manifest:

```sh
knowledge-base entity statement apply /tmp/statements.yaml
```

Success reports `outcome: applied`, `status: added`, and the assigned statement IDs. The whole batch succeeds or none of it is added. A statement is considered existing when its entity, property, and typed value match, even if its references differ.
