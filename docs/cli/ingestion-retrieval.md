# Retrieve and register webpages

Use retrieval commands to save a webpage as source material and register its canonical source reference. See the [ingestion guide](../ingestion.md) for how retrieval fits into the full workflow.

## Fetch a webpage

```sh
knowledge-base ingestion retrieval fetch https://example.com/page
knowledge-base ingestion retrieval fetch https://example.com/page --output-root /tmp/retrievals
```

`fetch` does not need `KNOWLEDGE_BASE_PATH`. It creates a uniquely named `fetch-...` directory under `temp/retrievals` by default, writes cleaned HTML to `page.html`, and writes source metadata to `retrieval.yaml`. The command prints the new bundle path.

## Register a source reference

```sh
knowledge-base ingestion retrieval register /tmp/retrievals/fetch-XXXXXX --dry-run
knowledge-base ingestion retrieval register /tmp/retrievals/fetch-XXXXXX
```

`register` requires `KNOWLEDGE_BASE_PATH`. It validates the bundle and either previews, creates, or reuses the reference for its canonical URL. Use `--dry-run` to inspect the YAML result without writing repository files. The bundle must contain `page.html` and supported `retrieval.yaml` metadata.
