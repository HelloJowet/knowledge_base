# knowledge-base-cli

Command-line interface for validating and reading a file-based knowledge base.

Set the knowledge-base root for every command:

```sh
export KNOWLEDGE_BASE_PATH=/path/to/knowledge-base
```

Validate the configured knowledge base:

```sh
cargo run -p knowledge-base-cli -- validate
```

Read a stored resource exactly as written:

```sh
cargo run -p knowledge-base-cli -- entity read Q1
cargo run -p knowledge-base-cli -- entity-type read T1
cargo run -p knowledge-base-cli -- property read P1
cargo run -p knowledge-base-cli -- reference read R1
cargo run -p knowledge-base-cli -- entity-context read Q1
```

Build the standalone `knowledge-base` executable with:

```sh
cargo build --release -p knowledge-base-cli
KNOWLEDGE_BASE_PATH=/path/to/knowledge-base target/release/knowledge-base validate
```

Commands exit unsuccessfully when `KNOWLEDGE_BASE_PATH` is unset or empty, a requested file cannot be read, or validation diagnostics are found.
