---
name: knowledge-base-data-model
description: Create, edit, review, and explain YAML and Markdown files in a file-based knowledge base. Use when working directly with entities, entity types, properties, statements, references, identifiers, typed values, source citations, images, or entity context. Prefer the CLI skill for supported reads, queries, validation, ingestion, reference registration, and statement application.
---

# Work with Knowledge Base Data

Read [the canonical data-model reference](references/docs/data-model.md) before creating or changing knowledge-base files.

## Workflow

1. Identify the requested resource or fact and locate the knowledge-base root.
2. Inspect the target and its related entity types, properties, entities, references, context, and allocation state. Check whether the concept already exists.
3. Prefer `$knowledge-base-cli` when the CLI supports the complete operation, especially for reference registration and adding statements to existing entities.
4. Otherwise, make the smallest valid direct edit. Preserve existing style and unrelated content, and add supporting sources before citing them.
5. Allocate repository-wide and statement identifiers according to the data-model reference without reusing gaps or moving counters backward.
6. Re-read the changed resources and validate the complete knowledge base with the CLI when it is available. Fix only diagnostics introduced by the change; if validation cannot run, perform a careful schema review and report that validation remains outstanding.
