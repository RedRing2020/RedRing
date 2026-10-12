//! Triangle2D の相似変換

use crate::SimilarityTransform2D;
use crate::Triangle2D;
use geo_contracts::{Scalar, SimilarityTransformable2D, TransformError};

impl<T: Scalar> SimilarityTransformable2D<T> for Triangle2D<T> {
    fn transform_similarity(
        &self,
        transform: &SimilarityTransform2D<T>,
    ) -> Result<Self, TransformError> {
        Self::new(
            transform.apply_point(&self.vertex_a),
            transform.apply_point(&self.vertex_b),
            transform.apply_point(&self.vertex_c),
        )
        .ok_or_else(|| TransformError::InvalidGeometry("degenerate triangle".to_string()))
    }
}
