//! Transform エラー型定義

use std::fmt;

/// Transform 操作で発生する可能性のあるエラー
#[derive(Debug, Clone, PartialEq)]
pub enum TransformError {
    /// 変換後の幾何構造体が無効（例：ゼロスケール）
    InvalidGeometry,
    /// ゼロベクトル（正規化不可能）
    ZeroVector,
    /// 不正なスケール倍率
    InvalidScaleFactor,
    /// 不正な回転パラメータ
    InvalidRotation,
}

impl fmt::Display for TransformError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TransformError::InvalidGeometry => write!(f, "変換後の幾何構造体が無効です"),
            TransformError::ZeroVector => write!(f, "ゼロベクトルは正規化できません"),
            TransformError::InvalidScaleFactor => write!(f, "不正なスケール倍率です"),
            TransformError::InvalidRotation => write!(f, "不正な回転パラメータです"),
        }
    }
}

impl std::error::Error for TransformError {}
