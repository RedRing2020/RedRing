//! SphericalSolid3D の相似変換

use crate::SimilarityTransform3D;
use crate::SphericalSolid3D;
use geo_contracts::{
    Scalar, SimilarityTransformable3D, SphericalSolid3DConstructor, TransformError,
};

impl<T: Scalar> SimilarityTransformable3D<T> for SphericalSolid3D<T> {
    fn transform_similarity(
        &self,
        transform: &SimilarityTransform3D<T>,
    ) -> Result<Self, TransformError> {
        let scale = transform.scale_factor();
        Self::new(
            transform.apply_point(self.center),
            transform.apply_direction(self.axis),
            transform.apply_direction(self.ref_direction),
            self.radius * scale,
        )
        .ok_or_else(|| TransformError::InvalidGeometry("invalid sphere".to_string()))
    }
}
