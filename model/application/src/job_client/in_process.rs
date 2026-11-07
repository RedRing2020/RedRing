//! 同じプロセスの中でジョブを実行するクライアント（試験用）

use std::sync::Arc;

use cam_sim::{
    CamJobExecutorAdapter, CamSolverInputProvider, CuttingSimulationInputProvider,
    InMemoryCamSolverInputStore, InMemoryCuttingSimulationInputStore,
};
use job_domain::{CamJobStatus, CamJobType};
use job_runtime::JobManager;

use super::{JobClient, JobSubmission, SimulationJob};
use crate::cam_simulation_workflow::{CamSimulationWorkflowError, CamSimulationWorkflowResult};
use crate::job_orchestration::{
    JobWorkflowOrchestration, JobWorkflowOrchestrator, JobWorkflowResultQuery,
    JobWorkflowSubmitRequest,
};

/// 同じプロセスの中のジョブマネージャーで実行するクライアント
#[derive(Debug)]
pub(crate) struct InProcessJobClient {
    orchestrator: JobWorkflowOrchestrator,
    solver_inputs: Arc<InMemoryCamSolverInputStore>,
    simulation_inputs: Arc<InMemoryCuttingSimulationInputStore>,
    /// 投入の ID ごとの、ツールパス生成と切削シミュレーションのジョブの番号
    submissions: Vec<(String, u64, u64)>,
}

impl Default for InProcessJobClient {
    fn default() -> Self {
        let solver_inputs = Arc::new(InMemoryCamSolverInputStore::new());
        let simulation_inputs = Arc::new(InMemoryCuttingSimulationInputStore::new());
        let executor = CamJobExecutorAdapter::with_input_provider(solver_inputs.clone())
            .with_simulation_input_provider(simulation_inputs.clone());
        Self {
            orchestrator: JobWorkflowOrchestrator::with_runtime(JobManager::new(), executor),
            solver_inputs,
            simulation_inputs,
            submissions: Vec::new(),
        }
    }
}

impl JobClient for InProcessJobClient {
    fn submit(&mut self, submission: JobSubmission) -> Result<String, CamSimulationWorkflowError> {
        let run = self.submissions.len() + 1;
        let submission_id = format!("run-{run}");
        let cam_input_ref = format!("input://cam/{submission_id}");
        let simulation_input_ref = format!("input://sim/{submission_id}");
        self.solver_inputs
            .register(&cam_input_ref, submission.solver_input)
            .map_err(|error| CamSimulationWorkflowError::InputRegistration(error.to_string()))?;
        self.simulation_inputs
            .register(&simulation_input_ref, submission.simulation_input)
            .map_err(|error| CamSimulationWorkflowError::InputRegistration(error.to_string()))?;
        let toolpath = self
            .orchestrator
            .submit_workflow(JobWorkflowSubmitRequest {
                job_type: CamJobType::CamProcess,
                input_ref: cam_input_ref,
                parent_job_id: None,
                source: submission.source.clone(),
            })?;
        let simulation = self
            .orchestrator
            .submit_workflow(JobWorkflowSubmitRequest {
                job_type: CamJobType::CuttingSimulation,
                input_ref: simulation_input_ref,
                parent_job_id: Some(toolpath.job_id),
                source: submission.source,
            })?;
        self.submissions
            .push((submission_id.clone(), toolpath.job_id, simulation.job_id));
        Ok(submission_id)
    }

    fn run_pending(&mut self) -> Result<usize, CamSimulationWorkflowError> {
        Ok(self.orchestrator.run_queued_jobs()?)
    }

    fn simulation_jobs(&self) -> Result<Vec<SimulationJob>, CamSimulationWorkflowError> {
        self.submissions
            .iter()
            .map(|(submission_id, toolpath_job_id, simulation_job_id)| {
                let status = self.orchestrator.query_result(JobWorkflowResultQuery {
                    job_id: *simulation_job_id,
                })?;
                let toolpath = self.orchestrator.query_result(JobWorkflowResultQuery {
                    job_id: *toolpath_job_id,
                })?;
                Ok(SimulationJob {
                    job_id: *simulation_job_id,
                    submission_id: submission_id.clone(),
                    source: status.job.spec.source.clone(),
                    status: status.job.status,
                    toolpath_status: toolpath.job.status,
                })
            })
            .collect()
    }

    fn pending_submissions(
        &self,
    ) -> Result<Vec<super::PendingSubmission>, CamSimulationWorkflowError> {
        // 投入したときにジョブになる
        Ok(Vec::new())
    }

    fn import(
        &self,
        simulation_job_id: u64,
    ) -> Result<CamSimulationWorkflowResult, CamSimulationWorkflowError> {
        let result = self.orchestrator.query_result(JobWorkflowResultQuery {
            job_id: simulation_job_id,
        })?;
        if result.job.status != CamJobStatus::Succeeded
            || result.job.spec.job_type != CamJobType::CuttingSimulation
        {
            return Err(CamSimulationWorkflowError::NotSucceededSimulation(
                simulation_job_id,
            ));
        }
        let missing = || CamSimulationWorkflowError::MissingArtifact(simulation_job_id);
        let missing_input = || CamSimulationWorkflowError::MissingInput(simulation_job_id);
        let simulation_input = self
            .simulation_inputs
            .resolve(&result.job.spec.input_ref)
            .ok_or_else(missing_input)?;
        let toolpath_job_id = result.job.parent_job_id.ok_or_else(missing_input)?;
        let toolpath_job = self.orchestrator.query_result(JobWorkflowResultQuery {
            job_id: toolpath_job_id,
        })?;
        let solver_input = self
            .solver_inputs
            .resolve(&toolpath_job.job.spec.input_ref)
            .ok_or_else(missing_input)?;
        Ok(CamSimulationWorkflowResult {
            // 切削シミュレーションの入力に使ったツールパス（親のジョブの成果物）
            toolpath_artifact: result.input_artifact.ok_or_else(missing)?,
            simulation_artifact: result.active_artifact.ok_or_else(missing)?,
            solver_input,
            simulation_input,
        })
    }
}
