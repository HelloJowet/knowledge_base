use crate::{ReferenceId, deserialize_optional_non_null};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Reference {
    pub id: ReferenceId,
    pub url: String,
    pub title: String,
    #[serde(default, deserialize_with = "deserialize_optional_non_null", skip_serializing_if = "Option::is_none")]
    pub publisher: Option<String>,
    #[serde(default, deserialize_with = "deserialize_optional_non_null", skip_serializing_if = "Option::is_none")]
    pub publication_date: Option<String>,
    #[serde(default, deserialize_with = "deserialize_optional_non_null", skip_serializing_if = "Option::is_none")]
    pub source_language: Option<String>,
    pub retrieved_at: String,
    #[serde(default, deserialize_with = "deserialize_optional_non_null", skip_serializing_if = "Option::is_none")]
    pub archive_url: Option<String>,
}
