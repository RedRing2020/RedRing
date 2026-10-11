//! TorusSolid3D の相似変換

use crate::TorusSolid3D;
use geo_contracts::{Scalar, SimilarityTransform3DCore, SimilarityTransformable3D, TransformError};

impl<T: Scalar> SimilarityTransformable3D<T> for TorusSolid3D<T> {
    fn transform_similarity<X: SimilarityTransform3DCore<T>>(
        &self,
        transform: &X,
    ) -> Result<Self, TransformError> {
        let scale = transform.scale_factor();
        Self::new(
            self.origin.transform_similarity(transform)?,
            self.z_axis.transform_similarity(transform)?,
            self.x_axis.transform_similarity(transform)?,
            self.major_radius * scale,
            self.minor_radius * scale,
        )
        .ok_or_else(|| TransformError::InvalidGeometry("invalid torus".to_string()))
    }
}
