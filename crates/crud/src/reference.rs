use crate::mutation::{FileEdit, MutationLock, commit, validate_staged};
use crate::{ApplyMode, Error, KnowledgeBase, resource};
use chrono::{DateTime, NaiveDate};
use knowledge_base_models::{IdAllocation, Reference, ReferenceId};
use knowledge_base_validation::validate_repository;
use language_tags::LanguageTag;
use serde::Serialize;
use std::fs;
use std::str::FromStr;
use url::Url;

#[derive(Clone, Copy, Debug)]
pub struct References<'a> {
    knowledge_base: &'a KnowledgeBase,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReferenceDraft {
    pub url: String,
    pub title: String,
    pub publisher: Option<String>,
    pub publication_date: Option<String>,
    pub source_language: Option<String>,
    pub retrieved_at: String,
    pub archive_url: Option<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ReferenceRegistrationStatus {
    Previewed,
    Registered,
    Existing,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ReferenceRegistrationOutcome {
    pub status: ReferenceRegistrationStatus,
    pub reference: ReferenceId,
}

impl<'a> References<'a> {
    pub(crate) fn new(knowledge_base: &'a KnowledgeBase) -> Self {
        Self { knowledge_base }
    }

    pub fn read(&self, id: &ReferenceId) -> Result<String, Error> {
        resource::read(self.knowledge_base.root(), "references", id.as_str(), "yaml")
    }

    pub fn register(&self, draft: &ReferenceDraft, mode: ApplyMode) -> Result<ReferenceRegistrationOutcome, Error> {
        validate_draft(draft)?;
        let root = self.knowledge_base.root();
        let _lock = MutationLock::acquire(root)?;

        let baseline = validate_repository(root);
        if !baseline.is_empty() {
            return Err(Error::Validation(baseline));
        }

        if let Some(reference) = find_reference_by_url(root, &draft.url)? {
            validate_staged(root, &[])?;
            return Ok(ReferenceRegistrationOutcome {
                status: ReferenceRegistrationStatus::Existing,
                reference,
            });
        }

        let (reference, edits) = plan_registration(root, draft)?;
        validate_staged(root, &edits)?;
        if mode == ApplyMode::Preview {
            return Ok(ReferenceRegistrationOutcome {
                status: ReferenceRegistrationStatus::Previewed,
                reference,
            });
        }

        commit(&edits)?;
        Ok(ReferenceRegistrationOutcome {
            status: ReferenceRegistrationStatus::Registered,
            reference,
        })
    }
}

fn validate_draft(draft: &ReferenceDraft) -> Result<(), Error> {
    validate_url("url", &draft.url)?;
    validate_nonempty("title", &draft.title)?;
    if let Some(value) = &draft.publisher {
        validate_nonempty("publisher", value)?;
    }
    if let Some(value) = &draft.publication_date {
        validate_nonempty("publication_date", value)?;
        if !valid_partial_date(value) {
            return Err(Error::InvalidRequest("publication_date must be a valid YYYY, YYYY-MM, or YYYY-MM-DD date".to_owned()));
        }
    }
    if let Some(value) = &draft.source_language {
        validate_nonempty("source_language", value)?;
        if value.parse::<LanguageTag>().is_err() {
            return Err(Error::InvalidRequest("source_language must be a well-formed BCP 47 tag".to_owned()));
        }
    }
    if let Some(value) = &draft.archive_url {
        validate_url("archive_url", value)?;
    }
    if DateTime::parse_from_rfc3339(&draft.retrieved_at).is_err() {
        return Err(Error::InvalidRequest("retrieved_at must be an RFC 3339 timestamp".to_owned()));
    }
    Ok(())
}

fn validate_nonempty(field: &str, value: &str) -> Result<(), Error> {
    if value.trim().is_empty() {
        Err(Error::InvalidRequest(format!("{field} must not be empty")))
    } else {
        Ok(())
    }
}

fn validate_url(field: &str, value: &str) -> Result<(), Error> {
    if Url::parse(value).is_err() {
        Err(Error::InvalidRequest(format!("{field} must be an absolute URL")))
    } else {
        Ok(())
    }
}

fn valid_partial_date(value: &str) -> bool {
    match value.len() {
        4 => value.bytes().all(|byte| byte.is_ascii_digit()),
        7 => value
            .get(..4)
            .zip(value.get(5..))
            .filter(|(year, month)| year.bytes().all(|byte| byte.is_ascii_digit()) && month.bytes().all(|byte| byte.is_ascii_digit()))
            .and_then(|(year, month)| year.parse::<i32>().ok().zip(month.parse::<u32>().ok()))
            .is_some_and(|(year, month)| value.as_bytes().get(4) == Some(&b'-') && NaiveDate::from_ymd_opt(year, month, 1).is_some()),
        10 => NaiveDate::parse_from_str(value, "%Y-%m-%d").is_ok(),
        _ => false,
    }
}

fn find_reference_by_url(root: &std::path::Path, url: &str) -> Result<Option<ReferenceId>, Error> {
    let directory = root.join("references");
    let mut references = Vec::new();
    for entry in fs::read_dir(&directory).map_err(|source| Error::Read { path: directory.clone(), source })? {
        let entry = entry.map_err(|source| Error::Read { path: directory.clone(), source })?;
        let path = entry.path();
        if entry.file_type().map_err(|source| Error::Read { path: path.clone(), source })?.is_file() && path.extension().and_then(|extension| extension.to_str()) == Some("yaml") {
            let source = fs::read(&path).map_err(|source| Error::Read { path: path.clone(), source })?;
            let reference = serde_yaml::from_slice::<Reference>(&source).map_err(|source| Error::ParseReference { path, source })?;
            references.push(reference);
        }
    }
    references.sort_by(|left, right| left.id.cmp(&right.id));
    Ok(references.into_iter().find(|reference| reference.url == url).map(|reference| reference.id))
}

fn plan_registration(root: &std::path::Path, draft: &ReferenceDraft) -> Result<(ReferenceId, Vec<FileEdit>), Error> {
    let allocation_path = root.join("id_allocation.yaml");
    let allocation_source = fs::read(&allocation_path).map_err(|source| Error::Read {
        path: allocation_path.clone(),
        source,
    })?;
    let mut allocation = serde_yaml::from_slice::<IdAllocation>(&allocation_source).map_err(|source| Error::ParseAllocation {
        path: allocation_path.clone(),
        source,
    })?;
    let next = allocation.next.reference;
    let incremented = next
        .checked_add(1)
        .ok_or_else(|| Error::InvalidRequest("cannot allocate another reference identifier".to_owned()))?;
    let reference = ReferenceId::from_str(&format!("R{next}")).expect("positive allocation counters form valid reference identifiers");
    let reference_path = resource::path(root, "references", reference.as_str(), "yaml");
    allocation.next.reference = incremented;
    let reference_source = serde_yaml::to_string(&Reference {
        id: reference.clone(),
        url: draft.url.clone(),
        title: draft.title.clone(),
        publisher: draft.publisher.clone(),
        publication_date: draft.publication_date.clone(),
        source_language: draft.source_language.clone(),
        retrieved_at: draft.retrieved_at.clone(),
        archive_url: draft.archive_url.clone(),
    })
    .map_err(|error| Error::Edit {
        path: reference_path.clone(),
        message: format!("cannot serialize reference: {error}"),
    })?;
    let updated_allocation = serde_yaml::to_string(&allocation).map_err(|error| Error::Edit {
        path: allocation_path.clone(),
        message: format!("cannot serialize identifier allocation: {error}"),
    })?;
    Ok((
        reference,
        vec![
            FileEdit {
                path: reference_path,
                original: None,
                replacement: reference_source.into_bytes(),
            },
            FileEdit {
                path: allocation_path,
                original: Some(allocation_source),
                replacement: updated_allocation.into_bytes(),
            },
        ],
    ))
}
