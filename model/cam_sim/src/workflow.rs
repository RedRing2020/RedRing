use std::error::Error;
use std::fmt::{Display, Formatter};

use job_runtime::{JobError, JobId, JobManager, JobRelation, JobSpec, JobType};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CamWorkflowError {
    /// job_runtime 層エラー
    JobRuntime(JobError),
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
            Self::InvalidParentType { expected, actual } => write!(
                f,
                "invalid parent type for cutting simulation: expected={:?}, actual={:?}",
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
}

impl<'a> CamWorkflowSubmitter<'a> {
    pub fn new(manager: &'a mut JobManager) -> Self {
        Self { manager }
    }

    pub fn submit_cam_process(&mut self, spec: JobSpec) -> Result<JobId, CamWorkflowError> {
        if spec.job_type != JobType::CamProcess {
            return Err(CamWorkflowError::InvalidParentType {
                expected: JobType::CamProcess,
                actual: spec.job_type,
            });
        }

        Ok(self.manager.submit(spec))
    }

    /// CAM 工程の末尾に切削シミュレーションを 1 件だけ登録する
    pub fn submit_cutting_simulation(
        &mut self,
        parent_cam_job_id: JobId,
        spec: JobSpec,
    ) -> Result<JobId, CamWorkflowError> {
        if spec.job_type != JobType::CuttingSimulation {
            return Err(CamWorkflowError::InvalidParentType {
                expected: JobType::CuttingSimulation,
                actual: spec.job_type,
            });
        }

        let parent = self.manager.get(parent_cam_job_id)?;
        if parent.spec.job_type != JobType::CamProcess {
            return Err(CamWorkflowError::InvalidParentType {
                expected: JobType::CamProcess,
                actual: parent.spec.job_type.clone(),
            });
        }

        // 末尾制約: すでに SIM 子がある CAM 工程には追加不可
        let has_sim_child = self
            .manager
            .list_by_parent(parent_cam_job_id)
            .iter()
            .any(|child| child.spec.job_type == JobType::CuttingSimulation);

        if has_sim_child {
            return Err(CamWorkflowError::SimulationAlreadyExists { parent_cam_job_id });
        }

        Ok(self.manager.submit_with_relation(
            spec,
            JobRelation {
                parent_job_id: Some(parent_cam_job_id),
                group_id: None,
            },
        )?)
    }

    /// 末尾制約: SIM を親にする投入は禁止
    pub fn submit_child_under(
        &mut self,
        parent_job_id: JobId,
        spec: JobSpec,
    ) -> Result<JobId, CamWorkflowError> {
        let parent = self.manager.get(parent_job_id)?;
        if parent.spec.job_type == JobType::CuttingSimulation {
            return Err(CamWorkflowError::SimulationCannotHaveChildren {
                simulation_job_id: parent_job_id,
            });
        }

        Ok(self.manager.submit_with_relation(
            spec,
            JobRelation {
                parent_job_id: Some(parent_job_id),
                group_id: None,
            },
        )?)
    }
}