//! Arc2D の相似変換
//!
//! 2D の円弧の角度は基底円の参照方向ではなくグローバル X 軸から測るため、
//! 回転角を開始角・終了角に加える。

use crate::Arc2D;
use crate::{SimilarityTransform2D, Vector2D};
use geo_contracts::{Angle, Scalar, SimilarityTransformable2D, TransformError};

impl<T: Scalar> SimilarityTransformable2D<T> for Arc2D<T> {
    fn transform_similarity(
        &self,
        transform: &SimilarityTransform2D<T>,
    ) -> Result<Self, TransformError> {
        let unit_x = transform.apply_vector(Vector2D::unit_x());
        let (x, y) = (unit_x.x(), unit_x.y());
        let rotation = Angle::from_radians(y.atan2(x));
        // 開始角を [0, 2π) に正規化し、掃引角は変えない（範囲判定が開始角基準のため）
        let start_angle = (self.start_angle + rotation).normalize();
        let end_angle = start_angle + (self.end_angle - self.start_angle);
        Ok(Self {
            circle: self.circle.transform_similarity(transform)?,
            start_angle,
            end_angle,
        })
    }
}
