# Register references

Use `reference register` to allocate a new reference or reuse an existing one with the exact same URL:

```sh
knowledge-base reference register \
  --url https://example.org/source \
  --title 'Example source' \
  --publisher 'Example Publisher' \
  --publication-date 2026-08 \
  --source-language en \
  --archive-url https://archive.example.org/source
```

`--url` and `--title` are required. `publisher`, `publication_date`, `source_language`, and `archive_url` are optional. The command supplies the current UTC `retrieved_at` timestamp; it does not retrieve or verify the remote page.

Use `--dry-run` to validate the same operation without writing files:

```sh
knowledge-base reference register \
  --url https://example.org/source \
  --title 'Example source' \
  --dry-run
```

The command writes deterministic YAML containing `status` and `reference`. Status is `registered` after a write, `previewed` for a successful dry run, or `existing` when an exactly equal stored URL already exists. Existing references are returned without changing their metadata or the allocation counter.

Registration validates the baseline repository and the staged result under the shared mutation lock. Invalid metadata, an invalid repository, allocation exhaustion, concurrent changes, or a failed staged validation exits unsuccessfully without changing knowledge-base data. A dry run may create the operational `.knowledge-base.lock` file but does not change repository records.
