use crate::message_catalog::{resolve_message, TableMessageCatalog, UiLocale, UiMessage};

const JOB_TEMPLATE_JA: &[(&str, &str)] = &[
    ("job.status.queued", "ジョブは待機中です"),
    ("job.status.running", "ジョブを実行中です"),
    (
        "job.status.needs_recompute",
        "依存更新により再計算が必要です（起点: {by_job_id}）",
    ),
    ("job.status.succeeded", "ジョブは正常終了しました"),
    ("job.status.failed", "ジョブは異常終了しました"),
    ("job.status.canceled", "ジョブはキャンセルされました"),
    ("job.error.job_not_found", "対象ジョブが見つかりません"),
    ("job.error.parent_job_not_found", "親ジョブが見つかりません"),
    ("job.error.retry_limit_exceeded", "リトライ上限を超えました"),
    (
        "job.error.invalid_transition",
        "許可されていない状態遷移です",
    ),
    ("job.error.already_terminal", "終端状態のため操作できません"),
    (
        "job.error.rerun_not_allowed",
        "この状態から再実行できません",
    ),
    ("job.error.invalid_input_ref", "入力参照が不正です"),
    ("job.progress.message", "進捗メッセージ: {message}"),
    (
        "job.artifact.validity_changed",
        "成果物状態が更新されました: {validity}",
    ),
];

const JOB_TEMPLATE_EN: &[(&str, &str)] = &[
    ("job.status.queued", "Job is queued"),
    ("job.status.running", "Job is running"),
    (
        "job.status.needs_recompute",
        "Recompute is required due to dependency change (source: {by_job_id})",
    ),
    ("job.status.succeeded", "Job completed successfully"),
    ("job.status.failed", "Job failed"),
    ("job.status.canceled", "Job was canceled"),
    ("job.error.job_not_found", "Target job was not found"),
    ("job.error.parent_job_not_found", "Parent job was not found"),
    ("job.error.retry_limit_exceeded", "Retry limit exceeded"),
    ("job.error.invalid_transition", "Invalid status transition"),
    (
        "job.error.already_terminal",
        "Operation is not allowed on terminal status",
    ),
    (
        "job.error.rerun_not_allowed",
        "Rerun is not allowed from current status",
    ),
    ("job.error.invalid_input_ref", "Invalid input reference"),
    ("job.progress.message", "Progress message: {message}"),
    (
        "job.artifact.validity_changed",
        "Artifact validity updated: {validity}",
    ),
];

pub const JOB_MESSAGE_CATALOG: TableMessageCatalog = TableMessageCatalog::new(
    JOB_TEMPLATE_JA,
    JOB_TEMPLATE_EN,
    "不明なメッセージです",
    "Unknown message",
);

pub fn resolve_job_message(locale: UiLocale, message: &UiMessage) -> String {
    resolve_message(&JOB_MESSAGE_CATALOG, locale, message)
}
