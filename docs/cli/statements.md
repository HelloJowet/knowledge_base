# Apply statement manifests

Use a statement manifest to add one or more facts to existing entities in a single operation. A manifest is a temporary YAML file with a top-level `statements` list:

```yaml
statements:
  - entity: Q1
    property: P1
    value: { type: integer, value: 123456789 }
    references: [R1]
```

Each item requires:

- `entity`: the entity receiving the statement, such as `Q1`
- `property`: what the value means, such as `P1`
- `value`: a typed value matching the property's `value_type`
- `references`: one or more existing reference IDs supporting the value

See the data model's [supported values](../data-model.md#values) for value shapes. Do not include a statement ID: the command assigns the next `S<n>` ID available within that entity. Items are processed in manifest order, and each entity has its own statement ID sequence.

The manifest format is strict. All four fields are required, `references` cannot be empty, and unknown fields are rejected. Qualifiers are not currently supported.

Keep temporary manifests outside the knowledge-base directory when practical because they are command input, not part of the knowledge base.

## Preview and apply

Preview a batch first with `--dry-run`:

```sh
cargo run --quiet --bin knowledge-base -- \
  entity statement apply /tmp/statement-manifest.yaml --dry-run
```

A successful preview prints `outcome: previewed` and one `would_add` result for each manifest item. The `index` is the item's one-based position, and `statement` is the ID that would be assigned:

```yaml
outcome: previewed
results:
- index: 1
  entity: Q1
  property: P1
  statement: S4
  status: would_add
```

After confirming every result is `would_add`, apply the same unchanged manifest without `--dry-run`:

```sh
cargo run --quiet --bin knowledge-base -- \
  entity statement apply /tmp/statement-manifest.yaml
```

The apply command reports `outcome: applied` and changes each result's status from `would_add` to `added`. It appends statements to the affected entity files while preserving their existing text and comments outside the appended statement blocks.

## Validation and write behavior

Preview and apply both validate the current knowledge base and the proposed result. The batch is all-or-nothing: if any item is invalid or already exists, no statements are written and the command exits unsuccessfully.

A statement is already present when the same entity has the same property and typed value; changing only its references does not create a new statement. Duplicate results use `outcome: not_applied` and `status: already_present`.

The command uses a `.knowledge-base.lock` file to coordinate with other mutations. A dry run may create this operational lock file but does not change entity data. Apply stages all affected entity files and writes only when every result can be added. Replacement is rollback-capable but not crash-atomic, so an abrupt process or machine failure can require manual recovery.
