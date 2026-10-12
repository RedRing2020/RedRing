//! InfiniteLine2D の相似変換

use crate::InfiniteLine2D;
use crate::SimilarityTransform2D;
use geo_contracts::{Scalar, SimilarityTransformable2D, TransformError};

impl<T: Scalar> SimilarityTransformable2D<T> for InfiniteLine2D<T> {
    fn transform_similarity(
        &self,
        transform: &SimilarityTransform2D<T>,
    ) -> Result<Self, TransformError> {
        Ok(Self {
            point: transform.apply_point(&self.point),
            direction: transform.apply_direction(self.direction),
        })
    }
}
