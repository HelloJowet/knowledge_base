---
name: knowledge-base-data-model
description: Create, edit, review, and explain YAML and Markdown files in a file-based knowledge base. Use when working with entities, entity types, properties, statements, references, identifiers, typed values, source citations, or entity context. Use the CLI skill instead when the task is only querying, reading, validating, or adding statements to existing entities.
---

# Work with Knowledge Base Data

A knowledge base represents subjects as entities and describes them through structured, sourced statements. Read [references/schema.md](references/schema.md) for an overview of the model, exact file shapes, and validation rules before changing data.

## Workflow

1. **Understand the change.** Identify the entity, classification, statement, or reference the user wants to add or update. If the task only adds statements to existing entities, prefer `$knowledge-base-cli` and its preview-first workflow when available.
2. **Inspect related files.** Find the knowledge-base root, read the target and its related types, properties, entities, and references, and check whether the concept already exists.
3. **Make the smallest valid edit.** Follow the examples and field names in the schema reference exactly. Add source records before citing them, preserve the file's existing style, and avoid unrelated cleanup.
4. **Allocate new IDs carefully.** For a new entity, property, reference, or entity type, use and increment the matching counter in `id_allocation.yaml`. For a statement, use the next available `S<n>` within its entity.
5. **Validate the result.** Re-read the changed files and check their related resources. If `knowledge-base` is installed, run:

```sh
KNOWLEDGE_BASE_PATH=/absolute/path knowledge-base validate
```

Fix diagnostics introduced by the change and rerun validation. Do not silently fix unrelated problems. If the executable is unavailable, perform a careful schema review and tell the user that full validation is still outstanding; do not install it.
