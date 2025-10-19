//! LineSegment2D Transform実装
//!
//! geo_foundation::extensions::BasicTransformトレイトの実装
//! Core/Extension分離パターンに従った変換機能の実装

use crate::{LineSegment2D, Point2D, Vector2D};
use geo_foundation::{extensions::BasicTransform, Angle, Scalar};

// ============================================================================
// BasicTransform implementation (geo_foundation統合)
// ============================================================================

impl<T: Scalar> BasicTransform<T> for LineSegment2D<T> {
    type Transformed = LineSegment2D<T>;
    type Vector2D = Vector2D<T>;
    type Point2D = Point2D<T>;
    type Angle = Angle<T>;

    /// 平行移動
    fn translate(&self, translation: Self::Vector2D) -> Self::Transformed {
        let new_start = self.start() + translation;
        let new_end = self.end() + translation;
        Self::new(new_start, new_end).unwrap()
    }

    /// 回転
    fn rotate(&self, center: Self::Point2D, angle: Self::Angle) -> Self::Transformed {
        let cos_angle = angle.cos();
        let sin_angle = angle.sin();

        // 始点を回転
        let start_relative = self.start() - center;
        let rotated_start = Point2D::new(
            start_relative.x() * cos_angle - start_relative.y() * sin_angle,
            start_relative.x() * sin_angle + start_relative.y() * cos_angle,
        ) + center.to_vector();

        // 終点を回転
        let end_relative = self.end() - center;
        let rotated_end = Point2D::new(
            end_relative.x() * cos_angle - end_relative.y() * sin_angle,
            end_relative.x() * sin_angle + end_relative.y() * cos_angle,
        ) + center.to_vector();

        Self::new(rotated_start, rotated_end).unwrap()
    }

    /// スケール
    fn scale(&self, center: Self::Point2D, scale_factor: T) -> Self::Transformed {
        let scaled_start = center + (self.start() - center) * scale_factor;
        let scaled_end = center + (self.end() - center) * scale_factor;
        Self::new(scaled_start, scaled_end).unwrap()
    }
}
