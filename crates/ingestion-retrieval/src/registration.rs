use std::path::Path;

use anyhow::{Context, Result, ensure};
use chrono::{DateTime, NaiveDate};
use knowledge_base_crud::KnowledgeBaseRepository;
use knowledge_base_crud::write::{ReferenceDraft, ReferenceRegistrationOutcome, WriteMode};
use url::Url;

use crate::{RetrievalMetadata, load_metadata};

/// Registers or reuses the reference represented by a retrieval bundle.
pub fn register_bundle(bundle_directory: &Path, repository: &KnowledgeBaseRepository, mode: WriteMode) -> Result<ReferenceRegistrationOutcome> {
    ensure!(bundle_directory.is_dir(), "retrieval bundle is not a directory: {}", bundle_directory.display());
    let page_path = bundle_directory.join("page.html");
    ensure!(page_path.is_file(), "retrieval bundle is missing {}", page_path.display());

    let metadata = load_metadata(bundle_directory)?;
    validate_metadata(&metadata)?;
    repository
        .write()
        .references()
        .register(
            &ReferenceDraft {
                url: metadata.url,
                title: metadata.title,
                publisher: metadata.publisher,
                publication_date: metadata.publication_date,
                source_language: Some(metadata.source_language),
                retrieved_at: metadata.retrieved_at,
                archive_url: None,
            },
            mode,
        )
        .context("could not register retrieval bundle reference")
}

fn validate_metadata(metadata: &RetrievalMetadata) -> Result<()> {
    ensure!(
        metadata.schema_version == RetrievalMetadata::SCHEMA_VERSION,
        "unsupported retrieval metadata schema version {}",
        metadata.schema_version
    );
    validate_https_url("requested_url", &metadata.requested_url)?;
    validate_https_url("url", &metadata.url)?;
    ensure!(!metadata.title.trim().is_empty(), "title must not be empty");
    ensure!(!metadata.source_language.trim().is_empty(), "source_language must not be empty");
    validate_utc_datetime(&metadata.retrieved_at)?;
    if let Some(publisher) = &metadata.publisher {
        ensure!(!publisher.trim().is_empty(), "publisher must not be empty");
    }
    if let Some(publication_date) = &metadata.publication_date {
        validate_publication_date(publication_date)?;
    }
    Ok(())
}

fn validate_utc_datetime(value: &str) -> Result<()> {
    ensure!(value.ends_with('Z'), "retrieved_at must be a UTC timestamp ending in Z");
    let retrieved_at = DateTime::parse_from_rfc3339(value).context("retrieved_at must be a valid RFC 3339 timestamp")?;
    ensure!(retrieved_at.offset().local_minus_utc() == 0, "retrieved_at must use UTC");
    Ok(())
}

fn validate_https_url(field: &str, value: &str) -> Result<()> {
    let url = Url::parse(value).with_context(|| format!("{field} must be a valid URL"))?;
    ensure!(
        value.starts_with("https://") && url.scheme() == "https" && url.host_str().is_some(),
        "{field} must be an absolute HTTPS URL"
    );
    Ok(())
}

fn validate_publication_date(value: &str) -> Result<()> {
    ensure!(matches!(value.len(), 4 | 7 | 10), "publication_date must use YYYY, YYYY-MM, or YYYY-MM-DD");
    match value.len() {
        4 => {
            value.parse::<i32>().context("publication_date contains an invalid year")?;
        }
        7 => {
            ensure!(value.as_bytes().get(4) == Some(&b'-'), "publication_date must use YYYY-MM");
            NaiveDate::from_ymd_opt(value[..4].parse()?, value[5..].parse()?, 1).context("publication_date contains an invalid month")?;
        }
        10 => {
            NaiveDate::parse_from_str(value, "%Y-%m-%d").context("publication_date contains an invalid date")?;
        }
        _ => unreachable!(),
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;
    use crate::save_bundle_in;
    use knowledge_base_crud::write::ReferenceRegistrationStatus;
    use tempfile::tempdir;

    fn write_repository(root: &Path, next_reference: u64) {
        for directory in ["entities", "entity_types", "properties", "references"] {
            fs::create_dir_all(root.join(directory)).unwrap();
        }
        fs::write(
            root.join("id_allocation.yaml"),
            format!("version: 1\nnext: {{entity: 1, property: 1, reference: {next_reference}, entity_type: 1}}\n"),
        )
        .unwrap();
    }

    fn metadata() -> RetrievalMetadata {
        RetrievalMetadata {
            schema_version: RetrievalMetadata::SCHEMA_VERSION,
            requested_url: "https://example.com/start".into(),
            url: "https://example.com/page".into(),
            title: "Example".into(),
            publisher: None,
            publication_date: None,
            source_language: "en".into(),
            retrieved_at: "2026-08-03T13:20:03Z".into(),
        }
    }

    #[test]
    fn registers_and_reuses_bundle_references() {
        let repository = tempdir().unwrap();
        write_repository(repository.path(), 1);
        let bundles = tempdir().unwrap();
        let bundle = save_bundle_in("page", &metadata(), bundles.path()).unwrap();
        let knowledge_base = KnowledgeBaseRepository::new(repository.path());

        let created = register_bundle(&bundle, &knowledge_base, WriteMode::Commit).unwrap();
        assert_eq!(created.status, ReferenceRegistrationStatus::Registered);
        assert_eq!(created.reference.to_string(), "R1");
        let existing = register_bundle(&bundle, &knowledge_base, WriteMode::Commit).unwrap();
        assert_eq!(existing.status, ReferenceRegistrationStatus::Existing);
    }

    #[test]
    fn previews_without_writing_and_rejects_invalid_bundles() {
        let repository = tempdir().unwrap();
        write_repository(repository.path(), 1);
        let bundles = tempdir().unwrap();
        let mut invalid = metadata();
        invalid.schema_version = 2;
        let bundle = save_bundle_in("page", &invalid, bundles.path()).unwrap();
        let knowledge_base = KnowledgeBaseRepository::new(repository.path());
        assert!(register_bundle(&bundle, &knowledge_base, WriteMode::Commit).is_err());

        let missing = bundles.path().join("missing");
        assert!(register_bundle(&missing, &knowledge_base, WriteMode::Commit).is_err());

        let valid = save_bundle_in("page", &metadata(), bundles.path()).unwrap();
        let preview = register_bundle(&valid, &knowledge_base, WriteMode::Preview).unwrap();
        assert_eq!(preview.status, ReferenceRegistrationStatus::Previewed);
        assert!(!repository.path().join("references/R1.yaml").exists());
    }
}
