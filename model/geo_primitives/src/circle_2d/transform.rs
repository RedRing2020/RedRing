//! Circle2D の相似変換

use crate::Circle2D;
use crate::SimilarityTransform2D;
use geo_contracts::{Scalar, SimilarityTransformable2D, TransformError};

impl<T: Scalar> SimilarityTransformable2D<T> for Circle2D<T> {
    fn transform_similarity(
        &self,
        transform: &SimilarityTransform2D<T>,
    ) -> Result<Self, TransformError> {
        Self::new_with_ref_direction(
            transform.apply_point(&self.center),
            self.radius * transform.scale_factor(),
            transform.apply_direction(self.ref_direction),
        )
        .ok_or_else(|| TransformError::InvalidGeometry("invalid circle".to_string()))
    }
}
