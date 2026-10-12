//! 変換で発生するエラー

use std::fmt;

/// 変換で発生するエラー
#[derive(Debug, Clone, PartialEq)]
pub enum TransformError {
    /// 扱わない変換（非一様スケール・ミラーリング・射影）、
    /// または形状が受け付けない変換（AABB の回転など）
    Unsupported(String),
    /// 変換結果が形状として成立しない
    InvalidGeometry(String),
    /// 変換の構築に不正な引数が渡された（0 以下のスケール、特異な行列など）
    InvalidParameter(String),
}

impl fmt::Display for TransformError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TransformError::Unsupported(msg) => write!(f, "unsupported transform: {msg}"),
            TransformError::InvalidGeometry(msg) => {
                write!(f, "invalid geometry after transform: {msg}")
            }
            TransformError::InvalidParameter(msg) => {
                write!(f, "invalid transform parameter: {msg}")
            }
        }
    }
}

impl std::error::Error for TransformError {}
