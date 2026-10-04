//! Arc3D の相似変換
//!
//! 角度は円弧平面の開始方向から測るため、フレームを変換すれば角度は変わらない。
//! 設計: TRANSFORM_TRAIT_DESIGN.md

use crate::Arc3D;
use geo_contracts::{Scalar, SimilarityTransform3DCore, SimilarityTransformable3D, TransformError};

impl<T: Scalar> SimilarityTransformable3D<T> for Arc3D<T> {
    fn transform_similarity<X: SimilarityTransform3DCore<T>>(
        &self,
        transform: &X,
    ) -> Result<Self, TransformError> {
        Self::new(
            self.center.transform_similarity(transform)?,
            self.radius * transform.scale_factor(),
            self.normal.transform_similarity(transform)?,
            self.start_dir.transform_similarity(transform)?,
            self.start_angle,
            self.end_angle,
        )
        .ok_or_else(|| TransformError::InvalidGeometry("invalid arc".to_string()))
    }
}
