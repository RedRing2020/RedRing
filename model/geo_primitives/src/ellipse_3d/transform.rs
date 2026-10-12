//! Ellipse3D の相似変換

use crate::Ellipse3D;
use crate::SimilarityTransform3D;
use geo_contracts::{Scalar, SimilarityTransformable3D, TransformError};

impl<T: Scalar> SimilarityTransformable3D<T> for Ellipse3D<T> {
    fn transform_similarity(
        &self,
        transform: &SimilarityTransform3D<T>,
    ) -> Result<Self, TransformError> {
        let scale = transform.scale_factor();
        Self::new(
            transform.apply_point(self.center),
            self.semi_major_axis * scale,
            self.semi_minor_axis * scale,
            transform.apply_direction(self.normal()).as_vector(),
            transform
                .apply_direction(self.major_axis_direction())
                .as_vector(),
        )
        .ok_or_else(|| TransformError::InvalidGeometry("invalid ellipse".to_string()))
    }
}
