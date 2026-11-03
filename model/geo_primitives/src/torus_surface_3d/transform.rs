//! TorusSurface3D の相似変換

use crate::SimilarityTransform3D;
use crate::TorusSurface3D;
use geo_contracts::{Scalar, SimilarityTransformable3D, TorusSurface3DConstructor, TransformError};

impl<T: Scalar> SimilarityTransformable3D<T> for TorusSurface3D<T> {
    fn transform_similarity(
        &self,
        transform: &SimilarityTransform3D<T>,
    ) -> Result<Self, TransformError> {
        let scale = transform.scale_factor();
        Self::new(
            transform.apply_point(self.origin),
            transform.apply_direction(self.z_axis),
            transform.apply_direction(self.x_axis),
            self.major_radius * scale,
            self.minor_radius * scale,
        )
        .ok_or_else(|| TransformError::InvalidGeometry("invalid torus".to_string()))
    }
}
