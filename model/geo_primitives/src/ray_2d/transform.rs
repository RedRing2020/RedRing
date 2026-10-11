//! Ray2D の相似変換

use crate::Ray2D;
use geo_contracts::{Scalar, SimilarityTransform2DCore, SimilarityTransformable2D, TransformError};

impl<T: Scalar> SimilarityTransformable2D<T> for Ray2D<T> {
    fn transform_similarity<X: SimilarityTransform2DCore<T>>(
        &self,
        transform: &X,
    ) -> Result<Self, TransformError> {
        Ok(Self {
            origin: self.origin.transform_similarity(transform)?,
            direction: self.direction.transform_similarity(transform)?,
        })
    }
}
