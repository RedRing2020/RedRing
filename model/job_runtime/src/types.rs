use std::error::Error;
use std::fmt::{Display, Formatter};
use std::time::SystemTime;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
/// 登録済みジョブを識別するID
pub struct JobId(pub u64);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JobType {
    /// 加工計画ジョブ
    CamProcessBatch,
    /// シミュレーションジョブ
    CuttingSimulationBatch,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JobStatus {
    /// 受け付け済みで未実行
    Queued,
    /// 実行中
    Running,
    /// 正常終了
    Succeeded,
    /// 異常終了
    Failed,
    /// キャンセル済み
    Canceled,
}

impl JobStatus {
    pub fn is_terminal(self) -> bool {
        matches!(self, Self::Succeeded | Self::Failed | Self::Canceled)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// リトライ設定
pub struct RetryPolicy {
    /// 最大リトライ回数
    pub max_retries: u32,
    /// リトライ間隔（ミリ秒）
    pub backoff_millis: u64,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_retries: 3,
            backoff_millis: 1000,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// ジョブ投入時の入力定義
pub struct JobSpec {
    /// 実行種別
    pub job_type: JobType,
    /// 入力参照
    pub input_ref: String,
    /// タイムアウト秒
    pub timeout_secs: u64,
    /// リトライ設定
    pub retry_policy: RetryPolicy,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// ジョブの実行状態
pub struct JobRecord {
    /// ジョブID
    pub id: JobId,
    /// 投入時の設定
    pub spec: JobSpec,
    /// 現在状態
    pub status: JobStatus,
    /// 実施済みリトライ回数
    pub attempts: u32,
    /// 作成時刻
    pub created_at: SystemTime,
    /// 最終更新時刻
    pub updated_at: SystemTime,
    /// 結果参照
    pub result_ref: Option<String>,
    /// ログ参照
    pub log_ref: Option<String>,
    /// 直近エラー
    pub last_error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JobError {
    /// 指定ジョブが存在しない
    JobNotFound(JobId),
    /// 状態遷移が許可されていない
    InvalidTransition { from: JobStatus, to: JobStatus },
    /// リトライ上限超過
    RetryLimitExceeded { attempts: u32, max_retries: u32 },
    /// すでに終端状態
    AlreadyTerminal(JobStatus),
}

impl Display for JobError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::JobNotFound(id) => write!(f, "job not found: {}", id.0),
            Self::InvalidTransition { from, to } => {
                write!(f, "invalid transition: {:?} -> {:?}", from, to)
            }
            Self::RetryLimitExceeded {
                attempts,
                max_retries,
            } => write!(
                f,
                "retry limit exceeded: attempts={}, max_retries={}",
                attempts, max_retries
            ),
            Self::AlreadyTerminal(status) => write!(f, "job already terminal: {:?}", status),
        }
    }
}

impl Error for JobError {}
