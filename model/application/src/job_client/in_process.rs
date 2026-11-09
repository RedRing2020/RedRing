//! 同じプロセスの中でジョブを実行するクライアント（試験用）

use std::sync::Arc;

use cam_sim::{
    CamJobExecutorAdapter, CamSolverInputProvider, CuttingSimulationInputProvider,
    InMemoryCamSolverInputStore, InMemoryCuttingSimulationInputStore,
};
use job_domain::{CamJobStatus, CamJobType};
use job_runtime::JobManager;

use super::{
    JobClient, JobSubmission, SimulationJob, SimulationSubmission, SimulationToolpath, ToolpathJob,
};
use crate::cam_simulation_workflow::{
    CamSimulationWorkflowError, CamSimulationWorkflowResult, CamToolpathResult,
};
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

    fn submit_simulation(
        &mut self,
        submission: SimulationSubmission,
    ) -> Result<String, CamSimulationWorkflowError> {
        // 同じプロセスの中の実行は、ツールパス生成のジョブを持たない切削シミュレーションを扱わない
        let SimulationToolpath::Job(toolpath_job_id) = submission.toolpath else {
            return Err(CamSimulationWorkflowError::InputRegistration(
                "simulating an imported toolpath needs the job manager".to_string(),
            ));
        };
        let run = self.submissions.len() + 1;
        let submission_id = format!("run-{run}");
        let simulation_input_ref = format!("input://sim/{submission_id}");
        self.simulation_inputs
            .register(&simulation_input_ref, submission.simulation_input)
            .map_err(|error| CamSimulationWorkflowError::InputRegistration(error.to_string()))?;
        let simulation = self
            .orchestrator
            .submit_workflow(JobWorkflowSubmitRequest {
                job_type: CamJobType::CuttingSimulation,
                input_ref: simulation_input_ref,
                parent_job_id: Some(toolpath_job_id),
                source: submission.source,
            })?;
        self.submissions
            .push((submission_id.clone(), toolpath_job_id, simulation.job_id));
        Ok(submission_id)
    }

    fn toolpath_jobs(&self) -> Result<Vec<ToolpathJob>, CamSimulationWorkflowError> {
        let mut job_ids: Vec<u64> = self
            .submissions
            .iter()
            .map(|(_, toolpath_job_id, _)| *toolpath_job_id)
            .collect();
        job_ids.sort_unstable();
        job_ids.dedup();
        job_ids
            .into_iter()
            .map(|job_id| {
                let result = self
                    .orchestrator
                    .query_result(JobWorkflowResultQuery { job_id })?;
                // 投入はいつも切削シミュレーションを置く
                Ok(ToolpathJob {
                    job_id,
                    source: result.job.spec.source.clone(),
                    status: result.job.status,
                    simulated: true,
                })
            })
            .collect()
    }

    fn run_pending(&mut self) -> Result<usize, CamSimulationWorkflowError> {
        Ok(self.orchestrator.run_queued_jobs()?)
    }

    fn open_manager_window(&mut self) -> Result<(), CamSimulationWorkflowError> {
        // 同じプロセスの中で実行するため、開くウィンドウはない
        Ok(())
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

    fn record_imported(&self, simulation_job_id: u64) -> Result<(), CamSimulationWorkflowError> {
        // 同じプロセスの中のジョブは、まとめて消す操作を持たないため、番号だけを確かめる
        self.simulation_jobs()?
            .iter()
            .any(|job| job.job_id == simulation_job_id && job.status == CamJobStatus::Succeeded)
            .then_some(())
            .ok_or(CamSimulationWorkflowError::NotSucceededSimulation(
                simulation_job_id,
            ))
    }

    fn toolpath_result(
        &self,
        toolpath_job_id: u64,
    ) -> Result<CamToolpathResult, CamSimulationWorkflowError> {
        let result = self.orchestrator.query_result(JobWorkflowResultQuery {
            job_id: toolpath_job_id,
        })?;
        if result.job.status != CamJobStatus::Succeeded
            || result.job.spec.job_type != CamJobType::CamProcess
        {
            return Err(CamSimulationWorkflowError::NotSucceededToolpath(
                toolpath_job_id,
            ));
        }
        // 切削シミュレーションが正常に終わると、ツールパス生成の成果物は置き換え済みになるため、その下の切削
        // シミュレーションの入力として読む
        let artifact = match result.active_artifact {
            Some(artifact) => Some(artifact),
            None => match self
                .submissions
                .iter()
                .find(|(_, toolpath, _)| *toolpath == toolpath_job_id)
            {
                Some((_, _, simulation)) => {
                    self.orchestrator
                        .query_result(JobWorkflowResultQuery {
                            job_id: *simulation,
                        })?
                        .input_artifact
                }
                None => None,
            },
        }
        .ok_or(CamSimulationWorkflowError::MissingArtifact(toolpath_job_id))?;
        Ok(CamToolpathResult {
            artifact,
            source: result.job.spec.source.clone(),
        })
    }

    fn status_snapshot(
        &self,
    ) -> Result<Option<super::JobStatusSnapshot>, CamSimulationWorkflowError> {
        // 投入したときにジョブになるため、取り込みを待つ投入はない
        Ok(Some(super::JobStatusSnapshot {
            pending: Vec::new(),
            simulations: self.simulation_jobs()?,
        }))
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
            solver_input: Some(solver_input),
            simulation_input,
            source: result.job.spec.source.clone(),
        })
    }
}
