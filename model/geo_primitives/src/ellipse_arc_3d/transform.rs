//! EllipseArc3D の相似変換
//!
//! 角度は基底楕円の局所パラメータのため、基底楕円を変換すれば角度は変わらない。

use crate::EllipseArc3D;
use crate::SimilarityTransform3D;
use geo_contracts::{Scalar, SimilarityTransformable3D, TransformError};

impl<T: Scalar> SimilarityTransformable3D<T> for EllipseArc3D<T> {
    fn transform_similarity(
        &self,
        transform: &SimilarityTransform3D<T>,
    ) -> Result<Self, TransformError> {
        Ok(Self::new(
            self.ellipse.transform_similarity(transform)?,
            self.start_angle,
            self.end_angle,
        ))
    }
}
