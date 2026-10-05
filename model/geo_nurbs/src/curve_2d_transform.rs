//! `NurbsCurve2D` の相似変換

use crate::curve_2d::NurbsCurve2D;
use geo_contracts::{Scalar, SimilarityTransform2DCore, SimilarityTransformable2D, TransformError};

/// 制御点を点として変換する。重みは相似変換で変わらないため保持する。
impl<T: Scalar> SimilarityTransformable2D<T> for NurbsCurve2D<T> {
    fn transform_similarity<X: SimilarityTransform2DCore<T>>(
        &self,
        transform: &X,
    ) -> Result<Self, TransformError> {
        Ok(self.map_control_points(|p| transform.apply_point(p)))
    }
}
