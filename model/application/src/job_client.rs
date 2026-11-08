//! ツールパス生成と切削シミュレーションのジョブの投入・状態の読み取り・成果物の取り込みの境界
//! （設計: JOB_MANAGER_STORAGE_DESIGN.md 4・8 章）
//!
//! ジョブの計算は、CAD/CAM のアプリの外のジョブマネージャーが行う。境界の後ろの実装（ジョブマネージャーに問い合わせる
//! 口）を替えても、投入・取り込みのユースケースは変わらない。

#[cfg(test)]
mod tests;

mod pipe;

#[cfg(any(test, feature = "test-fixtures"))]
mod in_process;

use std::fmt;

use cam_algorithms::CamSolverInput;
use cam_sim::CuttingSimulationInput;
use job_domain::{CamJobSource, CamJobStatus};

use crate::cam_simulation_workflow::{CamSimulationWorkflowError, CamSimulationWorkflowResult};

#[cfg(any(test, feature = "test-fixtures"))]
pub(crate) use in_process::InProcessJobClient;
pub(crate) use pipe::PipeJobClient;

/// 投入するツールパス生成と、その成果物を入力にする切削シミュレーション
#[derive(Debug, Clone)]
pub(crate) struct JobSubmission {
    pub solver_input: CamSolverInput,
    pub simulation_input: CuttingSimulationInput,
    /// 投入元（工程の ID と改訂の番号）
    pub source: Option<CamJobSource>,
}

/// 切削シミュレーションだけを投入するときの、入力にするツールパス
#[derive(Debug, Clone)]
pub(crate) enum SimulationToolpath {
    /// すでにあるツールパス生成のジョブ（その成果物を読む。ジョブの番号）
    Job(u64),
    /// CL（ツールパスの成果物のバイト列。ツールパス生成のジョブを持たない）
    Artifact(Vec<u8>),
}

/// 投入する、切削シミュレーションだけのジョブ
#[derive(Debug, Clone)]
pub(crate) struct SimulationSubmission {
    pub simulation_input: CuttingSimulationInput,
    pub toolpath: SimulationToolpath,
    /// 投入元（工程の ID と改訂の番号）
    pub source: Option<CamJobSource>,
}

/// ツールパス生成のジョブ
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ToolpathJob {
    pub job_id: u64,
    pub source: Option<CamJobSource>,
    pub status: CamJobStatus,
    /// その成果物を入力にする切削シミュレーションのジョブがある（ツールパス生成のジョブ 1 つに、切削シミュレーションは
    /// 1 つだけ置く）
    pub simulated: bool,
}

/// 切削シミュレーションのジョブ
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SimulationJob {
    pub job_id: u64,
    /// 投入の ID
    pub submission_id: String,
    pub source: Option<CamJobSource>,
    pub status: CamJobStatus,
    /// 入力にするツールパス生成のジョブの状態（CL を入力にするジョブは、正常に終わったものとする）
    pub toolpath_status: CamJobStatus,
}

/// まだジョブになっていない投入（ジョブマネージャーの取り込みを待っている・取り込めなかった）
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PendingSubmission {
    pub source: Option<CamJobSource>,
    /// 取り込めなかった
    pub rejected: bool,
}

/// 計算の状態（取り込みを待つ投入と、切削シミュレーションのジョブ）
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct JobStatusSnapshot {
    pub pending: Vec<PendingSubmission>,
    pub simulations: Vec<SimulationJob>,
}

/// ジョブの投入・状態の読み取り・成果物の取り込み
pub(crate) trait JobClient: fmt::Debug {
    /// ツールパス生成と切削シミュレーションのジョブを投入し、投入の ID を返す（計算を待たない）
    fn submit(&mut self, submission: JobSubmission) -> Result<String, CamSimulationWorkflowError>;

    /// 切削シミュレーションのジョブだけを投入し、投入の ID を返す（計算を待たない）
    fn submit_simulation(
        &mut self,
        submission: SimulationSubmission,
    ) -> Result<String, CamSimulationWorkflowError>;

    /// ツールパス生成のジョブ（番号の順）
    fn toolpath_jobs(&self) -> Result<Vec<ToolpathJob>, CamSimulationWorkflowError>;

    /// 待ちのジョブを実行させる。実行したジョブの数を返す（外のジョブマネージャーでは、止まっていれば起動し直すだけで 0）
    fn run_pending(&mut self) -> Result<usize, CamSimulationWorkflowError>;

    /// ジョブマネージャーのウィンドウを開く（動いていなければ、ウィンドウを出して起動する）
    fn open_manager_window(&mut self) -> Result<(), CamSimulationWorkflowError>;

    /// 切削シミュレーションのジョブ（番号の順）
    fn simulation_jobs(&self) -> Result<Vec<SimulationJob>, CamSimulationWorkflowError>;

    /// 計算の状態（画面を止めない実装は、読み終えるのを待たずに最後に読んだ状態を返し、まだ読めていない・前に読んだ
    /// 状態が古ければ `None`）
    fn status_snapshot(&self) -> Result<Option<JobStatusSnapshot>, CamSimulationWorkflowError>;

    /// 正常に終わった切削シミュレーションのジョブ `simulation_job_id` の、ツールパスと切削シミュレーションの成果物と、
    /// 投入した入力
    fn import(
        &self,
        simulation_job_id: u64,
    ) -> Result<CamSimulationWorkflowResult, CamSimulationWorkflowError>;

    /// 切削シミュレーションのジョブ `simulation_job_id` の結果（その CL）を、空間モデルに取り込んだと伝える（ジョブ
    /// マネージャーが、取り込み済みの CL の元をまとめて消せるように。画面を止めない実装は、伝え終えるのを待たない）
    fn record_imported(&self, simulation_job_id: u64) -> Result<(), CamSimulationWorkflowError>;
}
