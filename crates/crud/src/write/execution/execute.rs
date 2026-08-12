use super::{FileEdit, MutationLock, commit, validate_staged};
use crate::write::WriteMode;
use crate::{Error, KnowledgeBaseRepository};

/// Resource-specific planners return their outcome data and the exact files they
/// expect to replace. The executor owns every filesystem safety boundary.
pub(crate) struct PlannedMutation<T> {
    pub(crate) value: T,
    pub(crate) edits: Vec<FileEdit>,
    pub(crate) can_commit: bool,
}

impl<T> PlannedMutation<T> {
    pub(crate) fn new(value: T, edits: Vec<FileEdit>, can_commit: bool) -> Self {
        Self { value, edits, can_commit }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum MutationDisposition {
    Previewed,
    Committed,
    NotCommitted,
}

pub(crate) fn execute<T>(
    repository: &KnowledgeBaseRepository,
    mode: WriteMode,
    plan: impl FnOnce() -> Result<PlannedMutation<T>, Error>,
) -> Result<(T, MutationDisposition), Error> {
    let _lock = MutationLock::acquire(repository.root())?;
    let baseline = repository.validate();
    if !baseline.is_empty() {
        return Err(Error::Validation(baseline));
    }

    let planned = plan()?;
    validate_staged(repository.root(), &planned.edits, repository.validators())?;
    if !planned.can_commit {
        return Ok((planned.value, MutationDisposition::NotCommitted));
    }
    if mode == WriteMode::Preview {
        return Ok((planned.value, MutationDisposition::Previewed));
    }

    commit(&planned.edits)?;
    Ok((planned.value, MutationDisposition::Committed))
}
