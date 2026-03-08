use crate::types::{JobId, JobStatus};

#[derive(Debug, Clone, PartialEq)]
pub enum JobEvent {
    /// 状態変更イベント
    StatusChanged {
        job_id: JobId,
        from: JobStatus,
        to: JobStatus,
    },
    /// 進捗更新イベント
    ProgressUpdated {
        job_id: JobId,
        progress: f32,
        message: Option<String>,
    },
    /// 成果物準備完了イベント
    ArtifactReady {
        job_id: JobId,
        result_ref: String,
    },
    /// 終了イベント
    Completed {
        job_id: JobId,
        status: JobStatus,
        result_ref: Option<String>,
        log_ref: Option<String>,
    },
}
