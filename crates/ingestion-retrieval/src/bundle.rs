use std::{
    env, fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use tempfile::Builder;

use crate::RetrievalMetadata;

pub fn save_bundle(html: &str, metadata: &RetrievalMetadata) -> Result<PathBuf> {
    save_bundle_in(html, metadata, &env::temp_dir())
}

pub fn save_bundle_in(html: &str, metadata: &RetrievalMetadata, output_root: &Path) -> Result<PathBuf> {
    fs::create_dir_all(output_root).with_context(|| format!("could not create retrieval bundle root {}", output_root.display()))?;
    let temporary_directory = Builder::new()
        .prefix("fetch-")
        .tempdir_in(output_root)
        .context("could not create temporary retrieval directory")?;
    let directory = temporary_directory.path();
    let html_path = directory.join("page.html");
    let metadata_path = directory.join("retrieval.yaml");

    fs::write(&html_path, html).with_context(|| format!("could not write temporary file {}", html_path.display()))?;
    let yaml = serde_yaml::to_string(metadata).context("could not serialize retrieval metadata")?;
    fs::write(&metadata_path, yaml).with_context(|| format!("could not write temporary file {}", metadata_path.display()))?;

    Ok(temporary_directory.keep())
}

pub fn load_metadata(directory: &Path) -> Result<RetrievalMetadata> {
    let path = directory.join("retrieval.yaml");
    let yaml = fs::read_to_string(&path).with_context(|| format!("could not read retrieval metadata {}", path.display()))?;
    serde_yaml::from_str(&yaml).with_context(|| format!("could not parse retrieval metadata {}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn metadata() -> RetrievalMetadata {
        RetrievalMetadata {
            schema_version: RetrievalMetadata::SCHEMA_VERSION,
            requested_url: "https://example.com/start".into(),
            url: "https://example.com/page".into(),
            title: "Example page".into(),
            source_language: "en".into(),
            retrieved_at: "2026-07-30T14:05:57Z".into(),
            publisher: None,
            publication_date: None,
        }
    }

    #[test]
    fn saves_each_fetch_in_a_distinct_persistent_bundle() {
        let root = tempdir().unwrap();
        let first = save_bundle_in("<html>first</html>", &metadata(), root.path()).unwrap();
        let second = save_bundle_in("<html>second</html>", &metadata(), root.path()).unwrap();

        assert_ne!(first, second);
        assert_eq!(fs::read_to_string(first.join("page.html")).unwrap(), "<html>first</html>");
        assert_eq!(load_metadata(&first).unwrap().url, metadata().url);
        assert!(second.join("page.html").is_file());
        assert!(second.join("retrieval.yaml").is_file());
    }
}
