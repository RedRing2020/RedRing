use std::error::Error;
use std::fmt::{Display, Formatter};

use job_domain::{
    CamWorkflowPolicy, DomainRuleViolation, JOB_TYPE_CAM_PROCESS, JOB_TYPE_CUTTING_SIMULATION,
    JOB_TYPE_NC_POST_FROM_CAM, JobDomainService, JobNode, JobSubmissionRequest, WorkflowSnapshot,
};
use job_runtime::{JobError, JobId, JobManager, JobRelation, JobSpec, JobType};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CamWorkflowError {
    /// job_runtime 層エラー
    JobRuntime(JobError),
    /// 親ジョブ未指定
    MissingParent,
    /// 投入ジョブ種別が不正
    InvalidJobType { expected: JobType, actual: JobType },
    /// 親ジョブ種別が不正
    InvalidParentType { expected: JobType, actual: JobType },
    /// SIM は CAM 工程ごとに 1 件のみ
    SimulationAlreadyExists { parent_cam_job_id: JobId },
    /// SIM は末尾工程であるため子を持てない
    SimulationCannotHaveChildren { simulation_job_id: JobId },
}

impl Display for CamWorkflowError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::JobRuntime(err) => write!(f, "job runtime error: {}", err),
            Self::MissingParent => write!(f, "parent job is required for this submission"),
            Self::InvalidJobType { expected, actual } => write!(
                f,
                "invalid job type: expected={:?}, actual={:?}",
                expected, actual
            ),
            Self::InvalidParentType { expected, actual } => write!(
                f,
                "invalid parent type: expected={:?}, actual={:?}",
                expected, actual
            ),
            Self::SimulationAlreadyExists { parent_cam_job_id } => write!(
                f,
                "cutting simulation already exists for cam process: {}",
                parent_cam_job_id.0
            ),
            Self::SimulationCannotHaveChildren { simulation_job_id } => write!(
                f,
                "cutting simulation cannot have child jobs: {}",
                simulation_job_id.0
            ),
        }
    }
}

impl Error for CamWorkflowError {}

impl From<JobError> for CamWorkflowError {
    fn from(value: JobError) -> Self {
        Self::JobRuntime(value)
    }
}

/// CAM 工程における投入制約を適用するファサード
pub struct CamWorkflowSubmitter<'a> {
    manager: &'a mut JobManager,
    domain_service: JobDomainService<CamWorkflowPolicy>,
}

impl<'a> CamWorkflowSubmitter<'a> {
    pub fn new(manager: &'a mut JobManager) -> Self {
        Self {
            manager,
            domain_service: JobDomainService::new(CamWorkflowPolicy),
        }
    }

    pub fn submit_cam_process(&mut self, spec: JobSpec) -> Result<JobId, CamWorkflowError> {
        if spec.job_type != JobType::CamProcess {
            return Err(CamWorkflowError::InvalidJobType {
                expected: JobType::CamProcess,
                actual: spec.job_type,
            });
        }

        self.submit_with_domain_validation(spec, JobRelation::default())
    }

    /// CAM 工程の末尾に切削シミュレーションを 1 件だけ登録する
    pub fn submit_cutting_simulation(
        &mut self,
        parent_cam_job_id: JobId,
        spec: JobSpec,
    ) -> Result<JobId, CamWorkflowError> {
        if spec.job_type != JobType::CuttingSimulation {
            return Err(CamWorkflowError::InvalidJobType {
                expected: JobType::CuttingSimulation,
                actual: spec.job_type,
            });
        }

        self.submit_with_domain_validation(
            spec,
            JobRelation {
                parent_job_id: Some(parent_cam_job_id),
                group_id: None,
            },
        )
    }

