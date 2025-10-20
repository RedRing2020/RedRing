//! Transform操作のエラー型定義
//!
//! 幾何変換操作で発生する可能性のあるエラーを定義します。

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

/// 安全なTransform操作のトレイト
///
/// 失敗の可能性がある変換操作をResult型で表現します。
pub trait SafeTransform<T: crate::Scalar> {
    /// 安全な平行移動
    fn safe_translate(&self, offset: T) -> Result<Self, TransformError>
    where
        Self: Sized;

    /// 安全なスケール変換  
    fn safe_scale(&self, center: T, factor: T) -> Result<Self, TransformError>
    where
        Self: Sized;

    /// 安全な回転変換
    fn safe_rotate(
        &self,
        center: T,
        axis: T,
        angle: crate::Angle<T>,
    ) -> Result<Self, TransformError>
    where
        Self: Sized;
}
