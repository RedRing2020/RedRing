//! Ray3D テストモジュール

use crate::{Point3D, Ray3D, Vector3D};
use geo_foundation::Scalar;

// BasicTransformの実装を有効にするため
#[allow(unused_imports)]
// use crate::ray_3d_transform; // 削除済み
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ray_3d_creation() {
        let origin = Point3D::new(1.0, 2.0, 3.0);
        let direction = Vector3D::new(1.0, 0.0, 0.0);

        let ray = Ray3D::new(origin, direction).unwrap();

        assert_eq!(ray.origin(), origin);
        assert_eq!(ray.direction().as_vector(), Vector3D::new(1.0, 0.0, 0.0));
    }

    #[test]
    fn test_ray_3d_creation_with_normalization() {
        let origin = Point3D::new(0.0, 0.0, 0.0);
        let direction = Vector3D::new(3.0, 4.0, 0.0); // 長さ5のベクトル

        let ray = Ray3D::new(origin, direction).unwrap();

        assert_eq!(ray.origin(), origin);
        assert!((ray.direction().length() - 1.0).abs() < 1e-10);
        assert!((ray.direction().x() - 0.6).abs() < 1e-10);
        assert!((ray.direction().y() - 0.8).abs() < 1e-10);
    }

    #[test]
    fn test_ray_3d_creation_from_zero_vector() {
        let origin = Point3D::new(0.0, 0.0, 0.0);
        let direction = Vector3D::zero();

        let ray = Ray3D::new(origin, direction);

        assert!(ray.is_none());
    }

    #[test]
    fn test_ray_3d_from_points() {
        let start = Point3D::new(1.0, 2.0, 3.0);
        let through = Point3D::new(4.0, 6.0, 3.0);

        let ray = Ray3D::from_points(start, through).unwrap();

        assert_eq!(ray.origin(), start);
        assert!((ray.direction().length() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_point_at_parameter() {
        let origin = Point3D::new(1.0, 2.0, 3.0);
        let direction = Vector3D::new(1.0, 0.0, 0.0);
        let ray = Ray3D::new(origin, direction).unwrap();

        let point_at_0 = ray.point_at_parameter(0.0);
        let point_at_5 = ray.point_at_parameter(5.0);

        assert_eq!(point_at_0, Point3D::new(1.0, 2.0, 3.0));
        assert_eq!(point_at_5, Point3D::new(6.0, 2.0, 3.0));
    }

    #[test]
    fn test_parameter_for_point() {
        let origin = Point3D::new(0.0, 0.0, 0.0);
        let direction = Vector3D::new(1.0, 0.0, 0.0);
        let ray = Ray3D::new(origin, direction).unwrap();

        let point1 = Point3D::new(3.0, 0.0, 0.0);
        let point2 = Point3D::new(-2.0, 0.0, 0.0);

        assert_eq!(ray.parameter_for_point(&point1), 3.0);
        assert_eq!(ray.parameter_for_point(&point2), -2.0);
    }

    #[test]
    fn test_contains_point() {
        let origin = Point3D::new(0.0, 0.0, 0.0);
        let direction = Vector3D::new(1.0, 0.0, 0.0);
        let ray = Ray3D::new(origin, direction).unwrap();

        let point_on_ray = Point3D::new(5.0, 0.0, 0.0);
        let point_behind = Point3D::new(-2.0, 0.0, 0.0);
        let point_off_ray = Point3D::new(3.0, 1.0, 0.0);

        assert!(ray.contains_point(&point_on_ray, 1e-10));
        assert!(!ray.contains_point(&point_behind, 1e-10));
        assert!(!ray.contains_point(&point_off_ray, 1e-10));
    }

    #[test]
    fn test_points_towards() {
        let origin = Point3D::new(0.0, 0.0, 0.0);
        let direction = Vector3D::new(1.0, 0.0, 0.0);
        let ray = Ray3D::new(origin, direction).unwrap();

        let point_ahead = Point3D::new(5.0, 0.0, 0.0);
        let point_behind = Point3D::new(-2.0, 0.0, 0.0);
        let point_sideways = Point3D::new(0.0, 5.0, 0.0);

        assert!(ray.points_towards(&point_ahead));
        assert!(!ray.points_towards(&point_behind));
        assert!(!ray.points_towards(&point_sideways));
    }

    #[test]
    fn test_distance_to_point() {
        let origin = Point3D::new(0.0, 0.0, 0.0);
        let direction = Vector3D::new(1.0, 0.0, 0.0);
        let ray = Ray3D::new(origin, direction).unwrap();

        // 点が Ray 上にある場合
        let point_on_ray = Point3D::new(3.0, 0.0, 0.0);
        assert!((ray.distance_to_point(&point_on_ray) - 0.0).abs() < 1e-10);

        // 点が Ray の横にある場合
        let point_sideways = Point3D::new(3.0, 4.0, 0.0);
        assert!((ray.distance_to_point(&point_sideways) - 4.0).abs() < 1e-10);

        // 点が Ray の後ろにある場合
        let point_behind = Point3D::new(-3.0, 4.0, 0.0);
        let expected_distance = (3.0_f64.powi(2) + 4.0_f64.powi(2)).sqrt();
        assert!((ray.distance_to_point(&point_behind) - expected_distance).abs() < 1e-10);
    }

    #[test]
    fn test_closest_point_on_ray() {
        let origin = Point3D::new(0.0, 0.0, 0.0);
        let direction = Vector3D::new(1.0, 0.0, 0.0);
        let ray = Ray3D::new(origin, direction).unwrap();

        // 点が Ray の正の方向にある場合
        let point1 = Point3D::new(3.0, 4.0, 0.0);
        let closest1 = ray.closest_point_on_ray(&point1);
        assert_eq!(closest1, Point3D::new(3.0, 0.0, 0.0));

        // 点が Ray の後ろにある場合
        let point2 = Point3D::new(-2.0, 3.0, 0.0);
        let closest2 = ray.closest_point_on_ray(&point2);
        assert_eq!(closest2, Point3D::new(0.0, 0.0, 0.0)); // 起点が最も近い
    }

    #[test]
    fn test_reverse_direction() {
        let origin = Point3D::new(1.0, 2.0, 3.0);
        let direction = Vector3D::new(1.0, 0.0, 0.0);
        let ray = Ray3D::new(origin, direction).unwrap();

        let reversed = ray.reverse_direction();

        assert_eq!(reversed.origin(), origin);
        assert_eq!(
            reversed.direction().as_vector(),
            Vector3D::new(-1.0, 0.0, 0.0)
        );
    }

    #[test]
    fn test_axis_rays() {
        let origin = Point3D::new(1.0, 2.0, 3.0);

        let x_ray = Ray3D::along_x_axis(origin);
        let y_ray = Ray3D::along_y_axis(origin);
        let z_ray = Ray3D::along_z_axis(origin);

        assert_eq!(x_ray.direction().as_vector(), Vector3D::unit_x());
        assert_eq!(y_ray.direction().as_vector(), Vector3D::unit_y());
        assert_eq!(z_ray.direction().as_vector(), Vector3D::unit_z());
    }

    #[test]
    fn test_parameter_range() {
        let origin = Point3D::new(0.0, 0.0, 0.0);
        let direction = Vector3D::new(1.0, 0.0, 0.0);
        let ray = Ray3D::new(origin, direction).unwrap();

        let (min, max) = ray.parameter_range();
        assert_eq!(min, 0.0);
        assert_eq!(max, f64::INFINITY);
    }

    #[test]
    fn test_to_infinite_line() {
        let origin = Point3D::new(1.0, 2.0, 3.0);
        let direction = Vector3D::new(1.0, 0.0, 0.0);
        let ray = Ray3D::new(origin, direction).unwrap();

        let infinite_line = ray.to_infinite_line();

        // InfiniteLine3D の基本的な特性をテスト
        let point_on_line = infinite_line.point_at_parameter(5.0);
        assert_eq!(point_on_line, Point3D::new(6.0, 2.0, 3.0));
    }

    #[test]
    fn test_3d_diagonal_ray() {
        let origin = Point3D::new(0.0, 0.0, 0.0);
        let direction = Vector3D::new(1.0, 1.0, 1.0);
        let ray = Ray3D::new(origin, direction).unwrap();

        // 正規化確認
        let normalized_length = 1.0 / (3.0_f64).sqrt();
        assert!((ray.direction().x() - normalized_length).abs() < 1e-10);
        assert!((ray.direction().y() - normalized_length).abs() < 1e-10);
        assert!((ray.direction().z() - normalized_length).abs() < 1e-10);

        // 対角線上の点をテスト
        let point = ray.point_at_parameter((3.0_f64).sqrt());
        assert!((point.x() - 1.0).abs() < 1e-10);
        assert!((point.y() - 1.0).abs() < 1e-10);
        assert!((point.z() - 1.0).abs() < 1e-10);
    }
}
