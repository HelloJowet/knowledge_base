use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ArticleResult {
    pub production_entity: String,
    pub action: ArticleAction,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ArticleAction {
    Created,
    Updated,
    Unchanged,
    Omitted,
}

/// Areas reviewed or deliberately excluded while reading the source article.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Coverage {
    #[serde(default)]
    pub reviewed: Vec<CoverageReviewed>,
    #[serde(default)]
    pub excluded: Vec<CoverageExcluded>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CoverageReviewed {
    pub heading: String,
    pub location: String,
    pub scope: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CoverageExcluded {
    pub concept: String,
    pub location: String,
    pub reason: String,
}

/// A verbatim, locatable piece of source material supporting an inventory claim.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Evidence {
    pub id: String,
    pub reference: String,
    pub heading: String,
    pub location: String,
    pub excerpt: String,
}

/// Optional counts that are checked against the inventory rather than trusted as input.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct StatementCounts {
    pub total: u64,
    pub entity_valued: u64,
    pub candidates_with_statements: u64,
    #[serde(default)]
    pub by_property: BTreeMap<String, u64>,
}

/// Optional high-level totals and reviewer notes checked against the inventory.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct IngestionCandidateInventorySummary {
    #[serde(default)]
    pub outcome_counts: BTreeMap<String, u64>,
    #[serde(default)]
    pub candidate_count: u64,
    #[serde(default)]
    pub evidence_count: u64,
    #[serde(default)]
    pub draft_entity_type_count: u64,
    #[serde(default)]
    pub major_unresolved_questions: Vec<String>,
    #[serde(default)]
    pub incomplete_areas: Vec<String>,
    #[serde(default)]
    pub blockers: Vec<String>,
}
