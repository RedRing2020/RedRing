//! Rect3D の相似変換

use crate::Rect3D;
use crate::SimilarityTransform3D;
use geo_contracts::{Rect3DConstructor, Scalar, SimilarityTransformable3D, TransformError};

impl<T: Scalar> SimilarityTransformable3D<T> for Rect3D<T> {
    fn transform_similarity(
        &self,
        transform: &SimilarityTransform3D<T>,
    ) -> Result<Self, TransformError> {
        let scale = transform.scale_factor();
        Self::new(
            transform.apply_point(self.origin),
            transform.apply_direction(self.u_axis),
            transform.apply_direction(self.v_axis),
            self.width * scale,
            self.height * scale,
        )
        .ok_or_else(|| TransformError::InvalidGeometry("invalid rectangle".to_string()))
    }
}
