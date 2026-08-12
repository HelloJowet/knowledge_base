# knowledge-base-ingestion-retrieval

Retrieve web pages into portable source bundles and register their sources as knowledge-base references.

## What it provides

A retrieval bundle contains cleaned source HTML in `page.html` and versioned metadata in `retrieval.yaml`. `fetch_to_bundle` creates a uniquely named bundle under a chosen directory. `register_bundle` validates a bundle and registers or reuses its canonical reference through `knowledge-base-crud`.

Fetching does not require a knowledge base. Registration needs a `KnowledgeBase` and can run in preview mode before it writes files.

## Usage

```rust
use std::path::Path;

use knowledge_base_crud::{ApplyMode, KnowledgeBase};
use knowledge_base_ingestion_retrieval::{fetch_to_bundle, register_bundle};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let bundle = fetch_to_bundle("https://example.com/page", Path::new("temp/retrievals"))?;
    let knowledge_base = KnowledgeBase::new(Path::new("knowledge_base"));
    let outcome = register_bundle(&bundle, &knowledge_base, ApplyMode::Preview)?;
    println!("{}", outcome.reference);
    Ok(())
}
```

The fetch step follows redirects, cleans the HTML, and records the final URL, page title, retrieval time, and source language. Registration requires `page.html`, a supported `retrieval.yaml`, and absolute HTTPS requested and final URLs.

For the complete workflow, see the [ingestion guide](../../docs/ingestion.md).
