# Query direct entity relationships

Entity-valued statements create directed relationships between entities. Query both statements stored on an entity and backlinks stored on other entities with:

```sh
cargo run --quiet --bin knowledge-base -- entity relationships Q2
```

The command returns one-hop incoming, outgoing, and self-referential relationships as deterministic YAML. Each result contains its direction, the related entity's ID and labels, and the property and statement IDs. Relationships are not followed recursively.

The complete entity collection is read and parsed so incoming relationships are not missed. A missing requested entity or malformed entity collection fails the query rather than returning partial results.

## Pagination

Results are sorted by source entity, property, statement, and target before pagination. The command defaults to `--limit 100 --offset 0`; `--limit` must be greater than zero. Use both options to request another page:

```sh
cargo run --quiet --bin knowledge-base -- entity relationships Q2 --limit 25 --offset 25
```

The response includes `total` and includes `next_offset` when another page exists.
