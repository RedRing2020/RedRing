//! Ray3D の相似変換

use crate::Ray3D;
use crate::SimilarityTransform3D;
use geo_contracts::{Scalar, SimilarityTransformable3D, TransformError};

impl<T: Scalar> SimilarityTransformable3D<T> for Ray3D<T> {
    fn transform_similarity(
        &self,
        transform: &SimilarityTransform3D<T>,
    ) -> Result<Self, TransformError> {
        Ok(Self {
            origin: transform.apply_point(self.origin),
            direction: transform.apply_direction(self.direction),
        })
    }
}
