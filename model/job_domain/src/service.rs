use crate::policy::{DomainRuleViolation, WorkflowPolicy};
use crate::types::{JobSubmissionRequest, WorkflowSnapshot};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DomainAction {
    Submit(JobSubmissionRequest),
}

#[derive(Debug)]
pub struct JobDomainService<P> {
    policy: P,
}

impl<P> JobDomainService<P>
where
    P: WorkflowPolicy,
{
    pub fn new(policy: P) -> Self {
        Self { policy }
    }

    /// ルール検証のみを担当し、実行基盤への投入は呼び出し側に委譲する。
    pub fn plan_submit(
        &self,
        request: JobSubmissionRequest,
        snapshot: &WorkflowSnapshot,
    ) -> Result<DomainAction, DomainRuleViolation> {
        self.policy.validate_submit(&request, snapshot)?;
        Ok(DomainAction::Submit(request))
    }
}
