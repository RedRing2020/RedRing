//! ジョブの記録（`jobs/<ジョブの番号>/job.json`）と、投入の要求（`inbox/<投入の ID>/request.json`）の形

use serde::{Deserialize, Serialize};

/// 記録・要求の形式の版（書くときの版）
///
/// 版 2 で、投入元に改訂の鍵を足した。版 3 で、切削シミュレーションの入力に CL のファイルを持てるようにした。前の版を
/// 読む側は知らない項目を読み捨てるため、要求の項目を足したときも版を上げ、前の版のジョブマネージャーが新しい要求を
/// 黙って取り込まない（拒む）ようにする。記録の失敗の理由の種類（`error`）は、読み捨てても英語の文（`last_error`）が
/// 残るため、版を上げずに足した。
pub const STORE_FORMAT_VERSION: u32 = 3;

/// 中断（[`StoredJobStatus::Interrupted`]）の記録の形式の版（前の版の読む側は中断を知らないため、版を上げて、読めない
/// 記録として扱わせる。ほかの状態の記録は、表せる最も古い版 [`STORE_FORMAT_VERSION`] で書く）
pub(super) const INTERRUPTED_FORMAT_VERSION: u32 = 4;

/// 切削シミュレーションの入力に CL のファイルを持てる、最も古い要求の形式の版
pub const TOOLPATH_FILE_FORMAT_VERSION: u32 = 3;

/// 前の版で書ける要求の、最も新しい形式の版（CL のファイルを持たない要求）
const BASE_REQUEST_FORMAT_VERSION: u32 = 2;

/// ジョブの列 `jobs` を表せる、最も古い要求の形式の版（前の版のジョブマネージャーも取り込めるよう、新しい項目を
/// 使わない要求は前の版で書く）
pub fn request_format_version(jobs: &[SubmittedJob]) -> u32 {
    if jobs.iter().any(|job| job.toolpath_file.is_some()) {
        TOOLPATH_FILE_FORMAT_VERSION
    } else {
        BASE_REQUEST_FORMAT_VERSION
    }
}

/// 読める最も古い記録・要求の形式の版
pub(super) const OLDEST_STORE_FORMAT_VERSION: u32 = 1;

/// ランナーで実行するジョブの種類
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StoredJobKind {
    /// ツールパス生成
    Cam,
    /// 切削シミュレーション（依存するツールパス生成のジョブの成果物を入力にする）
    Sim,
}

/// ジョブの状態
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StoredJobStatus {
    Queued,
    Running,
    Succeeded,
    Failed,
    Canceled,
    /// 計算の途中で、ジョブマネージャーの終了・異常終了・OS の再起動で止まった（利用者が選ぶまで計算しない）
    Interrupted,
}

impl StoredJobStatus {
    /// 終わった状態か（正常に終わった・失敗・取り消し。中断は、続けて計算できるため含めない）
    pub fn is_terminal(self) -> bool {
        matches!(self, Self::Succeeded | Self::Failed | Self::Canceled)
    }
}

/// 投入元（工程の ID と改訂の番号・改訂の鍵など。権限の判断には使わない）
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StoredJobSource {
    pub source_id: String,
    pub revision: u32,
    /// 改訂の鍵（前の版の記録・要求は持たない）
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub revision_key: Option<String>,
}

/// ジョブの失敗の理由の種類（画面の言語の文言にするため。英語の文は `Display` で返し、記録の `last_error` に残す）
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum StoredJobError {
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
    /// ツールパスがメモリの上限を超えるため、計算を止めた
    ToolpathTooLarge,
    /// ランナーが、ほかの終了の番号で終わった
    RunnerExitCode { code: i32 },
    /// ランナーが止められた
    RunnerStopped,
    /// 計算中にジョブマネージャーが止まった
    ManagerStopped,
    /// ジョブマネージャーを終了した
    ManagerQuit,
    /// 読む側が知らない種類（新しい版で足した種類。英語の文 `last_error` を出す）
    #[serde(other)]
    Unknown,
}

