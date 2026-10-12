//! Triangle3D の相似変換

use crate::SimilarityTransform3D;
use crate::Triangle3D;
use geo_contracts::{Scalar, SimilarityTransformable3D, TransformError};

impl<T: Scalar> SimilarityTransformable3D<T> for Triangle3D<T> {
    fn transform_similarity(
        &self,
        transform: &SimilarityTransform3D<T>,
    ) -> Result<Self, TransformError> {
        Self::new(
            transform.apply_point(self.vertex_a),
            transform.apply_point(self.vertex_b),
            transform.apply_point(self.vertex_c),
        )
        .ok_or_else(|| TransformError::InvalidGeometry("degenerate triangle".to_string()))
    }
}
