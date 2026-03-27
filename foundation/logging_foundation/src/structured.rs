/// `error_kind` フィールドの定数値。
///
/// tracing の構造化フィールドに使用する:
/// ```rust
/// tracing::error!(
///     error_key = "validation.machine.feed_rate_limit_exceeded",
///     error_kind = logging_foundation::ERROR_KIND_VALIDATION,
///     message = "Feed rate exceeds machine limit: ...",
///     "cam simulation visualization failed"
/// );
/// ```
pub const ERROR_KIND_VALIDATION: &str = "validation";
pub const ERROR_KIND_SIMULATION: &str = "simulation";
pub const ERROR_KIND_SYSTEM: &str = "system";
pub const ERROR_KIND_APP: &str = "app";