impl std::fmt::Display for StoredJobError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DependencyNotEarlier { job } => {
                write!(f, "dependency job {job} is not an earlier job")
            }
            Self::DependencyCannotRun { job } => write!(f, "dependency job {job} cannot run"),
            Self::DependencyNotFound { job } => write!(f, "dependency job {job} is not found"),
            Self::DependencyFailed { job } => write!(f, "dependency job {job} failed"),
            Self::DependencyCanceled { job } => write!(f, "dependency job {job} was canceled"),
            Self::RunnerNotStarted => write!(f, "cannot start the runner"),
            Self::RunnerWroteNoResult => write!(f, "the runner wrote no result"),
            Self::RunnerInvalidArguments => write!(f, "the runner arguments are invalid"),
            Self::RunnerCannotReadInput => write!(f, "the runner cannot read the input"),
            Self::InvalidInput => write!(f, "the input is invalid"),
            Self::CalculationFailed => write!(f, "the calculation failed"),
            Self::RunnerCannotWriteResult => write!(f, "the runner cannot write the result"),
            Self::ToolpathTooLarge => {
                write!(f, "the toolpath is too large for the memory limit")
            }
            Self::RunnerExitCode { code } => write!(f, "the runner ended with exit code {code}"),
            Self::RunnerStopped => write!(f, "the runner was stopped"),
            Self::ManagerStopped => write!(f, "the job manager stopped while the job was running"),
            Self::ManagerQuit => write!(f, "the job manager was quit"),
            Self::Unknown => write!(f, "unknown error"),
        }
    }
}

/// ジョブの記録
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StoredJob {
    pub format_version: u32,
    pub job_id: u64,
    /// 投入の ID（投入した側が、自分のジョブを引くのに使う）
    pub submission_id: String,
    pub kind: StoredJobKind,
    /// 依存するジョブ（すべて正常に終わってから実行する）
    pub depends_on: Vec<u64>,
    /// 優先順位（大きいほど先）
    pub priority: i32,
    pub group_id: Option<String>,
    pub source: Option<StoredJobSource>,
    pub status: StoredJobStatus,
    /// 実行した回数
    pub attempts: u32,
    /// 作成・更新の時刻（UNIX 時刻のミリ秒）
    pub created_at_ms: u64,
    pub updated_at_ms: u64,
    /// 最後に計算を始めた時刻（UNIX 時刻のミリ秒。まだ始めていなければ `None`）
    #[serde(default)]
    pub started_at_ms: Option<u64>,
    pub result_ref: Option<String>,
    pub log_ref: Option<String>,
    /// 直近の誤り（英語の文。ログと照らし合わせるため）
    pub last_error: Option<String>,
    /// 直近の誤りの種類（前の版の記録・種類を持たない誤りは `None`）
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<StoredJobError>,
}

/// 投入の要求
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SubmissionRequest {
    pub format_version: u32,
    pub jobs: Vec<SubmittedJob>,
}

/// 投入するジョブ
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SubmittedJob {
    pub kind: StoredJobKind,
    /// 入力のファイルの名前（投入のフォルダの中のファイル）
    pub input_file: String,
    #[serde(default)]
    pub depends_on: Vec<DependencyRef>,
    #[serde(default)]
    pub priority: i32,
    #[serde(default)]
    pub group_id: Option<String>,
    #[serde(default)]
    pub source: Option<StoredJobSource>,
    /// 切削シミュレーションが入力にする CL のファイルの名前（投入のフォルダの中のファイル）。指定するときは、ツールパス
    /// 生成のジョブに依存しない
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub toolpath_file: Option<String>,
}

/// 依存先
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DependencyRef {
    /// 同じ投入の中の、前に並ぶジョブ（`jobs` の番号）
    Submitted(usize),
    /// すでにあるジョブの番号
    Existing(u64),
}
