//! LineSegment3D Transform実装
//!
//! geo_foundation::extensions::BasicTransformトレイトの実装
//! Core/Extension分離パターンに従った変換機能の実装

use crate::{LineSegment3D, Point3D, Vector3D};
use geo_foundation::{extensions::BasicTransform, Angle, Scalar};

// ============================================================================
// BasicTransform implementation (geo_foundation統合)
// ============================================================================

impl<T: Scalar> BasicTransform<T> for LineSegment3D<T> {
    type Transformed = LineSegment3D<T>;
    type Vector2D = Vector3D<T>; // 3Dでは Vector3D を使用
    type Point2D = Point3D<T>; // 3Dでは Point3D を使用
    type Angle = Angle<T>;

    /// 平行移動
    fn translate(&self, translation: Self::Vector2D) -> Self::Transformed {
        let new_start = self.start() + translation;
        let new_end = self.end() + translation;
        Self::new(new_start, new_end).unwrap()
    }

    /// 回転（Z軸周り）
    fn rotate(&self, center: Self::Point2D, angle: Self::Angle) -> Self::Transformed {
        let cos_angle = angle.cos();
        let sin_angle = angle.sin();

        // 始点を回転（Z軸周り）
        let start_relative = self.start() - center;
        let rotated_start = Point3D::new(
            start_relative.x() * cos_angle - start_relative.y() * sin_angle,
            start_relative.x() * sin_angle + start_relative.y() * cos_angle,
            start_relative.z(), // Z座標は変更なし
        ) + center.to_vector();

        // 終点を回転（Z軸周り）
        let end_relative = self.end() - center;
        let rotated_end = Point3D::new(
            end_relative.x() * cos_angle - end_relative.y() * sin_angle,
            end_relative.x() * sin_angle + end_relative.y() * cos_angle,
            end_relative.z(), // Z座標は変更なし
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

// ============================================================================
// テスト
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use geo_foundation::extensions::BasicTransform;

    #[test]
    fn test_line_segment3d_basic_transform_translate() {
        let segment =
            LineSegment3D::new(Point3D::new(0.0, 0.0, 0.0), Point3D::new(2.0, 0.0, 1.0)).unwrap();

        let translation = Vector3D::new(1.0, 1.0, 1.0);
        let translated = segment.translate(translation);

        assert_eq!(translated.start(), Point3D::new(1.0, 1.0, 1.0));
        assert_eq!(translated.end(), Point3D::new(3.0, 1.0, 2.0));
    }

    #[test]
    fn test_line_segment3d_basic_transform_rotate() {
        let segment =
            LineSegment3D::new(Point3D::new(1.0, 0.0, 0.0), Point3D::new(2.0, 0.0, 1.0)).unwrap();

        let center = Point3D::new(0.0, 0.0, 0.0);
        let angle = Angle::from_degrees(90.0);
        let rotated = segment.rotate(center, angle);

        // 90度回転後の位置を確認（Z軸周り）
        assert!((rotated.start().x() - 0.0).abs() < 1e-10);
        assert!((rotated.start().y() - 1.0).abs() < 1e-10);
        assert!((rotated.start().z() - 0.0).abs() < 1e-10);
        assert!((rotated.end().x() - 0.0).abs() < 1e-10);
        assert!((rotated.end().y() - 2.0).abs() < 1e-10);
        assert!((rotated.end().z() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_line_segment3d_basic_transform_scale() {
        let segment =
            LineSegment3D::new(Point3D::new(1.0, 1.0, 1.0), Point3D::new(3.0, 1.0, 2.0)).unwrap();

        let center = Point3D::new(0.0, 0.0, 0.0);
        let scaled = segment.scale(center, 2.0);

        assert_eq!(scaled.start(), Point3D::new(2.0, 2.0, 2.0));
        assert_eq!(scaled.end(), Point3D::new(6.0, 2.0, 4.0));
        assert_eq!(scaled.length(), segment.length() * 2.0);
    }
}
