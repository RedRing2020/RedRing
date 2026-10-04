//! LineSegment2D の相似変換
//!
//! support line と端点を変換し、support line 上のパラメータはスケール係数倍にする
//! （方向が単位ベクトルのため、パラメータは support line 上の距離）。

use crate::LineSegment2D;
use geo_contracts::{Scalar, SimilarityTransform2DCore, SimilarityTransformable2D, TransformError};

impl<T: Scalar> SimilarityTransformable2D<T> for LineSegment2D<T> {
    fn transform_similarity<X: SimilarityTransform2DCore<T>>(
        &self,
        transform: &X,
    ) -> Result<Self, TransformError> {
        let scale = transform.scale_factor();
        Ok(Self {
            line: self.line.transform_similarity(transform)?,
            start_param: self.start_param * scale,
            end_param: self.end_param * scale,
            start_point: self.start_point.transform_similarity(transform)?,
            end_point: self.end_point.transform_similarity(transform)?,
        })
    }
}
