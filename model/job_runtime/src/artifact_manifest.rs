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
    MissingResultRef,
    InvalidResultRef(String),
    EmptyFormat,
    EmptyFormatVersion,
    InvalidImageDigest(String),
    InvalidSha256(String),
    InvalidCreatedAtUtc(String),
}

impl Display for ArtifactManifestError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingResultRef => write!(f, "result_ref is required"),
            Self::InvalidResultRef(value) => write!(f, "invalid result_ref: {}", value),
            Self::EmptyFormat => write!(f, "format is required"),
            Self::EmptyFormatVersion => write!(f, "format_version is required"),
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
    if result_ref.trim().is_empty() {
        return Err(ArtifactManifestError::MissingResultRef);
    }

    if !result_ref.starts_with("result://") {
        return Err(ArtifactManifestError::InvalidResultRef(
            result_ref.to_string(),
        ));
    }

    manifest.validate()
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
    use super::{ArtifactManifest, ArtifactManifestError, ArtifactType, validate_output_contract};
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
}
