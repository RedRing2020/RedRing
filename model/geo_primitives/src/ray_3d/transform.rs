//! Ray3D の相似変換

use crate::Ray3D;
use geo_contracts::{Scalar, SimilarityTransform3DCore, SimilarityTransformable3D, TransformError};

impl<T: Scalar> SimilarityTransformable3D<T> for Ray3D<T> {
    fn transform_similarity<X: SimilarityTransform3DCore<T>>(
        &self,
        transform: &X,
    ) -> Result<Self, TransformError> {
        Ok(Self {
            origin: self.origin.transform_similarity(transform)?,
            direction: self.direction.transform_similarity(transform)?,
        })
    }
}
