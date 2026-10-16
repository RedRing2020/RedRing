//! `NurbsCurve2D` の相似変換

use crate::NurbsCurve2D;
use geo_contracts::{Scalar, SimilarityTransformable2D, TransformError};
use geo_core::SimilarityTransform2D;

/// 制御点を点として変換する。重みは相似変換で変わらないため保持する。
impl<T: Scalar> SimilarityTransformable2D<T> for NurbsCurve2D<T> {
    fn transform_similarity(
        &self,
        transform: &SimilarityTransform2D<T>,
    ) -> Result<Self, TransformError> {
        Ok(self.map_control_points(|p| transform.apply_point(p)))
    }
}
