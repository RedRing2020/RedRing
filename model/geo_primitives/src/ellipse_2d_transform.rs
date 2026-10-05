//! Ellipse2D の相似変換

use crate::Ellipse2D;
use geo_contracts::{Scalar, SimilarityTransform2DCore, SimilarityTransformable2D, TransformError};

/// 回転は変換後の長軸方向から求め直す（`rotation` はグローバル X 軸からの角度のため）。
impl<T: Scalar> SimilarityTransformable2D<T> for Ellipse2D<T> {
    fn transform_similarity<X: SimilarityTransform2DCore<T>>(
        &self,
        transform: &X,
    ) -> Result<Self, TransformError> {
        let major = self.major_axis_direction();
        let (x, y) = transform.apply_vector((major.x(), major.y()));
        let scale = transform.scale_factor();
        Self::new(
            self.center_internal().transform_similarity(transform)?,
            self.semi_major_internal() * scale,
            self.semi_minor_internal() * scale,
            y.atan2(x),
        )
        .ok_or_else(|| TransformError::InvalidGeometry("invalid ellipse".to_string()))
    }
}
