//! InfiniteLine3D の相似変換
//!
//! 設計: TRANSFORM_TRAIT_DESIGN.md

use crate::InfiniteLine3D;
use geo_contracts::{Scalar, SimilarityTransform3DCore, SimilarityTransformable3D, TransformError};

impl<T: Scalar> SimilarityTransformable3D<T> for InfiniteLine3D<T> {
    fn transform_similarity<X: SimilarityTransform3DCore<T>>(
        &self,
        transform: &X,
    ) -> Result<Self, TransformError> {
        Ok(Self::from_direction(
            self.point.transform_similarity(transform)?,
            self.direction.transform_similarity(transform)?,
        ))
    }
}
