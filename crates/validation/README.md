# knowledge-base-validation

Loads and validates a complete knowledge-base directory.

```rust
use knowledge_base_validation::validate_repository;

let diagnostics = validate_repository("/path/to/knowledge-base");
for diagnostic in &diagnostics {
    eprintln!("{diagnostic}");
}
assert!(diagnostics.is_empty());
```

An empty `Vec<Diagnostic>` means the repository is valid. Diagnostics are structured and returned in deterministic order.

See the [validation rules](../../docs/validation.md) for the checks performed.
