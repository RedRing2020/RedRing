//! InfiniteLine2D Transform実装
//!
//! geo_foundation::extensions::BasicTransformトレイトの実装
//! Core/Extension分離パターンに従った変換機能の実装

use crate::{InfiniteLine2D, Point2D, Vector2D};
use geo_foundation::{extensions::BasicTransform, Angle, Scalar};

// ============================================================================
// BasicTransform implementation (geo_foundation統合)
// ============================================================================

impl<T: Scalar> BasicTransform<T> for InfiniteLine2D<T> {
    type Transformed = InfiniteLine2D<T>;
    type Vector2D = Vector2D<T>;
    type Point2D = Point2D<T>;
    type Angle = Angle<T>;

    /// 平行移動
    fn translate(&self, translation: Self::Vector2D) -> Self::Transformed {
        let new_point = self.point() + translation;
        Self::new(new_point, self.direction().as_vector()).unwrap()
    }

    /// 回転
    fn rotate(&self, center: Self::Point2D, angle: Self::Angle) -> Self::Transformed {
        let cos_angle = angle.cos();
        let sin_angle = angle.sin();

        // 直線上の点を回転
        let relative_point = self.point() - center;
        let rotated_point_x = relative_point.x() * cos_angle - relative_point.y() * sin_angle;
        let rotated_point_y = relative_point.x() * sin_angle + relative_point.y() * cos_angle;
        let new_point = center + Vector2D::new(rotated_point_x, rotated_point_y);

        // 方向ベクトルを回転
        let dir = self.direction().as_vector();
        let rotated_dir_x = dir.x() * cos_angle - dir.y() * sin_angle;
        let rotated_dir_y = dir.x() * sin_angle + dir.y() * cos_angle;
        let new_direction = Vector2D::new(rotated_dir_x, rotated_dir_y);

        Self::new(new_point, new_direction).unwrap()
    }

    /// 均一スケール
    fn scale(&self, center: Self::Point2D, factor: T) -> Self::Transformed {
        let relative_point = Vector2D::from_points(center, self.point());
        let scaled_point = relative_point * factor;
        let new_point = center + scaled_point;

        // 方向ベクトルはスケールされない（正規化済み）
        Self::new(new_point, self.direction().as_vector()).unwrap()
    }
}
