//! ConicalSolid3D の相似変換

use crate::ConicalSolid3D;
use geo_contracts::{Scalar, SimilarityTransform3DCore, SimilarityTransformable3D, TransformError};

impl<T: Scalar> SimilarityTransformable3D<T> for ConicalSolid3D<T> {
    fn transform_similarity<X: SimilarityTransform3DCore<T>>(
        &self,
        transform: &X,
    ) -> Result<Self, TransformError> {
        let scale = transform.scale_factor();
        Self::new(
            self.center.transform_similarity(transform)?,
            self.axis.transform_similarity(transform)?.as_vector(),
            self.ref_direction
                .transform_similarity(transform)?
                .as_vector(),
            self.radius * scale,
            self.height * scale,
        )
        .ok_or_else(|| TransformError::InvalidGeometry("invalid cone".to_string()))
    }
}
