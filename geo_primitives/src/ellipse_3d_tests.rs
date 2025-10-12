//! Ellipse3D の基本テスト
//!
//! 基本機能のみテスト：作成、アクセサ、基本プロパティ

use crate::{Circle3D, Ellipse3D, Point3D, Vector3D};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_creation() {
        // XY平面上の基本的な楕円作成
        let center = Point3D::new(1.0, 2.0, 3.0);
        let ellipse = Ellipse3D::xy_aligned(center, 5.0, 3.0).unwrap();

        assert_eq!(ellipse.center(), center);
        assert_eq!(ellipse.semi_major_axis(), 5.0);
        assert_eq!(ellipse.semi_minor_axis(), 3.0);
        assert_eq!(ellipse.normal(), Vector3D::unit_z());
        assert_eq!(ellipse.major_axis_direction(), Vector3D::unit_x());

        // 不正な楕円（短軸が長軸より大きい）
        let invalid = Ellipse3D::xy_aligned(center, 3.0, 5.0);
        assert!(invalid.is_none());

        // 負の半軸
        let invalid2 = Ellipse3D::xy_aligned(center, -1.0, 2.0);
        assert!(invalid2.is_none());
    }

    #[test]
    fn test_from_circle() {
        let circle = Circle3D::new_xy_plane(Point3D::new(1.0, 2.0, 3.0), 4.0).unwrap();
        let ellipse = Ellipse3D::from_circle(&circle);

        assert_eq!(ellipse.center(), circle.center());
        assert_eq!(ellipse.semi_major_axis(), circle.radius());
        assert_eq!(ellipse.semi_minor_axis(), circle.radius());
        assert_eq!(ellipse.normal(), circle.normal());
        assert!(ellipse.is_circle());
    }

    #[test]
    fn test_basic_properties() {
        let ellipse = Ellipse3D::xy_aligned(Point3D::origin(), 5.0, 3.0).unwrap();

        // 離心率
        let eccentricity = ellipse.eccentricity();
        let expected_e = (1.0f64 - (3.0f64 * 3.0f64) / (5.0f64 * 5.0f64)).sqrt();
        assert!((eccentricity - expected_e).abs() < 1e-10);

        // 面積
        let area = ellipse.area();
        let expected_area = std::f64::consts::PI * 5.0 * 3.0;
        assert!((area - expected_area).abs() < 1e-10);

        // 円判定
        assert!(!ellipse.is_circle());

        // 退化判定
        assert!(!ellipse.is_degenerate());
    }

    #[test]
    fn test_circle_detection() {
        let circle_ellipse = Ellipse3D::xy_aligned(Point3D::origin(), 3.0, 3.0).unwrap();
        assert!(circle_ellipse.is_circle());
        assert_eq!(circle_ellipse.eccentricity(), 0.0);

        // 円への変換
        let circle = circle_ellipse.to_circle().unwrap();
        assert_eq!(circle.radius(), 3.0);
    }

    #[test]
    fn test_degenerate_ellipse() {
        let tiny_ellipse = Ellipse3D::xy_aligned(Point3D::origin(), 1e-12, 1e-12).unwrap();
        assert!(tiny_ellipse.is_degenerate());
    }

    #[test]
    fn test_axis_directions() {
        let ellipse = Ellipse3D::xy_aligned(Point3D::origin(), 4.0, 2.0).unwrap();

        // 長軸方向
        let major_dir = ellipse.major_axis_direction();
        assert_eq!(major_dir, Vector3D::unit_x());

        // 短軸方向
        let minor_dir = ellipse.minor_axis_direction();
        assert_eq!(minor_dir, Vector3D::unit_y());

        // 基本的な直交性確認
        let dot1: f64 = major_dir.dot(&minor_dir);
        let dot2: f64 = major_dir.dot(&ellipse.normal());
        let dot3: f64 = minor_dir.dot(&ellipse.normal());
        assert!(dot1.abs() < 1e-10f64);
        assert!(dot2.abs() < 1e-10f64);
        assert!(dot3.abs() < 1e-10f64);
    }

    #[test]
    fn test_simple_parametric() {
        let ellipse = Ellipse3D::xy_aligned(Point3D::origin(), 4.0, 2.0).unwrap();

        // 基本的なパラメータでの点
        let point_0 = ellipse.point_at_parameter(0.0);
        assert!((point_0.x() - 4.0f64).abs() < 1e-10);
        assert!((point_0.y() - 0.0f64).abs() < 1e-10);
        assert!((point_0.z() - 0.0f64).abs() < 1e-10);

        let point_pi2 = ellipse.point_at_parameter(std::f64::consts::PI / 2.0);
        assert!((point_pi2.x() - 0.0f64).abs() < 1e-10);
        assert!((point_pi2.y() - 2.0f64).abs() < 1e-10);
        assert!((point_pi2.z() - 0.0f64).abs() < 1e-10);
    }

    #[test]
    fn test_f32_compatibility() {
        // f32での基本操作
        let ellipse =
            Ellipse3D::xy_aligned(Point3D::new(0.0f32, 0.0f32, 0.0f32), 3.0f32, 2.0f32).unwrap();

        assert_eq!(ellipse.semi_major_axis(), 3.0f32);
        assert_eq!(ellipse.semi_minor_axis(), 2.0f32);

        let area = ellipse.area();
        let expected_area = std::f32::consts::PI * 3.0f32 * 2.0f32;
        assert!((area - expected_area).abs() < 1e-6);
    }
}
