use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum EntityError {
    #[error("属性コード0は無効です")]
    InvalidAttributeCode,

    #[error("system属性コードが未定義です: {0}")]
    UnknownSystemAttributeCode(i32),

    #[error("属性型不一致: code={code}, expected={expected}, actual={actual}")]
    AttributeTypeMismatch {
        code: i32,
        expected: &'static str,
        actual: &'static str,
    },

    #[error("SingleLayer制約違反: entity={entity_id}, current_layers={current_layers}")]
    SingleLayerPolicyViolation {
        entity_id: String,
        current_layers: usize,
    },
}

pub type EntityResult<T> = Result<T, EntityError>;
