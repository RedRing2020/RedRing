//! TorusSurface3D の相似変換

use crate::TorusSurface3D;
use geo_contracts::{Scalar, SimilarityTransform3DCore, SimilarityTransformable3D, TransformError};

impl<T: Scalar> SimilarityTransformable3D<T> for TorusSurface3D<T> {
    fn transform_similarity<X: SimilarityTransform3DCore<T>>(
        &self,
        transform: &X,
    ) -> Result<Self, TransformError> {
        let scale = transform.scale_factor();
        Self::new(
            self.origin_internal().transform_similarity(transform)?,
            self.z_axis_internal().transform_similarity(transform)?,
            self.x_axis_internal().transform_similarity(transform)?,
            self.major_radius_internal() * scale,
            self.minor_radius_internal() * scale,
        )
        .ok_or_else(|| TransformError::InvalidGeometry("invalid torus".to_string()))
    }
}
