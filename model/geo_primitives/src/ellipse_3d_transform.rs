//! Ellipse3D の相似変換
//!
//! 設計: TRANSFORM_TRAIT_DESIGN.md

use crate::Ellipse3D;
use geo_contracts::{Scalar, SimilarityTransform3DCore, SimilarityTransformable3D, TransformError};

impl<T: Scalar> SimilarityTransformable3D<T> for Ellipse3D<T> {
    fn transform_similarity<X: SimilarityTransform3DCore<T>>(
        &self,
        transform: &X,
    ) -> Result<Self, TransformError> {
        let scale = transform.scale_factor();
        Self::new(
            self.center_internal().transform_similarity(transform)?,
            self.semi_major_internal() * scale,
            self.semi_minor_internal() * scale,
            self.normal().transform_similarity(transform)?.as_vector(),
            self.major_axis_direction()
                .transform_similarity(transform)?
                .as_vector(),
        )
        .ok_or_else(|| TransformError::InvalidGeometry("invalid ellipse".to_string()))
    }
}
