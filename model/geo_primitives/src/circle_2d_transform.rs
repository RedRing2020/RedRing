//! Circle2D の相似変換
//!
//! 設計: TRANSFORM_TRAIT_DESIGN.md

use crate::Circle2D;
use geo_contracts::{Scalar, SimilarityTransform2DCore, SimilarityTransformable2D, TransformError};

impl<T: Scalar> SimilarityTransformable2D<T> for Circle2D<T> {
    fn transform_similarity<X: SimilarityTransform2DCore<T>>(
        &self,
        transform: &X,
    ) -> Result<Self, TransformError> {
        Self::new_with_ref_direction(
            self.center_internal().transform_similarity(transform)?,
            self.radius_internal() * transform.scale_factor(),
            self.ref_direction_internal()
                .transform_similarity(transform)?,
        )
        .ok_or_else(|| TransformError::InvalidGeometry("invalid circle".to_string()))
    }
}
