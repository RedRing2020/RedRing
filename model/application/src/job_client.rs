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
#[cfg(any(test, feature = "test-fixtures"))]
mod orchestrator;

use std::fmt;

use cam_algorithms::CamSolverInput;
use cam_sim::CuttingSimulationInput;
use job_domain::{CamJobSource, CamJobStatus};
use job_ipc::{JobFailure, ToolpathLimits};

use crate::cam_simulation_workflow::{
    CamSimulationWorkflowError, CamSimulationWorkflowResult, CamToolpathResult,
};

#[cfg(any(test, feature = "test-fixtures"))]
pub(crate) use in_process::InProcessJobClient;
pub(crate) use pipe::PipeJobClient;

/// 区間の切削シミュレーションが入力にするツールパス
#[derive(Debug, Clone)]
pub(crate) enum SegmentToolpath {
    /// この入力でツールパス生成のジョブを置き、その成果物を読む
    Solve(Box<CamSolverInput>),
    /// すでにあるツールパス生成のジョブ（その成果物を読む。ジョブの番号）
    Job(u64),
    /// CL（ツールパスの成果物のバイト列。ツールパス生成のジョブを持たない）
    Artifact(Vec<u8>),
}

/// 区間の切削シミュレーションの始まりの素材
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum StartStock {
    /// すでにある、前の区間の切削シミュレーションのジョブ（ジョブの番号）の終わりの素材
    Job(u64),
    /// 同じ投入の中の、前に並ぶ区間（区間の列の番号）の終わりの素材
    Submitted(usize),
}

/// 投入する、加工工程 1 つの区間（ツールパスと、それを入力にする切削シミュレーション）
#[derive(Debug, Clone)]
pub(crate) struct SegmentSubmission {
    pub toolpath: SegmentToolpath,
    pub simulation_input: CuttingSimulationInput,
    /// 始まりの素材（`None` は素材の範囲から削る）
    pub start_stock: Option<StartStock>,
    /// 投入元（工程の ID と改訂の番号・改訂の鍵。切削シミュレーションのジョブは、区間の前提の鍵も持つ）
    pub source: Option<CamJobSource>,
}

/// ツールパス生成のジョブ
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ToolpathJob {
    pub job_id: u64,
    /// 投入の ID
    pub submission_id: String,
    pub source: Option<CamJobSource>,
    pub status: CamJobStatus,
    /// その成果物を入力にする切削シミュレーションのジョブがある（ツールパス生成のジョブ 1 つに、切削シミュレーションは
    /// 区間の前提ごとに 1 つ置く）
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
    /// 始まりの素材にする、前の区間の切削シミュレーションのジョブ（素材の範囲から削るものは `None`）
    pub start_stock: Option<u64>,
    /// 失敗の理由（入力にするツールパス生成が失敗したときは、その理由。切削シミュレーションの理由は、依存するジョブが
    /// 失敗したことしか表さないため。理由を受け取れなかったときは `None`）
    pub failure: Option<JobFailure>,
}

