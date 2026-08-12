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

## Domain validators

Domain crates can add read-only rules without adding domain knowledge to this crate. Implement `KnowledgeBaseValidator` (or pass a closure) and call `validate_repository_with`:

```rust
use knowledge_base_validation::{Diagnostic, KnowledgeBaseValidator, ValidationContext, ValidationLayer, validate_repository_with};
use std::path::PathBuf;

struct TransportValidator;

impl KnowledgeBaseValidator for TransportValidator {
    fn validate(&self, context: &ValidationContext<'_>) -> Vec<Diagnostic> {
        let _entity = context.snapshot().entities().get(&"Q1".parse().unwrap());
        vec![Diagnostic {
            layer: ValidationLayer::Domain,
            path: PathBuf::from("entities/Q1.yaml"),
            line: None,
            identifier: Some("Q1".to_owned()),
            message: "example transport rule".to_owned(),
        }]
    }
}

let validator = TransportValidator;
let diagnostics = validate_repository_with("/path/to/knowledge-base", [&validator]);
```

Each validator receives the repository root and the same immutable structured snapshot. It must report paths relative to that root, because mutations validate a temporary staged copy before committing. Built-in validation runs first; when the snapshot loads, every domain validator then runs in registration order. If it cannot load, domain validators are skipped and the generic schema diagnostics are retained. The combined results are sorted deterministically by path, line, identifier, message, and layer. `validate_repository` remains the built-in-only convenience function.
