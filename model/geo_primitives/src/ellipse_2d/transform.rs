//! Ellipse2D の相似変換

use crate::Ellipse2D;
use crate::SimilarityTransform2D;
use geo_contracts::{
    Angle, Ellipse2DConstructor, Scalar, SimilarityTransformable2D, TransformError,
};

/// 回転は変換後の長軸方向から求め直す（`rotation` はグローバル X 軸からの角度のため）。
impl<T: Scalar> SimilarityTransformable2D<T> for Ellipse2D<T> {
    fn transform_similarity(
        &self,
        transform: &SimilarityTransform2D<T>,
    ) -> Result<Self, TransformError> {
        let major = self.major_axis_direction();
        let rotated = transform.apply_vector(major);
        let (x, y) = (rotated.x(), rotated.y());
        let scale = transform.scale_factor();
        <Self as Ellipse2DConstructor<T>>::new(
            transform.apply_point(self.center),
            self.semi_major * scale,
            self.semi_minor * scale,
            Angle::from_radians(y.atan2(x)),
        )
        .ok_or_else(|| TransformError::InvalidGeometry("invalid ellipse".to_string()))
    }
}
