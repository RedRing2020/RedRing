//! EllipseArc3D の相似変換
//!
//! 角度は基底楕円の局所パラメータのため、基底楕円を変換すれば角度は変わらない。
//! 設計: TRANSFORM_TRAIT_DESIGN.md

use crate::EllipseArc3D;
use geo_contracts::{Scalar, SimilarityTransform3DCore, SimilarityTransformable3D, TransformError};

impl<T: Scalar> SimilarityTransformable3D<T> for EllipseArc3D<T> {
    fn transform_similarity<X: SimilarityTransform3DCore<T>>(
        &self,
        transform: &X,
    ) -> Result<Self, TransformError> {
        Ok(Self::new(
            self.ellipse.transform_similarity(transform)?,
            self.start_angle,
            self.end_angle,
        ))
    }
}
