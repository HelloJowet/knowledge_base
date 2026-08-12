use serde::{Deserialize, Serialize};

use super::{ArticleResult, Candidate, Coverage, DraftEntityType, DraftProperty, Evidence, IngestionCandidateInventorySummary, StatementCounts};

/// The complete, reviewable handoff for one retrieved source article.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct IngestionCandidateInventory {
    pub source_reference: String,
    pub source_file: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub coverage: Option<Coverage>,
    #[serde(default)]
    pub evidence: Vec<Evidence>,
    pub draft_entity_types: Vec<DraftEntityType>,
    pub draft_properties: Vec<DraftProperty>,
    pub article_results: Vec<ArticleResult>,
    pub candidates: Vec<Candidate>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub statement_counts: Option<StatementCounts>,
    #[serde(default)]
    pub review_notes: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub summary: Option<IngestionCandidateInventorySummary>,
}
