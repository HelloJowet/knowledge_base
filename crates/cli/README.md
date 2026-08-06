# knowledge-base-cli

Command-line interface for validating a knowledge-base directory.

```sh
cargo run -p knowledge-base-cli -- validate fixtures/valid/minimal
```

Build the standalone `knowledge-base` executable with:

```sh
cargo build --release -p knowledge-base-cli
target/release/knowledge-base validate /path/to/knowledge-base
```

The command exits with `0` for valid data and `1` when diagnostics are found.
