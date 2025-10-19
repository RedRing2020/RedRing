//! Ray2D Transform実装
//!
//! geo_foundation::extensions::BasicTransformトレイトの実装
//! Core/Extension分離パターンに従った変換機能の実装

use crate::{Point2D, Ray2D, Vector2D};
use geo_foundation::{extensions::BasicTransform, Angle, Scalar};

// ============================================================================
// BasicTransform implementation (geo_foundation統合)
// ============================================================================

impl<T: Scalar> BasicTransform<T> for Ray2D<T> {
    type Transformed = Ray2D<T>;
    type Vector2D = Vector2D<T>;
    type Point2D = Point2D<T>;
    type Angle = Angle<T>;

    /// 平行移動
    fn translate(&self, translation: Self::Vector2D) -> Self::Transformed {
        let new_origin = self.origin() + translation;
        Self::new(new_origin, self.direction()).unwrap()
    }

    /// 回転
    fn rotate(&self, center: Self::Point2D, angle: Self::Angle) -> Self::Transformed {
        let cos_angle = angle.cos();
        let sin_angle = angle.sin();

        // 起点を回転
        let relative_origin = self.origin() - center;
        let rotated_origin_x = relative_origin.x() * cos_angle - relative_origin.y() * sin_angle;
        let rotated_origin_y = relative_origin.x() * sin_angle + relative_origin.y() * cos_angle;
        let new_origin = center + Vector2D::new(rotated_origin_x, rotated_origin_y);

        // 方向ベクトルを回転
        let dir = self.direction();
        let rotated_dir_x = dir.x() * cos_angle - dir.y() * sin_angle;
        let rotated_dir_y = dir.x() * sin_angle + dir.y() * cos_angle;
        let new_direction = Vector2D::new(rotated_dir_x, rotated_dir_y);

        Self::new(new_origin, new_direction).unwrap()
    }

    /// 均一スケール
    fn scale(&self, center: Self::Point2D, factor: T) -> Self::Transformed {
        let relative_origin = Vector2D::from_points(center, self.origin());
        let scaled_origin = relative_origin * factor;
        let new_origin = center + scaled_origin;

        // 方向ベクトルはスケールされない（正規化済み）
        Self::new(new_origin, self.direction()).unwrap()
    }
}
