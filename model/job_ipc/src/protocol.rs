//! 要求と応答の型（JSON の枠で送る。設計: JOB_IPC_DESIGN.md 4・5 章）

use serde::{Deserialize, Serialize};

/// 口の版（要求・応答の形を変えたら上げる。要求を足すだけなら上げない。前のジョブマネージャーは、知らない要求に
/// 正しくない要求の誤りを返す。ジョブの状態の値を足したら上げる。前のアプリは知らない状態を読めず、ジョブの一覧を
/// まるごと読めなくなるため。版 2 で中断を足した）
pub const PROTOCOL_VERSION: u32 = 2;

/// 要求
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Request {
    /// 接続の最初に送る、口の版
    Hello { protocol_version: u32 },
    /// ジョブの列を投入する（続けて、`inputs` の順に入力の中身を送る）
    Submit {
        jobs: Vec<SubmitJob>,
        inputs: Vec<InputHeader>,
    },
    /// ジョブの一覧
    Jobs,
    /// まだジョブになっていない投入
    PendingSubmissions,
    /// ジョブのファイルの中身
    ReadJobFile { job_id: u64, file: JobFile },
    /// ジョブマネージャーのウィンドウを開く
    OpenWindow,
    /// 切削シミュレーションのジョブの結果（その CL）を取り込んだことを伝える
    RecordImported { simulation_job_id: u64 },
    /// ジョブを取り消す（計算待ち・中断は取り消し、計算中はランナーを止める。終わったジョブ・ないジョブは何も
    /// しない。取り消し終えるのを待たない）
    Cancel { job_ids: Vec<u64> },
    /// 中断のジョブを、計算待ちに戻す（中断でないジョブ・ないジョブは何もしない。戻し終えるのを待たない）
    Resume { job_ids: Vec<u64> },
    /// 計算中のジョブを止めて、中断にする（途中の結果を残す。計算中でないジョブ・ないジョブは何もしない。止め終える
    /// のを待たない）
    Stop { job_ids: Vec<u64> },
    /// ツールパスの大きさの、計算の前に確かめる目安と上限
    ToolpathLimits,
}

/// 応答
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Response {
    /// 口の版と、ジョブマネージャーの版
    Hello {
        protocol_version: u32,
        manager_version: String,
    },
    /// 投入の ID
    Submitted {
        submission_id: String,
    },
    Jobs {
        jobs: Vec<JobSummary>,
    },
    PendingSubmissions {
        submissions: Vec<PendingSubmission>,
    },
    /// ファイルの大きさ（続けて、中身を送る）
    JobFile {
        size: u64,
    },
    Ok,
    ToolpathLimits {
        limits: ToolpathLimits,
    },
    /// 誤り（種類と、英語の詳しい理由）
    Error {
        kind: ErrorKind,
        message: String,
    },
}

/// ツールパスの大きさ（生成に使うメモリの見積もり）の、計算の前に確かめる目安と上限（バイト）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToolpathLimits {
    /// これ以上なら、確かめてから計算する
    pub confirm_bytes: u64,
    /// これを超えるなら計算しない（ランナーも、これを超えると止まる）
    pub limit_bytes: u64,
}

/// 投入の入力の名前と大きさ
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InputHeader {
    pub name: String,
    pub size: u64,
}

/// ジョブの種類
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JobKind {
    /// ツールパス生成
    Cam,
    /// 切削シミュレーション
    Sim,
}

/// ジョブの状態
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JobStatus {
    Queued,
    Running,
    Succeeded,
    Failed,
    Canceled,
    /// 計算の途中で、ジョブマネージャーの終了・異常終了・OS の再起動で止まった
    Interrupted,
}

/// 投入元（投入元の ID・改訂の番号・改訂の鍵）
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JobSource {
    pub source_id: String,
    pub revision: u32,
    pub revision_key: Option<String>,
}

