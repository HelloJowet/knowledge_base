use serde::{Deserialize, Serialize};

use super::CandidateStatement;

/// A possible production entity identified while reviewing one source article.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Candidate {
    pub key: String,
    #[serde(default)]
    pub source_names: Vec<String>,
    pub proposed_metadata: ProposedMetadata,
    #[serde(default)]
    pub unresolved_questions: Vec<String>,
    pub evidence: Vec<String>,
    pub recommended_outcome: RecommendedOutcome,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub existing_entity: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub production_entity: Option<String>,
    #[serde(default)]
    pub statements: Vec<CandidateStatement>,
    #[serde(default)]
    pub dependencies: Vec<String>,
}

/// The label, description, and classifications proposed for a candidate.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ProposedMetadata {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default)]
    pub classifications: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RecommendedOutcome {
    New,
    Existing,
    NeedsReview,
    Omit,
}
