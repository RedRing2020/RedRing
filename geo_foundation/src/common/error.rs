/// 幾何計算用のエラー型定義
use std::fmt;

/// 幾何計算エラー
#[derive(Debug, Clone, PartialEq)]
pub enum GeometryError {
    /// 無効な引数
    InvalidArgument(String),
    /// 計算エラー
    ComputationError(String),
    /// 許容誤差外
    ToleranceExceeded(String),
    /// 次元不整合
    DimensionMismatch(String),
}

impl fmt::Display for GeometryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GeometryError::InvalidArgument(msg) => write!(f, "Invalid argument: {}", msg),
            GeometryError::ComputationError(msg) => write!(f, "Computation error: {}", msg),
            GeometryError::ToleranceExceeded(msg) => write!(f, "Tolerance exceeded: {}", msg),
            GeometryError::DimensionMismatch(msg) => write!(f, "Dimension mismatch: {}", msg),
        }
    }
}

impl std::error::Error for GeometryError {}

/// 幾何計算の結果型
pub type GeometryResult<T> = Result<T, GeometryError>;