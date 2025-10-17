//! ベクトルの基本トレイト
//!
//! ベクトルの基本的な属性アクセスとデータ構造に直接関連する計算

use super::foundation::{BasicMetrics, GeometryFoundation};
use crate::Scalar;

// =============================================================================
// ベクトル (Vector)
// =============================================================================

/// ベクトル正規化エラー
#[derive(Debug, Clone, PartialEq)]
pub enum VectorNormalizationError {
    ZeroLength,
    NumericalInstability,
}

impl std::fmt::Display for VectorNormalizationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VectorNormalizationError::ZeroLength => {
                write!(f, "Cannot normalize zero-length vector")
            }
            VectorNormalizationError::NumericalInstability => {
                write!(f, "Numerical instability in vector normalization")
            }
        }
    }
}

impl std::error::Error for VectorNormalizationError {}

/// ベクトルの基本トレイト
pub trait VectorCore<T: Scalar>: GeometryFoundation<T> {
    /// ベクトルの成分を取得
    fn components(&self) -> Vec<T>;

    /// ベクトルの長さ（ノルム）を取得
    fn magnitude(&self) -> T;

    /// ベクトルの長さの二乗を取得（平方根計算の回避）
    fn magnitude_squared(&self) -> T;

    /// ゼロベクトルかどうかを判定
    fn is_zero(&self) -> bool {
        self.magnitude_squared() == T::ZERO
    }

    /// 単位ベクトル化（Result型で安全性を保証）
    fn normalize(&self) -> Result<Self, VectorNormalizationError>
    where
        Self: Sized;

    /// 他のベクトルとの内積を計算
    fn dot(&self, other: &Self) -> T;
}

/// VectorCoreに対するBasicMetricsのデフォルト実装
impl<T: Scalar, V: VectorCore<T>> BasicMetrics<T> for V {
    fn length(&self) -> Option<T> {
        Some(self.magnitude())
    }
}

/// 2Dベクトルの基本トレイト
pub trait Vector2DCore<T: Scalar>: VectorCore<T> {
    /// X成分を取得
    fn x(&self) -> T;

    /// Y成分を取得
    fn y(&self) -> T;

    /// 2D外積（スカラー値）を計算
    fn cross_2d(&self, other: &Self) -> T {
        self.x() * other.y() - self.y() * other.x()
    }

    /// 垂直ベクトルを取得（90度回転）
    fn perpendicular(&self) -> Self;

    /// ベクトルの角度を取得（ラジアン）
    fn angle(&self) -> T {
        self.y().atan2(self.x())
    }
}

/// 3Dベクトルの基本トレイト
pub trait Vector3DCore<T: Scalar>: Vector2DCore<T> {
    /// Z成分を取得
    fn z(&self) -> T;

    /// 3D外積を計算
    fn cross_3d(&self, other: &Self) -> Self;

    /// 混合積（スカラー三重積）を計算
    fn scalar_triple_product(&self, b: &Self, c: &Self) -> T
    where
        Self: Sized,
    {
        let cross_result = b.cross_3d(c);
        self.dot(&cross_result)
    }
}
