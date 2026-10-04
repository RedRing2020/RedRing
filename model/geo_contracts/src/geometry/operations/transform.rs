//! 変換の trait定義
//!
//! 変換の種類を型で表し、形状はその型に対する変換 trait を実装する。
//! 変換型の具象型は `geo_core` に置く。
//!
//! 相似変換（回転・平行移動・正の一様スケール）のみを扱い、非一様スケール・ミラーリング・射影は扱わない。

use crate::Scalar;
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
    /// 旧変換 API 用。新しい変換 trait では使用しない
    ZeroVector(String),
    /// 旧変換 API 用。新しい変換 trait では使用しない
    InvalidScaleFactor(String),
    /// 旧変換 API 用。新しい変換 trait では使用しない
    InvalidRotation(String),
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
            TransformError::ZeroVector(msg) => write!(f, "zero vector is not allowed: {msg}"),
            TransformError::InvalidScaleFactor(msg) => write!(f, "invalid scale factor: {msg}"),
            TransformError::InvalidRotation(msg) => write!(f, "invalid rotation: {msg}"),
        }
    }
}

impl std::error::Error for TransformError {}

/// 3D 相似変換（回転・平行移動・正の一様スケール）
pub trait SimilarityTransform3DCore<T: Scalar> {
    /// 点に適用する
    fn apply_point(&self, point: (T, T, T)) -> (T, T, T);

    /// ベクトルに適用する（平行移動を含まない）
    fn apply_vector(&self, vector: (T, T, T)) -> (T, T, T);

    /// 一様スケール係数（正）
    fn scale_factor(&self) -> T;

    /// 回転成分を含むか
    fn has_rotation(&self) -> bool;
}

/// 3D 相似変換を受け付ける形状
pub trait SimilarityTransformable3D<T: Scalar>: Sized {
    /// 相似変換を適用した形状を返す
    fn transform_similarity<X: SimilarityTransform3DCore<T>>(
        &self,
        transform: &X,
    ) -> Result<Self, TransformError>;
}

/// 2D 相似変換（回転・平行移動・正の一様スケール）
pub trait SimilarityTransform2DCore<T: Scalar> {
    /// 点に適用する
    fn apply_point(&self, point: (T, T)) -> (T, T);

    /// ベクトルに適用する（平行移動を含まない）
    fn apply_vector(&self, vector: (T, T)) -> (T, T);

    /// 一様スケール係数（正）
    fn scale_factor(&self) -> T;

    /// 回転成分を含むか
    fn has_rotation(&self) -> bool;
}

/// 2D 相似変換を受け付ける形状
pub trait SimilarityTransformable2D<T: Scalar>: Sized {
    /// 相似変換を適用した形状を返す
    fn transform_similarity<X: SimilarityTransform2DCore<T>>(
        &self,
        transform: &X,
    ) -> Result<Self, TransformError>;
}
