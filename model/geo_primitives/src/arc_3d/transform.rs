//! Arc3D の相似変換
//!
//! 角度は円弧平面の開始方向から測るため、フレームを変換すれば角度は変わらない。

use crate::Arc3D;
use crate::SimilarityTransform3D;
use geo_contracts::{Scalar, SimilarityTransformable3D, TransformError};

impl<T: Scalar> SimilarityTransformable3D<T> for Arc3D<T> {
    fn transform_similarity(
        &self,
        transform: &SimilarityTransform3D<T>,
    ) -> Result<Self, TransformError> {
        Self::new(
            transform.apply_point(&self.center),
            self.radius * transform.scale_factor(),
            transform.apply_direction(self.normal),
            transform.apply_direction(self.start_dir),
            self.start_angle,
            self.end_angle,
        )
        .ok_or_else(|| TransformError::InvalidGeometry("invalid arc".to_string()))
    }
}
