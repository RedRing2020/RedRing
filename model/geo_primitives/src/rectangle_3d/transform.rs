//! Rect3D の相似変換

use crate::Rect3D;
use crate::SimilarityTransform3D;
use geo_contracts::{Scalar, SimilarityTransformable3D, TransformError};

impl<T: Scalar> SimilarityTransformable3D<T> for Rect3D<T> {
    fn transform_similarity(
        &self,
        transform: &SimilarityTransform3D<T>,
    ) -> Result<Self, TransformError> {
        let scale = transform.scale_factor();
        Self::new(
            transform.apply_point(&self.origin_point()),
            transform.apply_direction(self.u_axis_dir()).as_vector(),
            transform.apply_direction(self.v_axis_dir()).as_vector(),
            self.width_value() * scale,
            self.height_value() * scale,
        )
        .ok_or_else(|| TransformError::InvalidGeometry("invalid rectangle".to_string()))
    }
}
