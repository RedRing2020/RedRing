use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum EntityError {
    #[error("attribute code 0 is invalid")]
    InvalidAttributeCode,

    #[error("system attribute code is not defined: {0}")]
    UnknownSystemAttributeCode(i32),

    #[error("attribute type mismatch: code={code}, expected={expected}, actual={actual}")]
    AttributeTypeMismatch {
        code: i32,
        expected: &'static str,
        actual: &'static str,
    },

    #[error("SingleLayer constraint violated: entity={entity_id}, current_layers={current_layers}")]
    SingleLayerPolicyViolation {
        entity_id: String,
        current_layers: usize,
    },
}

pub type EntityResult<T> = Result<T, EntityError>;
