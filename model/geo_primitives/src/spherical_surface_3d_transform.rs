//! SphericalSurface3D の相似変換

use crate::SphericalSurface3D;
use geo_contracts::{Scalar, SimilarityTransform3DCore, SimilarityTransformable3D, TransformError};

impl<T: Scalar> SimilarityTransformable3D<T> for SphericalSurface3D<T> {
    fn transform_similarity<X: SimilarityTransform3DCore<T>>(
        &self,
        transform: &X,
    ) -> Result<Self, TransformError> {
        let scale = transform.scale_factor();
        Self::new(
            self.center_internal().transform_similarity(transform)?,
            self.axis_internal()
                .transform_similarity(transform)?
                .as_vector(),
            self.ref_direction_internal()
                .transform_similarity(transform)?
                .as_vector(),
            self.radius_internal() * scale,
        )
        .ok_or_else(|| TransformError::InvalidGeometry("invalid sphere".to_string()))
    }
}
