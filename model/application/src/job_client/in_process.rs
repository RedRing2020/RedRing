//! 同じプロセスの中でジョブを実行するクライアント（試験用）

use std::sync::Arc;

use cam_sim::{
    CamJobExecutorAdapter, CamSolverInputProvider, CuttingSimulationInputProvider,
    InMemoryCamSolverInputStore, InMemoryCuttingSimulationInputStore, TOOLPATH_MEMORY_LIMIT_BYTES,
};
use job_domain::{CamJobSource, CamJobStatus, CamJobType};
use job_ipc::ToolpathLimits;
use job_runtime::JobManager;

use super::orchestrator::{
    JobWorkflowOrchestration, JobWorkflowOrchestrator, JobWorkflowResultQuery,
    JobWorkflowSubmitRequest,
};
use super::{
    JobClient, SegmentSubmission, SegmentToolpath, SimulationJob, StartStock, ToolpathJob,
};
use crate::cam_simulation_workflow::{
    CamSimulationWorkflowError, CamSimulationWorkflowResult, CamToolpathResult,
};

/// 同じプロセスの中で実行するときの、ツールパス生成のメモリの見積もりを計算の前に確かめる目安（バイト。2 GB）
const TOOLPATH_CONFIRM_BYTES: u64 = 2_000_000_000;

/// 同じプロセスの中のジョブマネージャーで実行するクライアント
#[derive(Debug)]
pub(crate) struct InProcessJobClient {
    orchestrator: JobWorkflowOrchestrator,
    solver_inputs: Arc<InMemoryCamSolverInputStore>,
    simulation_inputs: Arc<InMemoryCuttingSimulationInputStore>,
    /// 投入した区間（投入の順）
    submissions: Vec<Segment>,
    /// 投入を頼まれた回数（失敗した投入も数え、投入ごとに別の入力の参照にする）
    attempts: usize,
}

/// 投入した区間のジョブ
#[derive(Debug)]
struct Segment {
    submission_id: String,
    toolpath_job_id: u64,
    simulation_job_id: u64,
    /// 始まりの素材にする、前の区間の切削シミュレーションのジョブ
    start_stock: Option<u64>,
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
            attempts: 0,
        }
    }
}

impl JobClient for InProcessJobClient {
    fn submit(
        &mut self,
        segments: Vec<SegmentSubmission>,
    ) -> Result<String, CamSimulationWorkflowError> {
        self.attempts += 1;
        let submission_id = format!("run-{}", self.attempts);
        // 同じプロセスの中の実行は、ツールパス生成のジョブを持たない切削シミュレーションを扱わない（ジョブを置く前に
        // 確かめ、一部の区間だけを置かない）
        let unsupported = segments.iter().enumerate().any(|(index, segment)| {
            matches!(segment.toolpath, SegmentToolpath::Artifact(_))
                || matches!(segment.start_stock, Some(StartStock::Submitted(previous)) if previous >= index)
        });
        if unsupported {
            return Err(CamSimulationWorkflowError::InputRegistration(
                "simulating an imported toolpath or a segment starting from a later one is not supported"
                    .to_string(),
            ));
        }
        let registration = |error: job_runtime::RefValidationError| {
            CamSimulationWorkflowError::InputRegistration(error.to_string())
        };
        // 区間ごとの、切削シミュレーションのジョブの番号（後の区間の始まりの素材で指す）
        let mut simulation_job_ids: Vec<u64> = Vec::new();
        for (index, segment) in segments.into_iter().enumerate() {
            let toolpath_job_id = match segment.toolpath {
                SegmentToolpath::Solve(solver_input) => {
                    let cam_input_ref = format!("input://cam/{submission_id}-{index}");
                    self.solver_inputs
                        .register(&cam_input_ref, *solver_input)
                        .map_err(registration)?;
                    // ツールパス生成のジョブは、区間の前提の鍵を持たない
                    let source = segment.source.clone().map(|source| CamJobSource {
                        segment_key: None,
                        ..source
                    });
                    self.orchestrator
                        .submit_workflow(JobWorkflowSubmitRequest {
                            job_type: CamJobType::CamProcess,
                            input_ref: cam_input_ref,
                            parent_job_id: None,
                            source,
                        })?
                        .job_id
                }
                // 同じプロセスの中のジョブは、ツールパス生成 1 つに切削シミュレーションを 1 つしか置けないため、置いた
                // ものは、同じ入力でツールパス生成を置き直す（同じ CL になる）
                SegmentToolpath::Job(job_id)
                    if self
                        .submissions
                        .iter()
                        .any(|segment| segment.toolpath_job_id == job_id) =>
                {
                    let toolpath = self
                        .orchestrator
                        .query_result(JobWorkflowResultQuery { job_id })?
                        .job
                        .spec;
                    self.orchestrator
                        .submit_workflow(JobWorkflowSubmitRequest {
                            job_type: CamJobType::CamProcess,
                            input_ref: toolpath.input_ref,
                            parent_job_id: None,
                            source: toolpath.source,
                        })?
                        .job_id
                }
                SegmentToolpath::Job(job_id) => job_id,
                // 同じプロセスの中の実行は、ツールパス生成のジョブを持たない切削シミュレーションを扱わない
                SegmentToolpath::Artifact(_) => {
                    return Err(CamSimulationWorkflowError::InputRegistration(
                        "simulating an imported toolpath needs the job manager".to_string(),
                    ));
                }
            };
            let simulation_input_ref = format!("input://sim/{submission_id}-{index}");
            self.simulation_inputs
                .register(&simulation_input_ref, segment.simulation_input)
                .map_err(registration)?;
            let start_stock = match segment.start_stock {
                None => None,
                Some(StartStock::Submitted(previous)) => {
                    Some(*simulation_job_ids.get(previous).ok_or_else(|| {
                        CamSimulationWorkflowError::InputRegistration(format!(
                            "segment {index} starts from segment {previous}, which is not earlier"
                        ))
                    })?)
                }
                Some(StartStock::Job(job_id)) => Some(job_id),
            };
            if let Some(job_id) = start_stock {
                let previous_ref = self
                    .orchestrator
                    .query_result(JobWorkflowResultQuery { job_id })?
                    .job
                    .spec
                    .input_ref;
                self.simulation_inputs
                    .register_start_stock(&simulation_input_ref, &previous_ref)
                    .map_err(registration)?;
            }
            let simulation = self
                .orchestrator
                .submit_workflow(JobWorkflowSubmitRequest {
                    job_type: CamJobType::CuttingSimulation,
                    input_ref: simulation_input_ref.clone(),
                    parent_job_id: Some(toolpath_job_id),
                    source: segment.source,
                })?;
            simulation_job_ids.push(simulation.job_id);
            self.submissions.push(Segment {
                submission_id: submission_id.clone(),
                toolpath_job_id,
                simulation_job_id: simulation.job_id,
                start_stock,
            });
        }
        Ok(submission_id)
    }

