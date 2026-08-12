# knowledge-base-extension-wikidata

This extension adds Wikidata support to a custom `knowledge-base` binary. It checks that the configured Wikidata item-ID property stores one string statement per entity and adds a command that registers a cited Wikidata item page.

The standard `knowledge-base` binary stays generic and does not include this extension. A downstream application chooses to include it when it builds its own CLI application.

## Add it to an application

```rust
use knowledge_base_cli::Application;
use knowledge_base_extension_wikidata::WikidataExtension;
use std::process::ExitCode;

fn main() -> ExitCode {
    let application = Application::builder()
        .with_extension(WikidataExtension::new())
        .build()
        .expect("valid extension configuration");

    application.run()
}
```

Enable it for a repository by adding a property binding to `extensions.yaml`:

```yaml
version: 1
extensions:
  wikidata:
    contract: 1
    properties:
      item_id_property: P21
```

Then register or reuse a reference:

```text
knowledge-base extension wikidata reference register Q42
```

The command uses the canonical Wikidata URL, prefers an English label, falls back to another available label, and reuses an existing reference with the same URL.
