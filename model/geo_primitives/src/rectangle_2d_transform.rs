//! Rect2D の相似変換

use crate::Rect2D;
use geo_contracts::{Scalar, SimilarityTransform2DCore, SimilarityTransformable2D, TransformError};

/// 軸平行の矩形のため、平行移動・一様スケールのみ受け付ける。回転を含む場合は `Unsupported`。
impl<T: Scalar> SimilarityTransformable2D<T> for Rect2D<T> {
    fn transform_similarity<X: SimilarityTransform2DCore<T>>(
        &self,
        transform: &X,
    ) -> Result<Self, TransformError> {
        if transform.has_rotation() {
            return Err(TransformError::Unsupported(
                "rotation of an axis-aligned rectangle".to_string(),
            ));
        }
        let scale = transform.scale_factor();
        Self::new(
            self.origin_point().transform_similarity(transform)?,
            self.width_value() * scale,
            self.height_value() * scale,
        )
        .ok_or_else(|| TransformError::InvalidGeometry("invalid rectangle".to_string()))
    }
}
