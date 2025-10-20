//! Transform エラー型定義

use std::fmt;

/// Transform 操作で発生する可能性のあるエラー
#[derive(Debug, Clone, PartialEq)]
pub enum TransformError {
    /// 変換後の幾何構造体が無効（例：ゼロスケール）
    InvalidGeometry(String),
    /// ゼロベクトル（正規化不可能）
    ZeroVector(String),
    /// 不正なスケール倍率
    InvalidScaleFactor(String),
    /// 不正な回転パラメータ
    InvalidRotation(String),
}

impl fmt::Display for TransformError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TransformError::InvalidGeometry(msg) => {
                write!(f, "変換後の幾何構造体が無効です: {}", msg)
            }
            TransformError::ZeroVector(msg) => write!(f, "ゼロベクトルは正規化できません: {}", msg),
            TransformError::InvalidScaleFactor(msg) => write!(f, "不正なスケール倍率です: {}", msg),
            TransformError::InvalidRotation(msg) => write!(f, "不正な回転パラメータです: {}", msg),
        }
    }
}

impl std::error::Error for TransformError {}
