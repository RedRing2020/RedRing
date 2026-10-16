//! ConicalSolid3D の相似変換

use crate::ConicalSolid3D;
use crate::SimilarityTransform3D;
use geo_contracts::{ConicalSolid3DConstructor, Scalar, SimilarityTransformable3D, TransformError};

impl<T: Scalar> SimilarityTransformable3D<T> for ConicalSolid3D<T> {
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
            self.height * scale,
        )
        .ok_or_else(|| TransformError::InvalidGeometry("invalid cone".to_string()))
    }
}
