# Query entities

Use `entity query` to find entities containing a top-level statement with a given property and value:

```sh
knowledge-base entity query --filter P3=Q2
```

At least one `--filter` is required. Repeat it to require every property/value pair:

```sh
knowledge-base \
  entity query --filter P3=Q2 --filter P1=228334
```

Filters use `P<n>=value` syntax. The command reads the property definition and interprets the value using its declared `value_type`; see the data model's [supported values](../data-model.md#values). The first `=` separates the property from the value, so string and URL values may contain additional equals signs. Coordinates use `latitude,longitude` syntax.

Only top-level statements satisfy filters; qualifiers are ignored. Every filter must match, including repeated filters for the same property, and values use exact typed equality.

## Results and pagination

The command returns deterministic YAML containing the parsed filters, `total`, pagination metadata, and complete parsed entities sorted by numeric entity ID. It defaults to `--limit 100 --offset 0`; `--limit` must be greater than zero. When more matching entities remain, the response includes `next_offset`.

The query reads and parses every entity file. A malformed entity, duplicate identifier, or mismatch between a filename and its declared identifier fails the complete query rather than returning partial results.