    pub fn submit_nc_post_from_cam(
        &mut self,
        parent_cam_job_id: JobId,
        spec: JobSpec,
    ) -> Result<JobId, CamWorkflowError> {
        if spec.job_type != JobType::NcPostFromCam {
            return Err(CamWorkflowError::InvalidJobType {
                expected: JobType::NcPostFromCam,
                actual: spec.job_type,
            });
        }

        self.submit_with_domain_validation(
            spec,
            JobRelation {
                parent_job_id: Some(parent_cam_job_id),
                group_id: None,
            },
        )
    }

    /// 末尾制約: SIM を親にする投入は禁止
    pub fn submit_child_under(
        &mut self,
        parent_job_id: JobId,
        spec: JobSpec,
    ) -> Result<JobId, CamWorkflowError> {
        self.submit_with_domain_validation(
            spec,
            JobRelation {
                parent_job_id: Some(parent_job_id),
                group_id: None,
            },
        )
    }

    fn submit_with_domain_validation(
        &mut self,
        spec: JobSpec,
        relation: JobRelation,
    ) -> Result<JobId, CamWorkflowError> {
        let request = build_domain_request(&spec, &relation);
        let snapshot = build_snapshot(self.manager);
        self.domain_service
            .plan_submit(request, &snapshot)
            .map_err(map_domain_violation)?;

        Ok(self.manager.submit_with_relation(spec, relation)?)
    }
}

fn build_domain_request(spec: &JobSpec, relation: &JobRelation) -> JobSubmissionRequest {
    JobSubmissionRequest {
        job_type: job_type_to_domain_name(&spec.job_type).to_string(),
        input_ref: spec.input_ref.clone(),
        parent_job_id: relation.parent_job_id.map(|id| id.0),
        group_id: relation.group_id.clone(),
    }
}

fn build_snapshot(manager: &JobManager) -> WorkflowSnapshot {
    WorkflowSnapshot {
        jobs: manager
            .list()
            .into_iter()
            .map(|record| JobNode {
                id: record.id.0,
                job_type: job_type_to_domain_name(&record.spec.job_type).to_string(),
                parent_job_id: record.parent_job_id.map(|id| id.0),
            })
            .collect(),
    }
}

fn job_type_to_domain_name(job_type: &JobType) -> &'static str {
    match job_type {
        JobType::CamProcess => JOB_TYPE_CAM_PROCESS,
        JobType::CuttingSimulation => JOB_TYPE_CUTTING_SIMULATION,
        JobType::NcPostFromCam => JOB_TYPE_NC_POST_FROM_CAM,
    }
}

fn domain_name_to_job_type(name: &str) -> Option<JobType> {
    match name {
        JOB_TYPE_CAM_PROCESS => Some(JobType::CamProcess),
        JOB_TYPE_CUTTING_SIMULATION => Some(JobType::CuttingSimulation),
        JOB_TYPE_NC_POST_FROM_CAM => Some(JobType::NcPostFromCam),
        _ => None,
    }
}

fn map_domain_violation(violation: DomainRuleViolation) -> CamWorkflowError {
    match violation {
        DomainRuleViolation::MissingParent => CamWorkflowError::MissingParent,
        DomainRuleViolation::ParentNotFound(parent_id) => {
            CamWorkflowError::JobRuntime(JobError::ParentJobNotFound(JobId(parent_id)))
        }
        DomainRuleViolation::InvalidParentType { expected, actual } => {
            let expected = domain_name_to_job_type(expected).expect("known expected job type");
            let actual = domain_name_to_job_type(&actual).expect("known actual job type");
            CamWorkflowError::InvalidParentType { expected, actual }
        }
        DomainRuleViolation::TerminalConstraintViolation { parent_id } => {
            CamWorkflowError::SimulationCannotHaveChildren {
                simulation_job_id: JobId(parent_id),
            }
        }
        DomainRuleViolation::DuplicateSimulationUnderParent { parent_id } => {
            CamWorkflowError::SimulationAlreadyExists {
                parent_cam_job_id: JobId(parent_id),
            }
        }
    }
}
