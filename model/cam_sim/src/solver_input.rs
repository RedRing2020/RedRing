//! CamProcess ジョブの `InputRef` → solver 入力の解決境界（#684 §8.2）
//!
//! Job Manager は payload 本体を解釈しない。`cam_sim` の Job アダプタが
//! 本境界を通じて `InputRef` を `CamSolverInput` へ解決する。

use std::collections::HashMap;
use std::sync::RwLock;

use cam_algorithms::CamSolverInput;
use job_runtime::{RefParser, RefValidationError};

/// CamProcess の入力参照ドメイン
pub const CAM_INPUT_DOMAIN: &str = "cam";

/// `InputRef` から solver 入力を解決する境界
pub trait CamSolverInputProvider: Send + Sync {
    /// 参照先の solver 入力を返す。未登録の場合は `None`。
    fn resolve(&self, input_ref: &str) -> Option<CamSolverInput>;
}

/// プロセス内メモリで solver 入力を保持する provider
#[derive(Debug, Default)]
pub struct InMemoryCamSolverInputStore {
    inputs: RwLock<HashMap<String, CamSolverInput>>,
}

impl InMemoryCamSolverInputStore {
    pub fn new() -> Self {
        Self::default()
    }

    /// `input://cam/...` 形式の参照に solver 入力を登録する。同一参照は上書きする。
    pub fn register(
        &self,
        input_ref: &str,
        input: CamSolverInput,
    ) -> Result<(), RefValidationError> {
        validate_cam_input_ref(input_ref)?;
        self.inputs
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .insert(input_ref.trim().to_string(), input);
        Ok(())
    }
}

impl CamSolverInputProvider for InMemoryCamSolverInputStore {
    fn resolve(&self, input_ref: &str) -> Option<CamSolverInput> {
        self.inputs
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .get(input_ref.trim())
            .cloned()
    }
}

/// `input://cam/<name>...` 形式であることを検証する。
pub fn validate_cam_input_ref(input_ref: &str) -> Result<(), RefValidationError> {
    let parsed = RefParser::parse_input(input_ref)?;
    let segments = parsed.segments();
    match segments.first() {
        Some(domain) if domain == CAM_INPUT_DOMAIN => {}
        Some(domain) => {
            return Err(RefValidationError::DomainMismatch {
                expected: CAM_INPUT_DOMAIN.to_string(),
                actual: domain.clone(),
            });
        }
        None => {
            return Err(RefValidationError::MissingDomain {
                value: parsed.as_str().to_string(),
            });
        }
    }
    if segments.len() < 2 {
        return Err(RefValidationError::MissingSuffix {
            value: parsed.as_str().to_string(),
        });
    }
    Ok(())
}