/// まだジョブになっていない投入（ジョブマネージャーの取り込みを待っている・取り込めなかった）
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PendingSubmission {
    /// 投入するジョブの投入元（ジョブの順。投入元を持たないジョブは除く）
    pub sources: Vec<CamJobSource>,
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
    /// 加工の順の区間 `segments` のジョブ（区間ごとに、要ればツールパス生成と、切削シミュレーション）を 1 つの投入で
    /// 置き、投入の ID を返す（計算を待たない）
    fn submit(
        &mut self,
        segments: Vec<SegmentSubmission>,
    ) -> Result<String, CamSimulationWorkflowError>;

    /// ツールパス生成のジョブ（番号の順）
    fn toolpath_jobs(&self) -> Result<Vec<ToolpathJob>, CamSimulationWorkflowError>;

    /// 同じプロセスの中で実行するときに、待ちのジョブを実行する。実行したジョブの数を返す（外のジョブマネージャーは、
    /// 待ちのジョブを自分で実行するため、何もせず 0）
    fn run_pending(&mut self) -> Result<usize, CamSimulationWorkflowError>;

    /// ジョブマネージャーのウィンドウを開く（動いていなければ、ウィンドウを出して起動する）
    fn open_manager_window(&mut self) -> Result<(), CamSimulationWorkflowError>;

    /// 切削シミュレーションのジョブ（番号の順）
    fn simulation_jobs(&self) -> Result<Vec<SimulationJob>, CamSimulationWorkflowError>;

    /// 計算の状態（画面を止めない実装は、読み終えるのを待たずに最後に読んだ状態を返し、まだ読めていない・前に読んだ
    /// 状態が古ければ `None`）
    fn status_snapshot(&self) -> Result<Option<JobStatusSnapshot>, CamSimulationWorkflowError>;

    /// 計算の状態を、今読む（ジョブマネージャーが動いていなければ起動して、口が開くのを待つ）。読んだ状態は、
    /// [`Self::status_snapshot`] でも返す
    fn read_status_now(&mut self) -> Result<JobStatusSnapshot, CamSimulationWorkflowError>;

    /// 正常に終わった切削シミュレーションのジョブ `simulation_job_id` の、ツールパスと切削シミュレーションの成果物と、
    /// 投入した入力
    fn import(
        &self,
        simulation_job_id: u64,
    ) -> Result<CamSimulationWorkflowResult, CamSimulationWorkflowError>;

    /// 正常に終わったツールパス生成のジョブ `toolpath_job_id` の成果物（CL）と、その投入元
    fn toolpath_result(
        &self,
        toolpath_job_id: u64,
    ) -> Result<CamToolpathResult, CamSimulationWorkflowError>;

    /// 切削シミュレーションのジョブ `simulation_job_id` の結果（その CL）を、空間モデルに取り込んだと伝える（ジョブ
    /// マネージャーが、取り込み済みの CL の元をまとめて消せるように。画面を止めない実装は、伝え終えるのを待たない）
    fn record_imported(&self, simulation_job_id: u64) -> Result<(), CamSimulationWorkflowError>;

    /// 計算待ち・計算中・中断のジョブ `job_ids` を取り消すよう頼む（計算中はランナーを止める。取り消し終えるのを待たない）
    fn cancel(&mut self, job_ids: &[u64]) -> Result<(), CamSimulationWorkflowError>;

    /// 中断のジョブ `job_ids` を計算待ちに戻すよう頼む（戻し終えるのを待たない）
    fn resume(&mut self, job_ids: &[u64]) -> Result<(), CamSimulationWorkflowError>;

    /// 計算中のジョブ `job_ids` を止めて中断にするよう頼む（途中の結果を残す。止め終えるのを待たない）
    fn stop(&mut self, job_ids: &[u64]) -> Result<(), CamSimulationWorkflowError>;

    /// ツールパスの大きさ（生成に使うメモリの見積もり）の、計算の前に確かめる目安と上限
    fn toolpath_limits(&self) -> Result<ToolpathLimits, CamSimulationWorkflowError>;

    /// ジョブマネージャーが動いているか（起動しない。同じプロセスの中で実行する実装は、いつも動いている。動いていない・
    /// 見つからないときは偽、ほかの理由でつなげないときは誤り）
    fn manager_running(&self) -> Result<bool, CamSimulationWorkflowError>;

    /// ジョブマネージャーにつなぎ、[`Self::release_connection`] までの問い合わせを、その接続で行う（動いていなければ
    /// 起動して、口が開くのを待つ。1 回の操作で、つなぐのと起動を待つのを 1 回にする）。間の問い合わせに失敗したら、
    /// つなぎ直さずに、後の問い合わせも誤りにする
    fn hold_connection(&mut self) -> Result<(), CamSimulationWorkflowError>;

    /// [`Self::hold_connection`] でつないだ接続を閉じる（後の問い合わせは、問い合わせごとにつなぐ）
    fn release_connection(&mut self);
}
