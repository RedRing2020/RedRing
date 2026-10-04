//! Triangle3D の相似変換
//!
//! 設計: TRANSFORM_TRAIT_DESIGN.md

use crate::Triangle3D;
use geo_contracts::{Scalar, SimilarityTransform3DCore, SimilarityTransformable3D, TransformError};

impl<T: Scalar> SimilarityTransformable3D<T> for Triangle3D<T> {
    fn transform_similarity<X: SimilarityTransform3DCore<T>>(
        &self,
        transform: &X,
    ) -> Result<Self, TransformError> {
        Self::new(
            self.vertex_a_internal().transform_similarity(transform)?,
            self.vertex_b_internal().transform_similarity(transform)?,
            self.vertex_c_internal().transform_similarity(transform)?,
        )
        .ok_or_else(|| TransformError::InvalidGeometry("degenerate triangle".to_string()))
    }
}