/// 依存先
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Dependency {
    /// 同じ投入の中の、前に並ぶジョブ（`jobs` の番号）
    Submitted(usize),
    /// すでにあるジョブの番号
    Existing(u64),
}

/// 投入するジョブ
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SubmitJob {
    pub kind: JobKind,
    /// 入力の名前（投入の入力の 1 つ）
    pub input_file: String,
    pub depends_on: Vec<Dependency>,
    pub priority: i32,
    pub source: Option<JobSource>,
    /// 切削シミュレーションが入力にする CL の名前（投入の入力の 1 つ。指定するときは、ツールパス生成のジョブに依存
    /// しない）
    pub toolpath_file: Option<String>,
}

/// ジョブの一覧の 1 つ
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JobSummary {
    pub job_id: u64,
    pub kind: JobKind,
    pub status: JobStatus,
    /// 依存するジョブ
    pub depends_on: Vec<u64>,
    /// 投入の ID
    pub submission_id: String,
    pub source: Option<JobSource>,
    /// 入力の CL を持つ（CL を入力にする切削シミュレーション）
    pub has_input_toolpath: bool,
    /// 成果物がある
    pub has_result: bool,
    /// 失敗の理由（失敗のジョブだけ。前の版のジョブマネージャーは送らない）
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub failure: Option<JobFailure>,
}

/// ジョブの失敗の理由
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JobFailure {
    pub reason: JobFailureReason,
    /// 英語の詳しい理由（ログに残すため）
    pub message: String,
}

/// ジョブの失敗の理由の種類（画面の言語の文言にするため）
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum JobFailureReason {
    /// 依存するジョブが、前のジョブでない
    DependencyNotEarlier { job: u64 },
    /// 依存するジョブが、実行できない
    DependencyCannotRun { job: u64 },
    /// 依存するジョブがない
    DependencyNotFound { job: u64 },
    /// 依存するジョブが失敗した
    DependencyFailed { job: u64 },
    /// 依存するジョブが取り消された
    DependencyCanceled { job: u64 },
    /// ランナーを起動できない
    RunnerNotStarted,
    /// ランナーが成果物を書かずに終わった
    RunnerWroteNoResult,
    /// ランナーの引数が正しくない
    RunnerInvalidArguments,
    /// ランナーが入力を読めない
    RunnerCannotReadInput,
    /// 入力が正しくない
    InvalidInput,
    /// 計算に失敗した
    CalculationFailed,
    /// ランナーが成果物を書けない
    RunnerCannotWriteResult,
    /// ツールパスの点が多すぎて、メモリの上限を超えるため、計算を止めた
    ToolpathTooLarge,
    /// ランナーが、ほかの終了の番号で終わった
    RunnerExitCode { code: i32 },
    /// ランナーが止められた
    RunnerStopped,
    /// 計算中にジョブマネージャーが止まった
    ManagerStopped,
    /// ジョブマネージャーを終了した
    ManagerQuit,
    /// 種類が分からない（記録が種類を持たない・読む側が知らない種類。英語の詳しい理由だけがある）
    #[serde(other)]
    Unknown,
}

/// まだジョブになっていない投入
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PendingSubmission {
    pub source: Option<JobSource>,
    /// 取り込めなかった
    pub rejected: bool,
}

/// ジョブのファイル（ジョブのフォルダの決まった名前のファイルだけ）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JobFile {
    /// 入力
    Input,
    /// 入力の CL（CL を入力にする切削シミュレーション）
    InputToolpath,
    /// 成果物
    Result,
}

/// 誤りの種類
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorKind {
    /// 正しくない要求
    InvalidRequest,
    /// 口の版が違う
    UnsupportedVersion,
    /// ないジョブ
    JobNotFound,
    /// ないファイル
    FileNotFound,
    /// 保存先を読み書きできない
    StoreUnavailable,
    /// 大きさの上限を超えた
    TooLarge,
}
