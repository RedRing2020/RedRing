use std::error::Error;
use std::fmt::{Display, Formatter};
use std::time::SystemTime;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
/// 登録済みジョブを識別するID
pub struct JobId(pub u64);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JobType {
    /// 加工計画ジョブ
    CamProcess,
    /// シミュレーションジョブ
    CuttingSimulation,
    /// CAM成果物を入力とするNCポスト処理ジョブ
    NcPostFromCam,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JobStatus {
    /// 受け付け済みで未実行
    Queued,
    /// 実行中
    Running,
    /// 依存更新により再計算待ち
    NeedsRecompute,
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

#[derive(Debug, Clone, PartialEq, Eq, Default)]
/// ジョブ関連情報
pub struct JobRelation {
    /// 親ジョブID
    pub parent_job_id: Option<JobId>,
    /// グループID
    pub group_id: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// 成果物の有効性
pub enum JobOutputValidity {
    /// 現在有効な成果物
    Active,
    /// 後続成果物により置き換え済み
    Superseded,
    /// 依存更新により無効化
    InvalidatedByDependency,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// ジョブ成果物履歴
pub struct JobOutputRecord {
    /// 成果物参照
    pub result_ref: String,
    /// 成果物バイナリ
    pub artifact_bytes: Option<Vec<u8>>,
    /// ログ参照
    pub log_ref: Option<String>,
    /// 生成時刻
    pub produced_at: SystemTime,
    /// 有効性
    pub validity: JobOutputValidity,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// ジョブの実行状態
pub struct JobRecord {
    /// ジョブID
    pub id: JobId,
    /// 投入時の設定
    pub spec: JobSpec,
    /// 親ジョブID
    pub parent_job_id: Option<JobId>,
    /// グループID
    pub group_id: Option<String>,
    /// 現在状態
    pub status: JobStatus,
    /// 実施済みリトライ回数
    pub attempts: u32,
    /// 作成時刻
    pub created_at: SystemTime,
    /// 最終更新時刻
    pub updated_at: SystemTime,
    /// 成果物履歴
    pub output_history: Vec<JobOutputRecord>,
    /// 直近エラー
    pub last_error: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
/// グループ集約情報
pub struct JobGroupSummary {
    /// グループID
    pub group_id: String,
    /// ジョブ総数
    pub total: usize,
    /// 待機数
    pub queued: usize,
    /// 実行中数
    pub running: usize,
    /// 成功数
    pub succeeded: usize,
    /// 失敗数
    pub failed: usize,
    /// キャンセル数
    pub canceled: usize,
    /// 集約状態
    pub status: JobStatus,
    /// 進捗率
    pub progress: f32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JobError {
    /// 指定ジョブが存在しない
    JobNotFound(JobId),
    /// 指定親ジョブが存在しない
    ParentJobNotFound(JobId),
    /// 再実行が許可されていない
    RerunNotAllowed(JobStatus),
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
            Self::ParentJobNotFound(id) => write!(f, "parent job not found: {}", id.0),
            Self::RerunNotAllowed(status) => {
                write!(f, "rerun not allowed from status: {:?}", status)
            }
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
