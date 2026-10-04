//! Circle3D の相似変換

use crate::Circle3D;
use geo_contracts::{Scalar, SimilarityTransform3DCore, SimilarityTransformable3D, TransformError};

impl<T: Scalar> SimilarityTransformable3D<T> for Circle3D<T> {
    fn transform_similarity<X: SimilarityTransform3DCore<T>>(
        &self,
        transform: &X,
    ) -> Result<Self, TransformError> {
        Self::new_with_ref_direction(
            self.center_internal().transform_similarity(transform)?,
            self.axis_internal().transform_similarity(transform)?,
            self.ref_direction_internal()
                .transform_similarity(transform)?,
            self.radius_internal() * transform.scale_factor(),
        )
        .ok_or_else(|| TransformError::InvalidGeometry("invalid circle".to_string()))
    }
}
