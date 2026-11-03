use crate::types::{JobSubmissionRequest, WorkflowSnapshot};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DomainRuleViolation {
    MissingParent,
    ParentNotFound(u64),
    InvalidParentType {
        expected: &'static str,
        actual: String,
    },
    TerminalConstraintViolation {
        parent_id: u64,
    },
    DuplicateSimulationUnderParent {
        parent_id: u64,
    },
}

pub trait WorkflowPolicy {
    fn validate_submit(
        &self,
        request: &JobSubmissionRequest,
        snapshot: &WorkflowSnapshot,
    ) -> Result<(), DomainRuleViolation>;
}
