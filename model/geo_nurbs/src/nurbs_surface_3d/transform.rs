//! `NurbsSurface3D` の相似変換

use crate::NurbsSurface3D;
use geo_contracts::{Scalar, SimilarityTransformable3D, TransformError};
use geo_core::SimilarityTransform3D;

/// 制御点を点として変換する。重みは相似変換で変わらないため保持する。
impl<T: Scalar> SimilarityTransformable3D<T> for NurbsSurface3D<T> {
    fn transform_similarity(
        &self,
        transform: &SimilarityTransform3D<T>,
    ) -> Result<Self, TransformError> {
        Ok(self.map_control_points(|p| transform.apply_point(p)))
    }
}
