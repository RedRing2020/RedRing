//! EllipseArc2D の相似変換
//!
//! 角度は基底楕円の局所パラメータのため、基底楕円を変換すれば角度は変わらない。

use crate::EllipseArc2D;
use geo_contracts::{Scalar, SimilarityTransform2DCore, SimilarityTransformable2D, TransformError};

impl<T: Scalar> SimilarityTransformable2D<T> for EllipseArc2D<T> {
    fn transform_similarity<X: SimilarityTransform2DCore<T>>(
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
