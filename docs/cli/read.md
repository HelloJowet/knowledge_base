# Read stored resources

Use the resource-specific commands to read individual records or an entity's Markdown context:

```sh
cargo run -p knowledge-base-cli -- entity read Q1
cargo run -p knowledge-base-cli -- entity-type read T1
cargo run -p knowledge-base-cli -- property read P1
cargo run -p knowledge-base-cli -- reference read R1
cargo run -p knowledge-base-cli -- entity-context read Q1
```

Each command prints the requested file exactly as stored, including whether it has a trailing newline. It does not parse the file or validate the rest of the knowledge base. A missing or unreadable resource exits unsuccessfully without producing standard output.
