# knowledge-base-validation

Loads and validates a complete knowledge-base directory.

```rust
use knowledge_base_validation::validate_repository;

let diagnostics = validate_repository("/path/to/knowledge-base");
for diagnostic in &diagnostics {
    eprintln!("{diagnostic}");
}
```

The validator checks:

- required directories, file names, YAML syntax, and the [data model](../../docs/data-model.md);
- identifier uniqueness and cross-references;
- property applicability, value types, target types, cardinality, and qualifiers;
- URLs, dates, timestamps, decimals, coordinates, and allocation counters;
- required references on structured data; and
- entity Markdown file names, footnotes, and reference links.

Diagnostics include the path, line when available, validation layer, related identifier, and message. An empty list means the knowledge base is valid.
