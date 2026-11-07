//! ジョブの記録（`jobs/<ジョブの番号>/job.json`）と、投入の要求（`inbox/<投入の ID>/request.json`）の形

use serde::{Deserialize, Serialize};

/// 記録・要求の形式の版
pub const STORE_FORMAT_VERSION: u32 = 1;

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
}

impl StoredJobStatus {
    /// 終わった状態か（正常に終わった・失敗・取り消し）
    pub fn is_terminal(self) -> bool {
        matches!(self, Self::Succeeded | Self::Failed | Self::Canceled)
    }
}

/// 投入元（工程の ID と改訂の番号など。権限の判断には使わない）
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StoredJobSource {
    pub source_id: String,
    pub revision: u32,
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
    pub last_error: Option<String>,
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
