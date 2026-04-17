use std::error::Error;
use std::fmt::{Display, Formatter};

use crate::types::JobId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArtifactType {
    Toolpath,
    Interference,
    Generic,
}

impl ArtifactType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Toolpath => "toolpath",
            Self::Interference => "interference",
            Self::Generic => "generic",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArtifactManifest {
    pub artifact_type: ArtifactType,
    pub format: String,
    pub format_version: String,
    pub producer_job_id: JobId,
    pub image_digest: String,
    pub sha256: String,
    pub size_bytes: u64,
    pub created_at_utc: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArtifactManifestError {
    MissingInputRef,
    InvalidInputRef(String),
    MissingResultRef,
    InvalidResultRef(String),
    EmptyFormat,
    EmptyFormatVersion,
    FormatVersionMismatch { expected: String, actual: String },
    InvalidImageDigest(String),
    InvalidSha256(String),
    InvalidCreatedAtUtc(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContractValidationDecision {
    Accept,
    Reject,
    RetryRequired,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormatVersionCompatibility {
    Compatible,
    Incompatible,
}

impl Display for ArtifactManifestError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingInputRef => write!(f, "input_ref is required"),
            Self::InvalidInputRef(value) => write!(f, "invalid input_ref: {}", value),
            Self::MissingResultRef => write!(f, "result_ref is required"),
            Self::InvalidResultRef(value) => write!(f, "invalid result_ref: {}", value),
            Self::EmptyFormat => write!(f, "format is required"),
            Self::EmptyFormatVersion => write!(f, "format_version is required"),
            Self::FormatVersionMismatch { expected, actual } => write!(
                f,
                "format_version mismatch: expected={}, actual={}",
                expected, actual
            ),
            Self::InvalidImageDigest(value) => write!(f, "invalid image_digest: {}", value),
            Self::InvalidSha256(value) => write!(f, "invalid sha256: {}", value),
            Self::InvalidCreatedAtUtc(value) => write!(f, "invalid created_at_utc: {}", value),
        }
    }
}

impl Error for ArtifactManifestError {}

impl ArtifactManifest {
    /// Job Manager責務: 成果物メタデータ契約のみを検証する。
    pub fn validate(&self) -> Result<(), ArtifactManifestError> {
        if self.format.trim().is_empty() {
            return Err(ArtifactManifestError::EmptyFormat);
        }

        if self.format_version.trim().is_empty() {
            return Err(ArtifactManifestError::EmptyFormatVersion);
        }

        if !is_valid_image_digest(&self.image_digest) {
            return Err(ArtifactManifestError::InvalidImageDigest(
                self.image_digest.clone(),
            ));
        }

        if !is_valid_sha256(&self.sha256) {
            return Err(ArtifactManifestError::InvalidSha256(self.sha256.clone()));
        }

        if !is_likely_rfc3339_utc(&self.created_at_utc) {
            return Err(ArtifactManifestError::InvalidCreatedAtUtc(
                self.created_at_utc.clone(),
            ));
        }

        Ok(())
    }
}

pub fn validate_output_contract(
    result_ref: &str,
    manifest: &ArtifactManifest,
) -> Result<(), ArtifactManifestError> {
    validate_ref(
        result_ref,
        "result://",
        ArtifactManifestError::MissingResultRef,
        ArtifactManifestError::InvalidResultRef,
    )?;

    manifest.validate()?;

    Ok(())
}

/// Job Manager責務: InputRef/ResultRefとmanifestバージョン契約を検証する。
pub fn validate_io_contract(
    input_ref: &str,
    result_ref: &str,
    manifest: &ArtifactManifest,
    expected_format_version: &str,
) -> Result<(), ArtifactManifestError> {
    validate_ref(
        input_ref,
        "input://",
        ArtifactManifestError::MissingInputRef,
        ArtifactManifestError::InvalidInputRef,
    )?;

    validate_ref(
        result_ref,
        "result://",
        ArtifactManifestError::MissingResultRef,
        ArtifactManifestError::InvalidResultRef,
    )?;

    manifest.validate()?;

    // 非互換なmanifest versionは受理せずrejectする。
    if evaluate_format_version_compatibility(&manifest.format_version, expected_format_version)
        == FormatVersionCompatibility::Incompatible
    {
        return Err(ArtifactManifestError::FormatVersionMismatch {
            expected: expected_format_version.to_string(),
            actual: manifest.format_version.clone(),
        });
    }

    Ok(())
}

/// v1時点では format_version は厳密一致のみを互換とみなす。
pub fn evaluate_format_version_compatibility(
    actual: &str,
    expected: &str,
) -> FormatVersionCompatibility {
    if actual == expected {
        FormatVersionCompatibility::Compatible
    } else {
        FormatVersionCompatibility::Incompatible
    }
}

/// Job Manager責務: 検証エラーを運用上の意思決定へ正規化する。
pub fn decide_contract_validation_error(
    error: &ArtifactManifestError,
) -> ContractValidationDecision {
    match error {
        ArtifactManifestError::MissingInputRef
        | ArtifactManifestError::InvalidInputRef(_)
        | ArtifactManifestError::MissingResultRef
        | ArtifactManifestError::InvalidResultRef(_)
        | ArtifactManifestError::FormatVersionMismatch { .. }
        | ArtifactManifestError::InvalidImageDigest(_)
        | ArtifactManifestError::InvalidSha256(_)
        | ArtifactManifestError::EmptyFormat
        | ArtifactManifestError::EmptyFormatVersion
        | ArtifactManifestError::InvalidCreatedAtUtc(_) => ContractValidationDecision::Reject,
    }
}

fn validate_ref<F>(
    value: &str,
    expected_scheme: &str,
    missing_error: ArtifactManifestError,
    invalid_error: F,
) -> Result<(), ArtifactManifestError>
where
    F: FnOnce(String) -> ArtifactManifestError,
{
    if value.trim().is_empty() {
        return Err(missing_error);
    }

    let Some(suffix) = value.strip_prefix(expected_scheme) else {
        return Err(invalid_error(value.to_string()));
    };

    if suffix.trim().is_empty() {
        return Err(invalid_error(value.to_string()));
    }

    Ok(())
}

fn is_valid_image_digest(value: &str) -> bool {
    if !value.starts_with("sha256:") {
        return false;
    }

    let hash = &value[7..];
    is_hex_with_len(hash, 64)
}

fn is_valid_sha256(value: &str) -> bool {
    is_hex_with_len(value, 64)
}

fn is_hex_with_len(value: &str, len: usize) -> bool {
    value.len() == len && value.bytes().all(|b| b.is_ascii_hexdigit())
}

fn is_likely_rfc3339_utc(value: &str) -> bool {
    value.contains('T') && value.ends_with('Z')
}

#[cfg(test)]
mod tests {
    use super::{
        ArtifactManifest, ArtifactManifestError, ArtifactType, ContractValidationDecision,
        FormatVersionCompatibility, decide_contract_validation_error,
        evaluate_format_version_compatibility, validate_io_contract, validate_output_contract,
    };
    use crate::types::JobId;

    fn sample_manifest() -> ArtifactManifest {
        ArtifactManifest {
            artifact_type: ArtifactType::Toolpath,
            format: "binary".to_string(),
            format_version: "v1".to_string(),
            producer_job_id: JobId(42),
            image_digest: format!("sha256:{}", "a".repeat(64)),
            sha256: "b".repeat(64),
            size_bytes: 1024,
            created_at_utc: "2026-03-08T09:00:00Z".to_string(),
        }
    }

    #[test]
    fn validate_manifest_accepts_minimal_v1() {
        let manifest = sample_manifest();
        assert_eq!(manifest.validate(), Ok(()));
    }

    #[test]
    fn validate_manifest_rejects_invalid_digest() {
        let mut manifest = sample_manifest();
        manifest.image_digest = "sha256:xyz".to_string();

        assert!(matches!(
            manifest.validate(),
            Err(ArtifactManifestError::InvalidImageDigest(_))
        ));
    }

    #[test]
    fn validate_output_contract_rejects_missing_result_ref() {
        let manifest = sample_manifest();

        assert_eq!(
            validate_output_contract("", &manifest),
            Err(ArtifactManifestError::MissingResultRef)
        );
    }

    #[test]
    fn validate_output_contract_requires_result_scheme() {
        let manifest = sample_manifest();

        assert!(matches!(
            validate_output_contract("output://artifact", &manifest),
            Err(ArtifactManifestError::InvalidResultRef(_))
        ));
    }

    #[test]
    fn validate_output_contract_rejects_empty_result_suffix() {
        let manifest = sample_manifest();

        assert!(matches!(
            validate_output_contract("result://   ", &manifest),
            Err(ArtifactManifestError::InvalidResultRef(_))
        ));
    }

    #[test]
    fn validate_io_contract_rejects_missing_input_ref() {
        let manifest = sample_manifest();

        assert_eq!(
            validate_io_contract("", "result://artifact-1", &manifest, "v1"),
            Err(ArtifactManifestError::MissingInputRef)
        );
    }

    #[test]
    fn validate_io_contract_requires_input_scheme() {
        let manifest = sample_manifest();

        assert!(matches!(
            validate_io_contract("output://job-42", "result://artifact-1", &manifest, "v1"),
            Err(ArtifactManifestError::InvalidInputRef(_))
        ));
    }

    #[test]
    fn validate_io_contract_rejects_empty_input_suffix() {
        let manifest = sample_manifest();

        assert!(matches!(
            validate_io_contract("input://   ", "result://artifact-1", &manifest, "v1"),
            Err(ArtifactManifestError::InvalidInputRef(_))
        ));
    }

    #[test]
    fn validate_io_contract_rejects_version_mismatch() {
        let mut manifest = sample_manifest();
        manifest.format_version = "v2".to_string();

        assert_eq!(
            validate_io_contract("input://job-42", "result://artifact-1", &manifest, "v1"),
            Err(ArtifactManifestError::FormatVersionMismatch {
                expected: "v1".to_string(),
                actual: "v2".to_string(),
            })
        );
    }

    #[test]
    fn validate_io_contract_accepts_valid_minimum_set() {
        let manifest = sample_manifest();

        assert_eq!(
            validate_io_contract("input://job-42", "result://artifact-1", &manifest, "v1"),
            Ok(())
        );
    }

    #[test]
    fn evaluate_format_version_compatibility_requires_exact_match() {
        assert_eq!(
            evaluate_format_version_compatibility("v1", "v1"),
            FormatVersionCompatibility::Compatible
        );
        assert_eq!(
            evaluate_format_version_compatibility("v2", "v1"),
            FormatVersionCompatibility::Incompatible
        );
    }

    #[test]
    fn decide_contract_validation_error_for_missing_ref_is_reject() {
        assert_eq!(
            decide_contract_validation_error(&ArtifactManifestError::MissingInputRef),
            ContractValidationDecision::Reject
        );
    }

    #[test]
    fn decide_contract_validation_error_for_version_mismatch_is_reject() {
        assert_eq!(
            decide_contract_validation_error(&ArtifactManifestError::FormatVersionMismatch {
                expected: "v1".to_string(),
                actual: "v2".to_string(),
            }),
            ContractValidationDecision::Reject
        );
    }

    #[test]
    fn decide_contract_validation_error_for_invalid_hash_is_reject() {
        assert_eq!(
            decide_contract_validation_error(&ArtifactManifestError::InvalidSha256(
                "x".to_string()
            )),
            ContractValidationDecision::Reject
        );
    }
}
