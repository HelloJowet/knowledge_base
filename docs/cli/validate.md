# Validate a knowledge base

Validate the knowledge base configured by `KNOWLEDGE_BASE_PATH`:

```sh
cargo run -p knowledge-base-cli -- validate
```

The command checks file structure, values, ontology rules, cross-references, provenance, and Markdown citations. It reports diagnostics without changing knowledge-base files and exits unsuccessfully when any diagnostics are found.

See the [`knowledge-base-validation`](../../crates/validation/README.md) crate for details about the validation library.
