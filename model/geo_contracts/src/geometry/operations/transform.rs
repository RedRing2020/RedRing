//! 変換の trait定義
//!
//! 形状は、相似変換の型（`geo_core` の `SimilarityTransform2D` / `SimilarityTransform3D`）に対する変換の trait定義を実装する。
//! 値型（点・ベクトル・方向・AABB）の変換は、相似変換の型の固有メソッド（`apply_point` 等）で行う。
//!
//! 相似変換（回転・平行移動・正の一様スケール）のみを扱い、非一様スケール・ミラーリング・射影は扱わない。

use crate::Scalar;
use geo_core::{SimilarityTransform2D, SimilarityTransform3D, TransformError};

/// 3D 相似変換を受け付ける形状
pub trait SimilarityTransformable3D<T: Scalar>: Sized {
    /// 相似変換を適用した形状を返す
    fn transform_similarity(
        &self,
        transform: &SimilarityTransform3D<T>,
    ) -> Result<Self, TransformError>;
}

/// 2D 相似変換を受け付ける形状
pub trait SimilarityTransformable2D<T: Scalar>: Sized {
    /// 相似変換を適用した形状を返す
    fn transform_similarity(
        &self,
        transform: &SimilarityTransform2D<T>,
    ) -> Result<Self, TransformError>;
}
