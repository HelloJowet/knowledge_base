//! Webpage retrieval bundles for knowledge-base ingestion.
//!
//! A bundle contains cleaned source HTML in `page.html` and versioned metadata
//! in `retrieval.yaml`. A valid bundle can be registered as a canonical
//! knowledge-base reference.

#![forbid(unsafe_code)]

mod bundle;
mod fetch;
mod html;
mod metadata;
mod registration;

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

pub use bundle::{load_metadata, save_bundle, save_bundle_in};
pub use fetch::{FetchedPage, fetch_and_clean};
pub use metadata::RetrievalMetadata;
pub use registration::register_bundle;

/// Fetches, cleans, and stores a webpage in a newly-created retrieval bundle.
pub fn fetch_to_bundle(url: &str, output_root: &Path) -> Result<PathBuf> {
    let page = fetch_and_clean(url).context("failed to clean web page")?;
    let metadata = RetrievalMetadata {
        schema_version: RetrievalMetadata::SCHEMA_VERSION,
        requested_url: url.to_owned(),
        url: page.url,
        title: page.title,
        source_language: "en".to_owned(),
        retrieved_at: page.retrieved_at,
        publisher: None,
        publication_date: None,
    };
    save_bundle_in(&page.html, &metadata, output_root).context("failed to save fetched page")
}
