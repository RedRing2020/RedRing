//! Triangle2D の相似変換

use crate::Triangle2D;
use geo_contracts::{Scalar, SimilarityTransform2DCore, SimilarityTransformable2D, TransformError};

impl<T: Scalar> SimilarityTransformable2D<T> for Triangle2D<T> {
    fn transform_similarity<X: SimilarityTransform2DCore<T>>(
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
