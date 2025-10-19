//! Ray2D test file - Basic test only
use crate::{Point2D, Ray2D, Vector2D};
use geo_foundation::{extensions::BasicTransform, Angle};

// BasicTransformの実装を有効にするため
#[allow(unused_imports)]
use crate::ray_2d_transform;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_creation() {
        let origin = Point2D::new(1.0_f64, 2.0_f64);
        let direction = Vector2D::new(3.0_f64, 4.0_f64);
        let ray = Ray2D::new(origin, direction).unwrap();
        assert_eq!(ray.origin(), origin);
    }

    #[test]
    fn test_basic_transform_translate() {
        let ray = Ray2D::new(
            Point2D::new(1.0_f64, 2.0_f64),
            Vector2D::new(1.0_f64, 0.0_f64),
        )
        .unwrap();
        let offset = Vector2D::new(3.0_f64, 4.0_f64);

        let translated = BasicTransform::translate(&ray, offset);
        assert_eq!(translated.origin(), Point2D::new(4.0_f64, 6.0_f64));
        assert_eq!(translated.direction(), ray.direction());
    }

    #[test]
    fn test_basic_transform_rotate() {
        let ray = Ray2D::new(
            Point2D::new(2.0_f64, 0.0_f64),
            Vector2D::new(1.0_f64, 0.0_f64),
        )
        .unwrap();
        let center = Point2D::new(0.0_f64, 0.0_f64);
        let angle = Angle::from_degrees(90.0_f64);

        let rotated = BasicTransform::rotate(&ray, center, angle);

        // 90度回転後、(2,0) → (0,2)、方向(1,0) → (0,1)
        let expected_origin = Point2D::new(0.0_f64, 2.0_f64);
        let expected_direction = Vector2D::new(0.0_f64, 1.0_f64);

        assert!((rotated.origin().x() - expected_origin.x()).abs() < 1e-10_f64);
        assert!((rotated.origin().y() - expected_origin.y()).abs() < 1e-10_f64);
        assert!((rotated.direction().x() - expected_direction.x()).abs() < 1e-10_f64);
        assert!((rotated.direction().y() - expected_direction.y()).abs() < 1e-10_f64);
    }

    #[test]
    fn test_basic_transform_scale() {
        let ray = Ray2D::new(
            Point2D::new(2.0_f64, 3.0_f64),
            Vector2D::new(1.0_f64, 0.0_f64),
        )
        .unwrap();
        let center = Point2D::new(0.0_f64, 0.0_f64);
        let factor = 2.0_f64;

        let scaled = BasicTransform::scale(&ray, center, factor);

        // スケール後、起点は(4,6)、方向は正規化されているので変わらず
        assert_eq!(scaled.origin(), Point2D::new(4.0_f64, 6.0_f64));
        assert_eq!(scaled.direction(), ray.direction());
    }

    #[test]
    fn test_extension_translate() {
        let ray = Ray2D::new(
            Point2D::new(1.0_f64, 2.0_f64),
            Vector2D::new(1.0_f64, 0.0_f64),
        )
        .unwrap();
        let offset = Vector2D::new(3.0_f64, 4.0_f64);

        let translated = ray.translate(offset);
        assert_eq!(translated.origin(), Point2D::new(4.0_f64, 6.0_f64));
        assert_eq!(translated.direction(), ray.direction());
    }

    #[test]
    fn test_extension_reflect_x() {
        let ray = Ray2D::new(
            Point2D::new(1.0_f64, 2.0_f64),
            Vector2D::new(1.0_f64, 1.0_f64),
        )
        .unwrap();

        let reflected = ray.reflect_x();
        assert_eq!(reflected.origin(), Point2D::new(1.0_f64, -2.0_f64));

        // 方向ベクトルのY成分が反転
        let expected_dir = Vector2D::new(1.0_f64, -1.0_f64).normalize();
        assert!((reflected.direction().x() - expected_dir.x()).abs() < 1e-10_f64);
        assert!((reflected.direction().y() - expected_dir.y()).abs() < 1e-10_f64);
    }

    #[test]
    fn test_extension_reflect_y() {
        let ray = Ray2D::new(
            Point2D::new(1.0_f64, 2.0_f64),
            Vector2D::new(1.0_f64, 1.0_f64),
        )
        .unwrap();

        let reflected = ray.reflect_y();
        assert_eq!(reflected.origin(), Point2D::new(-1.0_f64, 2.0_f64));

        // 方向ベクトルのX成分が反転
        let expected_dir = Vector2D::new(-1.0_f64, 1.0_f64).normalize();
        assert!((reflected.direction().x() - expected_dir.x()).abs() < 1e-10_f64);
        assert!((reflected.direction().y() - expected_dir.y()).abs() < 1e-10_f64);
    }

    #[test]
    fn test_extension_with_direction() {
        let ray = Ray2D::new(
            Point2D::new(1.0_f64, 2.0_f64),
            Vector2D::new(1.0_f64, 0.0_f64),
        )
        .unwrap();
        let new_direction = Vector2D::new(0.0_f64, 1.0_f64);

        let modified = ray.with_direction(new_direction).unwrap();
        assert_eq!(modified.origin(), ray.origin());
        assert_eq!(modified.direction(), new_direction.normalize());
    }

    #[test]
    fn test_extension_with_origin() {
        let ray = Ray2D::new(
            Point2D::new(1.0_f64, 2.0_f64),
            Vector2D::new(1.0_f64, 0.0_f64),
        )
        .unwrap();
        let new_origin = Point2D::new(5.0_f64, 6.0_f64);

        let modified = ray.with_origin(new_origin);
        assert_eq!(modified.origin(), new_origin);
        assert_eq!(modified.direction(), ray.direction());
    }

    #[test]
    fn test_extension_to_line_segment_with_length() {
        let ray = Ray2D::new(
            Point2D::new(0.0_f64, 0.0_f64),
            Vector2D::new(1.0_f64, 0.0_f64),
        )
        .unwrap();
        let length = 5.0_f64;

        let segment = ray.to_line_segment_with_length(length).unwrap();
        assert_eq!(segment.start(), Point2D::new(0.0_f64, 0.0_f64));
        assert_eq!(segment.end(), Point2D::new(5.0_f64, 0.0_f64));

        // 負の長さは None を返す
        assert!(ray.to_line_segment_with_length(-1.0_f64).is_none());
        assert!(ray.to_line_segment_with_length(0.0_f64).is_none());
    }

    #[test]
    fn test_extension_non_uniform_scale() {
        let ray = Ray2D::new(
            Point2D::new(2.0_f64, 3.0_f64),
            Vector2D::new(1.0_f64, 1.0_f64),
        )
        .unwrap();
        let center = Point2D::new(0.0_f64, 0.0_f64);

        let scaled = ray.scale_non_uniform(&center, 2.0_f64, 3.0_f64);

        // 起点は (4.0, 9.0) になる
        assert_eq!(scaled.origin(), Point2D::new(4.0_f64, 9.0_f64));

        // 方向ベクトルは (2.0, 3.0) → 正規化される
        let expected_dir = Vector2D::new(2.0_f64, 3.0_f64).normalize();
        assert!((scaled.direction().x() - expected_dir.x()).abs() < 1e-10_f64);
        assert!((scaled.direction().y() - expected_dir.y()).abs() < 1e-10_f64);
    }
}
