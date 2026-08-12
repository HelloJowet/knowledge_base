pub mod schema;
mod validation;

pub use schema::{
    ArticleAction, ArticleResult, Candidate, CandidateQualifier, CandidateStatement, CandidateValue, Coverage, CoverageExcluded, CoverageReviewed, DatePrecision, DraftEntityType,
    DraftProperty, Evidence, IngestionCandidateInventory, IngestionCandidateInventorySummary, ProposedMetadata, RecommendedOutcome, StatementCounts,
};
pub use validation::validate_ingestion_candidate_inventory;
