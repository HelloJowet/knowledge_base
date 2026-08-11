---
name: knowledge-base-cli
description: Use the `knowledge-base` CLI to validate a file-based knowledge base, read its resources, find entities, inspect relationships, and safely add statements. Use for tasks that should be completed through CLI commands rather than direct edits to knowledge-base YAML or Markdown files.
---

# Use the Knowledge Base CLI

Use [references/commands.md](references/commands.md) for command examples, filter formats, and statement value types.

## Choose the knowledge base

`KNOWLEDGE_BASE_PATH` must point to the folder that contains `entities/`, `properties/`, `references/`, and `id_allocation.yaml`.

If the path is already set, use it. Otherwise, identify the intended folder and include its absolute path with each command:

```sh
KNOWLEDGE_BASE_PATH=/absolute/path/to/knowledge-base knowledge-base validate
```

If more than one folder could be the knowledge base, ask the user which one to use.

## Choose the command

- Use `validate` to check the whole knowledge base.
- Use a `read` command to retrieve one entity, entity type, property, reference, or entity context file.
- Use `entity query` to find entities by property values.
- Use `entity relationships` to see an entity's direct incoming and outgoing links.
- Use `entity statement apply` to add statements to existing entities.

Check whether each command succeeds before using its output. Explain errors in plain language and do not present partial results as complete.

The CLI can add statements, but it cannot create or directly edit other resources. Use `$knowledge-base-data-model` for changes to entities, entity types, properties, references, entity context, or identifier allocation.

## Add statements safely

Only add statements when the user asked to change the knowledge base.

1. Read the target entities, properties, and references.
2. Create a statement manifest using the command reference.
3. Run `entity statement apply <manifest> --dry-run`.
4. If the preview fails or rejects any statement, explain the problem and do not apply it.
5. Apply the same unchanged manifest without `--dry-run`.
6. Run `validate` and report the added statement IDs.
