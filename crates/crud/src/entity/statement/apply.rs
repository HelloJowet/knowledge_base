use super::planner::StatementPlanner;
use super::{ApplyMode, ApplyStatementsOutcome, StatementBatch, StatementResultStatus, validate_batch};
use crate::Error;
use crate::entity::Entities;
use crate::mutation::{MutationLock, commit, validate_staged};

impl Entities<'_> {
    pub fn apply_statements(&self, batch: &StatementBatch, mode: ApplyMode) -> Result<ApplyStatementsOutcome, Error> {
        validate_batch(batch)?;
        let _lock = MutationLock::acquire(self.knowledge_base.root())?;

        let baseline = self.knowledge_base.validate();
        if !baseline.is_empty() {
            return Err(Error::Validation(baseline));
        }

        let plan = StatementPlanner::new(self.knowledge_base.root(), batch).plan()?;
        validate_staged(self.knowledge_base.root(), &plan.edits, self.knowledge_base.additional_validators())?;

        if !plan.all_new() {
            return Ok(ApplyStatementsOutcome::NotApplied(plan.results));
        }
        if mode == ApplyMode::Preview {
            return Ok(ApplyStatementsOutcome::Previewed(plan.results));
        }

        commit(&plan.edits)?;
        let mut results = plan.results;
        for result in &mut results {
            debug_assert_eq!(result.status, StatementResultStatus::WouldAdd);
            result.status = StatementResultStatus::Added;
        }
        Ok(ApplyStatementsOutcome::Applied(results))
    }
}
