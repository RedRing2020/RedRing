//! 要求と応答の型（JSON の枠で送る。設計: JOB_IPC_DESIGN.md 4・5 章）

use serde::{Deserialize, Serialize};

/// 口の版（要求・応答の形を変えたら上げる。要求を足すだけなら上げない。前のジョブマネージャーは、知らない要求に
/// 正しくない要求の誤りを返す）
pub const PROTOCOL_VERSION: u32 = 1;

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
    /// ジョブを取り消す（計算待ちは取り消し、計算中はランナーを止める。終わったジョブ・ないジョブは何もしない。
    /// 取り消し終えるのを待たない）
    Cancel { job_ids: Vec<u64> },
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
    /// 誤り（種類と、英語の詳しい理由）
    Error {
        kind: ErrorKind,
        message: String,
    },
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
