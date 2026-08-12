use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RetrievalMetadata {
    pub schema_version: u8,
    pub requested_url: String,
    pub url: String,
    pub title: String,
    pub source_language: String,
    pub retrieved_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub publisher: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub publication_date: Option<String>,
}

impl RetrievalMetadata {
    pub const SCHEMA_VERSION: u8 = 1;
}
