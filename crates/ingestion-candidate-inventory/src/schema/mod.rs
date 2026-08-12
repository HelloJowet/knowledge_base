//! The strict YAML schema used by ingestion candidate inventories.

mod candidate;
mod document;
mod ontology;
mod supporting;
mod value;

pub use candidate::{Candidate, ProposedMetadata, RecommendedOutcome};
pub use document::IngestionCandidateInventory;
pub use ontology::{DraftEntityType, DraftProperty};
pub use supporting::{ArticleAction, ArticleResult, Coverage, CoverageExcluded, CoverageReviewed, Evidence, IngestionCandidateInventorySummary, StatementCounts};
pub use value::{CandidateQualifier, CandidateStatement, CandidateValue, DatePrecision};
