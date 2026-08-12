use super::planner::StatementPlanner;
use super::{ApplyStatementsOutcome, StatementBatch, StatementResultStatus, validate_batch};
use crate::Error;
use crate::write::execution::{MutationDisposition, PlannedMutation, execute};
use crate::write::{StatementMutations, WriteMode};

impl StatementMutations<'_> {
    pub fn apply(&self, batch: &StatementBatch, mode: WriteMode) -> Result<ApplyStatementsOutcome, Error> {
        validate_batch(batch)?;
        let (mut results, disposition) = execute(self.repository, mode, || {
            let plan = StatementPlanner::new(self.repository.root(), batch).plan()?;
            let can_commit = plan.all_new();
            Ok(PlannedMutation::new(plan.results, plan.edits, can_commit))
        })?;
        match disposition {
            MutationDisposition::NotCommitted => Ok(ApplyStatementsOutcome::NotApplied(results)),
            MutationDisposition::Previewed => Ok(ApplyStatementsOutcome::Previewed(results)),
            MutationDisposition::Committed => {
                for result in &mut results {
                    debug_assert_eq!(result.status, StatementResultStatus::WouldAdd);
                    result.status = StatementResultStatus::Added;
                }
                Ok(ApplyStatementsOutcome::Applied(results))
            }
        }
    }
}
