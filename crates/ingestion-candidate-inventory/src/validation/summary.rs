use std::{collections::BTreeMap, path::Path};

use super::diagnostics::{DiagnosticFactory, ValidationReport};
use crate::{IngestionCandidateInventory, RecommendedOutcome, StatementCounts};

pub(crate) fn validate(path: &Path, inventory: &IngestionCandidateInventory, report: &mut ValidationReport) {
    if let Some(counts) = &inventory.statement_counts {
        validate_statement_counts(path, inventory, counts, report);
    }
    if let Some(summary) = &inventory.summary {
        let outcomes = inventory.candidates.iter().fold(BTreeMap::<String, u64>::new(), |mut counts, candidate| {
            *counts.entry(outcome_name(candidate.recommended_outcome).to_owned()).or_default() += 1;
            counts
        });
        compare(path, report, "summary.outcome_counts", &summary.outcome_counts, &outcomes);
        for (field, actual, expected) in [
            ("candidate_count", summary.candidate_count, inventory.candidates.len() as u64),
            ("evidence_count", summary.evidence_count, inventory.evidence.len() as u64),
            ("draft_entity_type_count", summary.draft_entity_type_count, inventory.draft_entity_types.len() as u64),
        ] {
            if actual != expected {
                report.push(DiagnosticFactory::domain(path, format!("summary.{field}"), format!("must be {expected}")));
            }
        }
        for (field, values) in [
            ("major_unresolved_questions", &summary.major_unresolved_questions),
            ("incomplete_areas", &summary.incomplete_areas),
            ("blockers", &summary.blockers),
        ] {
            super::rules::validate_text_list(report, path, &format!("summary.{field}"), values);
        }
    }
}

fn validate_statement_counts(path: &Path, inventory: &IngestionCandidateInventory, counts: &StatementCounts, report: &mut ValidationReport) {
    let statements: Vec<_> = inventory.candidates.iter().flat_map(|candidate| candidate.statements.iter()).collect();
    let by_property = statements.iter().fold(BTreeMap::<String, u64>::new(), |mut counts, statement| {
        *counts.entry(statement.property.clone()).or_default() += 1;
        counts
    });
    for (field, actual, expected) in [
        ("total", counts.total, statements.len() as u64),
        (
            "entity_valued",
            counts.entity_valued,
            statements.iter().filter(|statement| statement.value.entity_id().is_some()).count() as u64,
        ),
        (
            "candidates_with_statements",
            counts.candidates_with_statements,
            inventory.candidates.iter().filter(|candidate| !candidate.statements.is_empty()).count() as u64,
        ),
    ] {
        if actual != expected {
            report.push(DiagnosticFactory::domain(path, format!("statement_counts.{field}"), format!("must be {expected}")));
        }
    }
    compare(path, report, "statement_counts.by_property", &counts.by_property, &by_property);
}
fn compare(path: &Path, report: &mut ValidationReport, context: &str, actual: &BTreeMap<String, u64>, expected: &BTreeMap<String, u64>) {
    if actual != expected {
        report.push(DiagnosticFactory::domain(path, context, format!("must be {expected:?}")));
    }
}
fn outcome_name(outcome: RecommendedOutcome) -> &'static str {
    match outcome {
        RecommendedOutcome::New => "new",
        RecommendedOutcome::Existing => "existing",
        RecommendedOutcome::NeedsReview => "needs_review",
        RecommendedOutcome::Omit => "omit",
    }
}
