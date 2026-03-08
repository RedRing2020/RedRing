use job_domain::{CamJobEvent, CamJobOutputValidity, CamJobRecord, CamJobStatus, CamJobType};

/// UI表示用のジョブ状態（Modelの状態を直接公開しない境界型）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JobStatusDto {
    Queued,
    Running,
    NeedsRecompute,
    Succeeded,
    Failed,
    Canceled,
}

impl From<CamJobStatus> for JobStatusDto {
    fn from(value: CamJobStatus) -> Self {
        match value {
            CamJobStatus::Queued => Self::Queued,
            CamJobStatus::Running => Self::Running,
            CamJobStatus::NeedsRecompute => Self::NeedsRecompute,
            CamJobStatus::Succeeded => Self::Succeeded,
            CamJobStatus::Failed => Self::Failed,
            CamJobStatus::Canceled => Self::Canceled,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArtifactValidityDto {
    Active,
    Superseded,
    InvalidatedByDependency,
}

impl From<CamJobOutputValidity> for ArtifactValidityDto {
    fn from(value: CamJobOutputValidity) -> Self {
        match value {
            CamJobOutputValidity::Active => Self::Active,
            CamJobOutputValidity::Superseded => Self::Superseded,
            CamJobOutputValidity::InvalidatedByDependency => Self::InvalidatedByDependency,
        }
    }
}

/// 成果物参照をUIへ渡すときのラッパー。
/// 生のref文字列ではなく有効性を含めて受け渡す。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArtifactRefDto {
    pub result_ref: String,
    pub log_ref: Option<String>,
    pub validity: ArtifactValidityDto,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiMessageArg {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiMessage {
    pub key: String,
    pub args: Vec<UiMessageArg>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiLocale {
    Ja,
    En,
}

#[derive(Debug, Clone, PartialEq)]
pub struct JobSummaryDto {
    pub job_id: u64,
    pub status: JobStatusDto,
    pub progress_percent: f32,
    pub artifact: Option<ArtifactRefDto>,
    pub failure_message: Option<UiMessage>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct JobDetailDto {
    pub summary: JobSummaryDto,
    pub parent_job_id: Option<u64>,
    pub group_id: Option<String>,
    pub attempts: u32,
    pub input_ref: String,
    pub job_type: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct JobProgressDto {
    pub job_id: u64,
    pub status: Option<JobStatusDto>,
    pub progress_percent: Option<f32>,
    pub artifact: Option<ArtifactRefDto>,
    pub message: Option<UiMessage>,
}

/// `ProgressUpdated` が無い場合のフォールバック進捗。
pub fn status_to_progress_percent(status: CamJobStatus) -> f32 {
    match status {
        CamJobStatus::Queued => 0.0,
        CamJobStatus::Running => 50.0,
        CamJobStatus::NeedsRecompute => 0.0,
        CamJobStatus::Succeeded | CamJobStatus::Failed | CamJobStatus::Canceled => 100.0,
    }
}

/// イベント進捗を優先し、無ければ状態推定へフォールバックする。
pub fn estimate_progress_percent(
    status: CamJobStatus,
    latest_progress_percent: Option<f32>,
) -> f32 {
    latest_progress_percent
        .map(|v| v.clamp(0.0, 100.0))
        .unwrap_or_else(|| status_to_progress_percent(status))
}

pub fn job_record_to_summary_dto(
    record: &CamJobRecord,
    latest_progress_percent: Option<f32>,
) -> JobSummaryDto {
    JobSummaryDto {
        job_id: record.id,
        status: record.status.into(),
        progress_percent: estimate_progress_percent(record.status, latest_progress_percent),
        artifact: active_artifact_dto(record),
        failure_message: record
            .last_error
            .as_deref()
            .map(normalize_job_error_message),
    }
}

pub fn job_record_to_detail_dto(
    record: &CamJobRecord,
    latest_progress_percent: Option<f32>,
) -> JobDetailDto {
    JobDetailDto {
        summary: job_record_to_summary_dto(record, latest_progress_percent),
        parent_job_id: record.parent_job_id,
        group_id: record.group_id.clone(),
        attempts: record.attempts,
        input_ref: record.spec.input_ref.clone(),
        job_type: job_type_key(record.spec.job_type.clone()).to_string(),
    }
}

/// job_runtime のイベントを、UIが扱いやすい差分DTOへ変換する。
pub fn job_event_to_progress_dto(event: &CamJobEvent) -> Option<JobProgressDto> {
    match event {
        CamJobEvent::StatusChanged { job_id, to } => Some(JobProgressDto {
            job_id: *job_id,
            status: Some((*to).into()),
            progress_percent: Some(status_to_progress_percent(*to)),
            artifact: None,
            message: Some(UiMessage {
                key: format!("job.status.{}", status_key(*to)),
                args: Vec::new(),
            }),
        }),
        CamJobEvent::ProgressUpdated {
            job_id,
            progress,
            message,
        } => Some(JobProgressDto {
            job_id: *job_id,
            status: None,
            progress_percent: Some((progress * 100.0).clamp(0.0, 100.0)),
            artifact: None,
            message: message.as_ref().map(|m| UiMessage {
                key: "job.progress.message".to_string(),
                args: vec![UiMessageArg {
                    name: "message".to_string(),
                    value: m.clone(),
                }],
            }),
        }),
        CamJobEvent::ArtifactReady { job_id, result_ref } => Some(JobProgressDto {
            job_id: *job_id,
            status: None,
            progress_percent: None,
            artifact: Some(ArtifactRefDto {
                result_ref: result_ref.clone(),
                log_ref: None,
                validity: ArtifactValidityDto::Active,
            }),
            message: None,
        }),
        CamJobEvent::Completed {
            job_id,
            status,
            result_ref,
            log_ref,
        } => Some(JobProgressDto {
            job_id: *job_id,
            status: Some((*status).into()),
            progress_percent: Some(status_to_progress_percent(*status)),
            artifact: result_ref.as_ref().map(|r| ArtifactRefDto {
                result_ref: r.clone(),
                log_ref: log_ref.clone(),
                validity: ArtifactValidityDto::Active,
            }),
            message: if *status == CamJobStatus::Failed {
                Some(UiMessage {
                    key: "job.error.unknown".to_string(),
                    args: Vec::new(),
                })
            } else {
                None
            },
        }),
        CamJobEvent::OutputValidityChanged {
            job_id,
            result_ref,
            validity,
        } => Some(JobProgressDto {
            job_id: *job_id,
            status: None,
            progress_percent: None,
            artifact: Some(ArtifactRefDto {
                result_ref: result_ref.clone(),
                log_ref: None,
                validity: (*validity).into(),
            }),
            message: Some(UiMessage {
                key: "job.artifact.validity_changed".to_string(),
                args: vec![UiMessageArg {
                    name: "validity".to_string(),
                    value: format!("{:?}", validity),
                }],
            }),
        }),
        CamJobEvent::MarkedNeedsRecompute { job_id, by_job_id } => Some(JobProgressDto {
            job_id: *job_id,
            status: Some(JobStatusDto::NeedsRecompute),
            progress_percent: Some(0.0),
            artifact: None,
            message: Some(UiMessage {
                key: "job.status.needs_recompute".to_string(),
                args: vec![UiMessageArg {
                    name: "by_job_id".to_string(),
                    value: by_job_id.to_string(),
                }],
            }),
        }),
        CamJobEvent::GroupProgressUpdated { .. } => None,
    }
}

/// 生エラーメッセージをUI向けのキー形式へ正規化する。
/// 実表示は `resolve_ui_message` 側でロケール解決する。
pub fn normalize_job_error_message(raw: &str) -> UiMessage {
    let code = if raw.contains("parent job not found") {
        "parent_job_not_found"
    } else if raw.contains("job not found") {
        "job_not_found"
    } else if raw.contains("retry limit exceeded") {
        "retry_limit_exceeded"
    } else if raw.contains("invalid transition") {
        "invalid_transition"
    } else if raw.contains("already terminal") {
        "already_terminal"
    } else if raw.contains("rerun not allowed") {
        "rerun_not_allowed"
    } else if raw.contains("invalid input_ref") {
        "invalid_input_ref"
    } else {
        "unknown"
    };

    UiMessage {
        key: format!("job.error.{code}"),
        args: vec![UiMessageArg {
            name: "raw".to_string(),
            value: raw.to_string(),
        }],
    }
}

/// `message_key + args` をロケール別テンプレートへ適用する。
pub fn resolve_ui_message(locale: UiLocale, message: &UiMessage) -> String {
    let template = match locale {
        UiLocale::Ja => resolve_ja_template(&message.key),
        UiLocale::En => resolve_en_template(&message.key),
    };

    let mut resolved = template.to_string();
    for arg in &message.args {
        resolved = resolved.replace(&format!("{{{}}}", arg.name), &arg.value);
    }
    resolved
}

fn active_artifact_dto(record: &CamJobRecord) -> Option<ArtifactRefDto> {
    record
        .output_history
        .iter()
        .find(|o| o.validity == CamJobOutputValidity::Active)
        .map(|o| ArtifactRefDto {
            result_ref: o.result_ref.clone(),
            log_ref: o.log_ref.clone(),
            validity: o.validity.into(),
        })
}

fn job_type_key(job_type: CamJobType) -> &'static str {
    match job_type {
        CamJobType::CamProcess => "job.type.cam_process",
        CamJobType::CuttingSimulation => "job.type.cutting_simulation",
    }
}

fn status_key(status: CamJobStatus) -> &'static str {
    match status {
        CamJobStatus::Queued => "queued",
        CamJobStatus::Running => "running",
        CamJobStatus::NeedsRecompute => "needs_recompute",
        CamJobStatus::Succeeded => "succeeded",
        CamJobStatus::Failed => "failed",
        CamJobStatus::Canceled => "canceled",
    }
}

fn resolve_ja_template(key: &str) -> &'static str {
    match key {
        "job.status.queued" => "ジョブは待機中です",
        "job.status.running" => "ジョブを実行中です",
        "job.status.needs_recompute" => "依存更新により再計算が必要です（起点: {by_job_id}）",
        "job.status.succeeded" => "ジョブは正常終了しました",
        "job.status.failed" => "ジョブは異常終了しました",
        "job.status.canceled" => "ジョブはキャンセルされました",
        "job.error.job_not_found" => "対象ジョブが見つかりません",
        "job.error.parent_job_not_found" => "親ジョブが見つかりません",
        "job.error.retry_limit_exceeded" => "リトライ上限を超えました",
        "job.error.invalid_transition" => "許可されていない状態遷移です",
        "job.error.already_terminal" => "終端状態のため操作できません",
        "job.error.rerun_not_allowed" => "この状態から再実行できません",
        "job.error.invalid_input_ref" => "入力参照が不正です",
        "job.progress.message" => "進捗メッセージ: {message}",
        "job.artifact.validity_changed" => "成果物状態が更新されました: {validity}",
        _ => "不明なメッセージです",
    }
}

fn resolve_en_template(key: &str) -> &'static str {
    match key {
        "job.status.queued" => "Job is queued",
        "job.status.running" => "Job is running",
        "job.status.needs_recompute" => {
            "Recompute is required due to dependency change (source: {by_job_id})"
        }
        "job.status.succeeded" => "Job completed successfully",
        "job.status.failed" => "Job failed",
        "job.status.canceled" => "Job was canceled",
        "job.error.job_not_found" => "Target job was not found",
        "job.error.parent_job_not_found" => "Parent job was not found",
        "job.error.retry_limit_exceeded" => "Retry limit exceeded",
        "job.error.invalid_transition" => "Invalid status transition",
        "job.error.already_terminal" => "Operation is not allowed on terminal status",
        "job.error.rerun_not_allowed" => "Rerun is not allowed from current status",
        "job.error.invalid_input_ref" => "Invalid input reference",
        "job.progress.message" => "Progress message: {message}",
        "job.artifact.validity_changed" => "Artifact validity updated: {validity}",
        _ => "Unknown message",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_record(status: CamJobStatus) -> CamJobRecord {
        CamJobRecord {
            id: 10,
            spec: job_domain::CamJobSpec {
                job_type: CamJobType::CamProcess,
                input_ref: "input://cam/a".to_string(),
            },
            parent_job_id: None,
            group_id: Some("g1".to_string()),
            status,
            attempts: 0,
            output_history: vec![cam_sim::CamJobOutputRecord {
                result_ref: "result://cam/1".to_string(),
                log_ref: Some("log://cam/1".to_string()),
                produced_at: std::time::SystemTime::now(),
                validity: CamJobOutputValidity::Active,
            }],
            last_error: None,
        }
    }

    #[test]
    fn progress_can_be_estimated_from_status() {
        let dto = job_record_to_summary_dto(&sample_record(CamJobStatus::Running), None);
        assert_eq!(dto.progress_percent, 50.0);
    }

    #[test]
    fn artifact_is_wrapped_by_option_artifact_ref_dto() {
        let dto = job_record_to_summary_dto(&sample_record(CamJobStatus::Succeeded), None);
        assert!(dto.artifact.is_some());
        assert_eq!(
            dto.artifact.expect("artifact expected").result_ref,
            "result://cam/1"
        );
    }

    #[test]
    fn error_is_normalized_to_message_key() {
        let message = normalize_job_error_message("parent job not found: 12");
        assert_eq!(message.key, "job.error.parent_job_not_found");
    }

    #[test]
    fn localized_message_is_resolved_from_key() {
        let message = UiMessage {
            key: "job.status.needs_recompute".to_string(),
            args: vec![UiMessageArg {
                name: "by_job_id".to_string(),
                value: "42".to_string(),
            }],
        };

        let ja = resolve_ui_message(UiLocale::Ja, &message);
        let en = resolve_ui_message(UiLocale::En, &message);
        assert!(ja.contains("42"));
        assert!(en.contains("42"));
    }
}
