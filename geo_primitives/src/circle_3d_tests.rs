//! Circle3D のテスト

use crate::{Circle3D, Point3D, Vector3D};
use geo_foundation::abstract_types::geometry::foundation::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_circle3d_creation() {
        let center = Point3D::new(0.0_f64, 0.0, 0.0);
        let normal = Vector3D::unit_z();
        let radius = 5.0;

        let circle = Circle3D::new(center, normal, radius).unwrap();
        assert_eq!(circle.center(), center);
        assert_eq!(circle.radius(), radius);
        assert_eq!(circle.diameter(), 10.0);
    }

    #[test]
    fn test_circle3d_invalid_creation() {
        let center = Point3D::new(0.0_f64, 0.0, 0.0);
        let normal = Vector3D::unit_z();

        // 負の半径
        assert!(Circle3D::new(center, normal, -1.0).is_none());

        // ゼロ半径
        assert!(Circle3D::new(center, normal, 0.0).is_none());

        // ゼロベクトル法線
        assert!(Circle3D::new(center, Vector3D::zero(), 1.0).is_none());
    }

    #[test]
    fn test_circle3d_plane_constructors() {
        let center = Point3D::new(1.0, 2.0, 3.0);
        let radius = 4.0;

        let xy_circle = Circle3D::new_xy_plane(center, radius).unwrap();
        let xz_circle = Circle3D::new_xz_plane(center, radius).unwrap();
        let yz_circle = Circle3D::new_yz_plane(center, radius).unwrap();

        assert_eq!(xy_circle.normal(), Vector3D::unit_z());
        assert_eq!(xz_circle.normal(), Vector3D::unit_y());
        assert_eq!(yz_circle.normal(), Vector3D::unit_x());
    }

    #[test]
    fn test_circle3d_metrics() {
        let circle = Circle3D::new_xy_plane(Point3D::origin(), 2.0).unwrap();

        // 円周 = 2πr
        let expected_circumference = 2.0 * std::f64::consts::PI * 2.0;
        assert!((circle.circumference() - expected_circumference).abs() < 1e-10);

        // 面積 = πr²
        let expected_area = std::f64::consts::PI * 4.0;
        assert!((circle.area() - expected_area).abs() < 1e-10);
    }

    #[test]
    fn test_circle3d_point_at_angle() {
        let circle = Circle3D::new_xy_plane(Point3D::origin(), 1.0_f64).unwrap();

        // 0度（X軸正方向）
        let p0 = circle.point_at_angle(0.0);
        assert!((p0.x() - 1.0).abs() < 1e-10);
        assert!(p0.y().abs() < 1e-10);
        assert!(p0.z().abs() < 1e-10);

        // 90度（Y軸正方向）
        let p90 = circle.point_at_angle(std::f64::consts::PI / 2.0);
        assert!(p90.x().abs() < 1e-10);
        assert!((p90.y() - 1.0).abs() < 1e-10);
        assert!(p90.z().abs() < 1e-10);
    }

    #[test]
    fn test_circle3d_distance_calculations() {
        let circle = Circle3D::new_xy_plane(Point3D::origin(), 2.0_f64).unwrap();

        // 中心への距離
        let point = Point3D::new(3.0, 4.0, 0.0);
        assert_eq!(circle.distance_to_center(&point), 5.0);

        // 円上の点
        let point_on_circle = Point3D::new(2.0, 0.0, 0.0);
        assert!(circle.point_on_plane(&point_on_circle, 1e-10));
        assert!(circle.distance_to_circle(&point_on_circle).abs() < 1e-10);
    }

    // === foundation トレイトテスト ===

    #[test]
    fn test_geometry_foundation() {
        let circle = Circle3D::new_xy_plane(Point3D::new(1.0, 1.0, 0.0), 2.0).unwrap();
        let bbox = circle.bounding_box();

        // 境界ボックスは円を包含する
        assert!(bbox.min().x() <= -1.0); // center.x - radius
        assert!(bbox.max().x() >= 3.0); // center.x + radius
        assert!(bbox.min().y() <= -1.0); // center.y - radius
        assert!(bbox.max().y() >= 3.0); // center.y + radius
    }

    #[test]
    fn test_basic_metrics() {
        let circle = Circle3D::new_xy_plane(Point3D::origin(), 3.0).unwrap();

        let circumference = BasicMetrics::perimeter(&circle).unwrap();
        let area = BasicMetrics::area(&circle).unwrap();

        assert!((circumference - 6.0 * std::f64::consts::PI).abs() < 1e-10);
        assert!((area - 9.0 * std::f64::consts::PI).abs() < 1e-10);
    }

    #[test]
    fn test_basic_containment() {
        let circle = Circle3D::new_xy_plane(Point3D::origin(), 1.0).unwrap();

        // 円上の点
        let point_on_circle = Point3D::new(1.0, 0.0, 0.0);
        assert!(circle.contains_point(&point_on_circle));
        assert!(circle.on_boundary(&point_on_circle, 1e-10));

        // 円外の点
        let point_outside = Point3D::new(2.0, 0.0, 0.0);
        assert!(!circle.contains_point(&point_outside));
        assert!(!circle.on_boundary(&point_outside, 1e-10));

        // 平面外の点
        let point_off_plane = Point3D::new(1.0, 0.0, 1.0);
        assert!(!circle.contains_point(&point_off_plane));
    }

    #[test]
    fn test_basic_parametric() {
        let circle = Circle3D::new_xy_plane(Point3D::origin(), 1.0).unwrap();

        let (min_t, max_t) = circle.parameter_range();
        assert_eq!(min_t, 0.0);
        assert!((max_t - 2.0 * std::f64::consts::PI).abs() < 1e-10);

        // パラメータでの点取得
        let point = circle.point_at_parameter(0.0);
        assert!((point.x() - 1.0).abs() < 1e-10);
        assert!(point.y().abs() < 1e-10);

        // 接線ベクトル
        let tangent = circle.tangent_at_parameter(0.0);
        // t=0での接線はY方向
        assert!(tangent.x().abs() < 1e-10);
        assert!((tangent.y() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_circle3d_f32_compatibility() {
        let circle_f32 =
            Circle3D::new_xy_plane(Point3D::new(0.0f32, 0.0f32, 0.0f32), 1.0f32).unwrap();

        let circle_f64 =
            Circle3D::new_xy_plane(Point3D::new(0.0f64, 0.0f64, 0.0f64), 1.0f64).unwrap();

        assert_eq!(circle_f32.radius(), 1.0f32);
        assert_eq!(circle_f64.radius(), 1.0f64);
    }
}