    fn toolpath_jobs(&self) -> Result<Vec<ToolpathJob>, CamSimulationWorkflowError> {
        // ツールパス生成のジョブは、それを作った（最初の）投入の ID を持つ（後の投入は、その下に切削シミュレーション
        // だけを置く）
        let mut jobs: Vec<(u64, &str)> = Vec::new();
        for segment in &self.submissions {
            if !jobs
                .iter()
                .any(|(job_id, _)| *job_id == segment.toolpath_job_id)
            {
                jobs.push((segment.toolpath_job_id, &segment.submission_id));
            }
        }
        jobs.sort_unstable();
        jobs.into_iter()
            .map(|(job_id, submission_id)| {
                let result = self
                    .orchestrator
                    .query_result(JobWorkflowResultQuery { job_id })?;
                // 投入はいつも切削シミュレーションを置く
                Ok(ToolpathJob {
                    job_id,
                    submission_id: submission_id.to_string(),
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
            .map(|segment| {
                let status = self.orchestrator.query_result(JobWorkflowResultQuery {
                    job_id: segment.simulation_job_id,
                })?;
                let toolpath = self.orchestrator.query_result(JobWorkflowResultQuery {
                    job_id: segment.toolpath_job_id,
                })?;
                Ok(SimulationJob {
                    job_id: segment.simulation_job_id,
                    submission_id: segment.submission_id.clone(),
                    source: status.job.spec.source.clone(),
                    status: status.job.status,
                    toolpath_status: toolpath.job.status,
                    start_stock: segment.start_stock,
                    // 同じプロセスの中の計算は、失敗の理由の種類を持たない
                    failure: None,
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

    fn cancel(&mut self, job_ids: &[u64]) -> Result<(), CamSimulationWorkflowError> {
        for &job_id in job_ids {
            self.orchestrator.cancel_job(job_id)?;
        }
        Ok(())
    }

    fn resume(&mut self, _job_ids: &[u64]) -> Result<(), CamSimulationWorkflowError> {
        // 同じプロセスの中で実行するため、計算の途中で止まった中断のジョブを持たない
        Ok(())
    }

    fn stop(&mut self, job_ids: &[u64]) -> Result<(), CamSimulationWorkflowError> {
        // 同じプロセスの中で実行するため、中断を持たない。止める頼みは取り消しにする
        self.cancel(job_ids)
    }

    fn toolpath_limits(&self) -> Result<ToolpathLimits, CamSimulationWorkflowError> {
        Ok(ToolpathLimits {
            confirm_bytes: TOOLPATH_CONFIRM_BYTES,
            limit_bytes: TOOLPATH_MEMORY_LIMIT_BYTES,
        })
    }

    fn manager_running(&self) -> Result<bool, CamSimulationWorkflowError> {
        Ok(true)
    }

    fn hold_connection(&mut self) -> Result<(), CamSimulationWorkflowError> {
        // 同じプロセスの中で実行するため、つなぐ口を持たない
        Ok(())
    }

    fn release_connection(&mut self) {}

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
                .find(|segment| segment.toolpath_job_id == toolpath_job_id)
            {
                Some(segment) => {
                    self.orchestrator
                        .query_result(JobWorkflowResultQuery {
                            job_id: segment.simulation_job_id,
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

    fn read_status_now(&mut self) -> Result<super::JobStatusSnapshot, CamSimulationWorkflowError> {
        Ok(super::JobStatusSnapshot {
            pending: Vec::new(),
            simulations: self.simulation_jobs()?,
        })
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
