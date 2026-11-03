//! Circle3D の相似変換

use crate::Circle3D;
use crate::SimilarityTransform3D;
use geo_contracts::{Scalar, SimilarityTransformable3D, TransformError};

impl<T: Scalar> SimilarityTransformable3D<T> for Circle3D<T> {
    fn transform_similarity(
        &self,
        transform: &SimilarityTransform3D<T>,
    ) -> Result<Self, TransformError> {
        Self::new_with_ref_direction(
            transform.apply_point(self.center),
            transform.apply_direction(self.axis),
            transform.apply_direction(self.ref_direction),
            self.radius * transform.scale_factor(),
        )
        .ok_or_else(|| TransformError::InvalidGeometry("invalid circle".to_string()))
    }
}
