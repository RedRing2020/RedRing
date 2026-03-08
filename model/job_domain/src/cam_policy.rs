use crate::policy::{DomainRuleViolation, WorkflowPolicy};
use crate::types::{JobSubmissionRequest, WorkflowSnapshot};

pub const JOB_TYPE_CAM_PROCESS: &str = "cam_process";
pub const JOB_TYPE_CUTTING_SIMULATION: &str = "cutting_simulation";

#[derive(Debug, Default, Clone, Copy)]
pub struct CamWorkflowPolicy;

impl WorkflowPolicy for CamWorkflowPolicy {
    fn validate_submit(
        &self,
        request: &JobSubmissionRequest,
        snapshot: &WorkflowSnapshot,
    ) -> Result<(), DomainRuleViolation> {
        match request.parent_job_id {
            Some(parent_id) => {
                let parent = snapshot
                    .find(parent_id)
                    .ok_or(DomainRuleViolation::ParentNotFound(parent_id))?;

                // SIM投入時は親がCAMであること、かつ同親配下で重複しないことを保証する。
                if request.job_type == JOB_TYPE_CUTTING_SIMULATION {
                    if parent.job_type != JOB_TYPE_CAM_PROCESS {
                        return Err(DomainRuleViolation::InvalidParentType {
                            expected: JOB_TYPE_CAM_PROCESS,
                            actual: parent.job_type.clone(),
                        });
                    }

                    let has_sim_child = snapshot
                        .children_of(parent_id)
                        .any(|child| child.job_type == JOB_TYPE_CUTTING_SIMULATION);

                    if has_sim_child {
                        return Err(DomainRuleViolation::DuplicateSimulationUnderParent {
                            parent_id,
                        });
                    }
                } else if parent.job_type == JOB_TYPE_CUTTING_SIMULATION {
                    // SIMは末尾工程。SIM配下への投入は禁止。
                    return Err(DomainRuleViolation::TerminalConstraintViolation { parent_id });
                }
            }
            None => {
                if request.job_type == JOB_TYPE_CUTTING_SIMULATION {
                    return Err(DomainRuleViolation::MissingParent);
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{CamWorkflowPolicy, JOB_TYPE_CAM_PROCESS, JOB_TYPE_CUTTING_SIMULATION};
    use crate::policy::{DomainRuleViolation, WorkflowPolicy};
    use crate::types::{JobNode, JobSubmissionRequest, WorkflowSnapshot};

    fn cam_job(id: u64, parent_job_id: Option<u64>) -> JobNode {
        JobNode {
            id,
            job_type: JOB_TYPE_CAM_PROCESS.to_string(),
            parent_job_id,
        }
    }

    fn sim_job(id: u64, parent_job_id: Option<u64>) -> JobNode {
        JobNode {
            id,
            job_type: JOB_TYPE_CUTTING_SIMULATION.to_string(),
            parent_job_id,
        }
    }

    #[test]
    fn rejects_simulation_without_parent() {
        let policy = CamWorkflowPolicy;
        let snapshot = WorkflowSnapshot::default();
        let request = JobSubmissionRequest {
            job_type: JOB_TYPE_CUTTING_SIMULATION.to_string(),
            input_ref: "input://sim/no-parent".to_string(),
            parent_job_id: None,
            group_id: None,
        };

        assert_eq!(
            policy.validate_submit(&request, &snapshot),
            Err(DomainRuleViolation::MissingParent)
        );
    }

    #[test]
    fn rejects_second_simulation_under_same_parent() {
        let policy = CamWorkflowPolicy;
        let snapshot = WorkflowSnapshot {
            jobs: vec![cam_job(1, None), sim_job(2, Some(1))],
        };
        let request = JobSubmissionRequest {
            job_type: JOB_TYPE_CUTTING_SIMULATION.to_string(),
            input_ref: "input://sim/second".to_string(),
            parent_job_id: Some(1),
            group_id: None,
        };

        assert_eq!(
            policy.validate_submit(&request, &snapshot),
            Err(DomainRuleViolation::DuplicateSimulationUnderParent { parent_id: 1 })
        );
    }

    #[test]
    fn rejects_child_under_simulation() {
        let policy = CamWorkflowPolicy;
        let snapshot = WorkflowSnapshot {
            jobs: vec![cam_job(1, None), sim_job(2, Some(1))],
        };
        let request = JobSubmissionRequest {
            job_type: JOB_TYPE_CAM_PROCESS.to_string(),
            input_ref: "input://cam/after-sim".to_string(),
            parent_job_id: Some(2),
            group_id: None,
        };

        assert_eq!(
            policy.validate_submit(&request, &snapshot),
            Err(DomainRuleViolation::TerminalConstraintViolation { parent_id: 2 })
        );
    }

    #[test]
    fn allows_first_simulation_under_cam_parent() {
        let policy = CamWorkflowPolicy;
        let snapshot = WorkflowSnapshot {
            jobs: vec![cam_job(1, None)],
        };
        let request = JobSubmissionRequest {
            job_type: JOB_TYPE_CUTTING_SIMULATION.to_string(),
            input_ref: "input://sim/first".to_string(),
            parent_job_id: Some(1),
            group_id: None,
        };

        assert_eq!(policy.validate_submit(&request, &snapshot), Ok(()));
    }
}
