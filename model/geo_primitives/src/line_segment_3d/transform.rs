//! LineSegment3D の相似変換
//!
//! support line と端点を変換し、support line 上のパラメータはスケール係数倍にする
//! （方向が単位ベクトルのため、パラメータは support line 上の距離）。

use crate::LineSegment3D;
use crate::SimilarityTransform3D;
use geo_contracts::{Scalar, SimilarityTransformable3D, TransformError};

impl<T: Scalar> SimilarityTransformable3D<T> for LineSegment3D<T> {
    fn transform_similarity(
        &self,
        transform: &SimilarityTransform3D<T>,
    ) -> Result<Self, TransformError> {
        let scale = transform.scale_factor();
        Ok(Self {
            line: self.line.transform_similarity(transform)?,
            start_param: self.start_param * scale,
            end_param: self.end_param * scale,
            start_point: transform.apply_point(self.start_point),
            end_point: transform.apply_point(self.end_point),
        })
    }
}
