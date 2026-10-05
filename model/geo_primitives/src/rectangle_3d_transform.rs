//! Rect3D の相似変換

use crate::Rect3D;
use geo_contracts::{Scalar, SimilarityTransform3DCore, SimilarityTransformable3D, TransformError};

impl<T: Scalar> SimilarityTransformable3D<T> for Rect3D<T> {
    fn transform_similarity<X: SimilarityTransform3DCore<T>>(
        &self,
        transform: &X,
    ) -> Result<Self, TransformError> {
        let scale = transform.scale_factor();
        Self::new(
            self.origin_point().transform_similarity(transform)?,
            self.u_axis_dir()
                .transform_similarity(transform)?
                .as_vector(),
            self.v_axis_dir()
                .transform_similarity(transform)?
                .as_vector(),
            self.width_value() * scale,
            self.height_value() * scale,
        )
        .ok_or_else(|| TransformError::InvalidGeometry("invalid rectangle".to_string()))
    }
}
