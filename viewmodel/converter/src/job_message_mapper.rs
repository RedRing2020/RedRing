use i18n_foundation::{UiMessage, UiMessageArg};

/// JobError相当の生文字列を、Job専用の message key へ正規化する。
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
