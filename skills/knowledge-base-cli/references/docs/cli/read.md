# Read stored resources

Use the resource-specific commands to read individual records or an entity's Markdown context:

```sh
knowledge-base entity read Q1
knowledge-base entity-type read T1
knowledge-base property read P1
knowledge-base reference read R1
knowledge-base entity-context read Q1
```

Each command prints the requested file exactly as stored, including whether it has a trailing newline. It does not parse the file or validate the rest of the knowledge base. A missing or unreadable resource exits unsuccessfully without producing standard output.
