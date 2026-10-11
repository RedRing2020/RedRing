//! ConicalSurface3D の相似変換
//!
//! 半頂角は相似変換で変わらないため保持する。

use crate::ConicalSurface3D;
use geo_contracts::{Scalar, SimilarityTransform3DCore, SimilarityTransformable3D, TransformError};

impl<T: Scalar> SimilarityTransformable3D<T> for ConicalSurface3D<T> {
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
            self.semi_angle,
        )
        .ok_or_else(|| TransformError::InvalidGeometry("invalid cone".to_string()))
    }
}
