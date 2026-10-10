//! Ellipse2D のテスト

use crate::{Circle2D, Ellipse2D, Point2D};
use geo_contracts::Scalar;

#[cfg(test)]
mod tests {
    use super::*;
    use analysis::test_constants::{TOLERANCE_F32, TOLERANCE_F64};

    #[test]
    fn test_ellipse_creation() {
        // 基本的な楕円作成
        let center = Point2D::new(2.0, 3.0);
        let ellipse = Ellipse2D::new(center, 5.0, 3.0, 0.0).unwrap();

        assert_eq!(ellipse.center_internal(), center);
        assert_eq!(ellipse.semi_major_internal(), 5.0);
        assert_eq!(ellipse.semi_minor_internal(), 3.0);
        assert_eq!(ellipse.rotation(), 0.0);

        // 不正な楕円（短軸が長軸より大きい）
        let invalid = Ellipse2D::new(center, 3.0, 5.0, 0.0);
        assert!(invalid.is_none());

        // 負の半軸
        let invalid2 = Ellipse2D::new(center, -1.0, 2.0, 0.0);
        assert!(invalid2.is_none());
    }

    #[test]
    fn test_axis_aligned_ellipse() {
        let ellipse = Ellipse2D::new(Point2D::new(0.0, 0.0), 4.0, 2.0, 0.0).unwrap();

        assert_eq!(ellipse.rotation(), 0.0);
        assert_eq!(ellipse.semi_major_internal(), 4.0);
        assert_eq!(ellipse.semi_minor_internal(), 2.0);
    }

    #[test]
    fn test_from_circle() {
        let circle = Circle2D::new(Point2D::new(1.0, 2.0), 3.0).unwrap();
        let ellipse = Ellipse2D::from_circle(circle);

        assert_eq!(ellipse.center_internal(), circle.center_internal());
        assert_eq!(ellipse.semi_major_internal(), circle.radius_internal());
        assert_eq!(ellipse.semi_minor_internal(), circle.radius_internal());
        assert!(ellipse.is_circle(TOLERANCE_F64));
    }

    #[test]
    fn test_ellipse_properties() {
        let ellipse = Ellipse2D::new(Point2D::origin(), 5.0, 3.0, 0.0).unwrap();

        // 離心率
        let eccentricity = ellipse.eccentricity();
        let expected_e = (1.0 - (3.0 * 3.0) / (5.0 * 5.0)).sqrt();
        assert!((eccentricity - expected_e).abs() < TOLERANCE_F64);

        // 面積
        let area = ellipse.area();
        let expected_area = std::f64::consts::PI * 5.0 * 3.0;
        assert!((area - expected_area).abs() < TOLERANCE_F64);

        // 円判定
        assert!(!ellipse.is_circle(TOLERANCE_F64));
    }

    #[test]
    fn test_circle_detection() {
        let circle_ellipse = Ellipse2D::new(Point2D::origin(), 3.0, 3.0, 0.0).unwrap();
        assert!(circle_ellipse.is_circle(TOLERANCE_F64));
        assert_eq!(circle_ellipse.eccentricity(), 0.0);

        // 円への変換
        let circle = circle_ellipse.to_circle().unwrap();
        assert_eq!(circle.radius_internal(), 3.0);
    }

    #[test]
    fn test_point_at_parameter() {
        let ellipse = Ellipse2D::new(Point2D::origin(), 4.0, 2.0, 0.0).unwrap();

        // Ellipse core parameter は local angle domain (0..2π)
        // 主軸上の点
        let point_0 = ellipse.point_at_parameter(0.0);
        assert!((point_0.x() - 4.0).abs() < TOLERANCE_F64);
        assert!((point_0.y() - 0.0).abs() < TOLERANCE_F64);

        let point_pi2 = ellipse.point_at_parameter(std::f64::consts::PI / 2.0);
        assert!((point_pi2.x() - 0.0).abs() < TOLERANCE_F64);
        assert!((point_pi2.y() - 2.0).abs() < TOLERANCE_F64);

        let point_pi = ellipse.point_at_parameter(std::f64::consts::PI);
        assert!((point_pi.x() - (-4.0)).abs() < TOLERANCE_F64);
        assert!((point_pi.y() - 0.0).abs() < TOLERANCE_F64);
    }

    #[test]
    fn test_tangent_at_parameter() {
        let ellipse = Ellipse2D::new(Point2D::origin(), 4.0, 2.0, 0.0).unwrap();

        // t=0での接線（Y軸方向）
        let tangent_0 = ellipse.tangent_at_parameter(0.0);
        assert!((tangent_0.x() - 0.0).abs() < TOLERANCE_F64);
        assert!(tangent_0.y() > 0.0); // 正のY方向

        // t=π/2での接線（X軸負方向）
        let tangent_pi2 = ellipse.tangent_at_parameter(std::f64::consts::PI / 2.0);
        assert!(tangent_pi2.x() < 0.0); // 負のX方向
        assert!((tangent_pi2.y() - 0.0).abs() < TOLERANCE_F64);
    }

    #[test]
    fn test_point_at_parameter_lies_on_ellipse() {
        let ellipse = Ellipse2D::new(Point2D::origin(), 5.0, 3.0, 0.0).unwrap();

        // 境界上の点
        let boundary_point = ellipse.point_at_parameter(std::f64::consts::PI / 6.0);
        // 楕円上の点であることを別の方法で確認（楕円方程式を使用）
        let center = ellipse.center_internal();
        let dx = boundary_point.x() - center.x();
        let dy = boundary_point.y() - center.y();
        let normalized_distance_squared = (dx / 5.0) * (dx / 5.0) + (dy / 3.0) * (dy / 3.0);
        assert!((normalized_distance_squared - 1.0).abs() < TOLERANCE_F64); // 楕円方程式による検証
    }

    #[test]
    fn test_distance_to_point() {
        let ellipse = Ellipse2D::new(Point2D::origin(), 4.0, 2.0, 0.0).unwrap();

        // 中心からの距離（内部点なので0ではないが小さい値）
        let distance_center = ellipse.distance_to_point(&Point2D::origin());
        assert!(distance_center >= 0.0);

        // 楕円上の点への距離（理論的には0に近い）
        let boundary_point = ellipse.point_at_parameter(0.0);
        let distance_boundary = ellipse.distance_to_point(&boundary_point);
        assert!(distance_boundary < TOLERANCE_F64);
    }

    #[test]
    fn test_closest_point_to() {
        let ellipse = Ellipse2D::new(Point2D::origin(), 4.0, 2.0, 0.0).unwrap();
        let closest = ellipse.closest_point(&Point2D::new(10.0, 0.0));

        assert!((closest.x() - 4.0).abs() < TOLERANCE_F64);
        assert!(closest.y().abs() < TOLERANCE_F64);
    }

    #[test]
    fn test_bounding_box() {
        // 軸に平行な楕円
        let ellipse = Ellipse2D::new(Point2D::new(2.0, 3.0), 4.0, 2.0, 0.0).unwrap();
        let bbox = ellipse.bounding_box();

        assert_eq!(bbox.min_point(), Point2D::new(-2.0, 1.0));
        assert_eq!(bbox.max_point(), Point2D::new(6.0, 5.0));
    }

    #[test]
    fn test_circumference_approximation() {
        // 円の場合
        let circle_ellipse = Ellipse2D::new(Point2D::origin(), 3.0, 3.0, 0.0).unwrap();
        let circle_circumference = circle_ellipse.circumference();
        let expected_circle_circumference = 2.0 * std::f64::consts::PI * 3.0;
        assert!(
            (circle_circumference - expected_circle_circumference).abs()
                / expected_circle_circumference
                < 0.01
        );

        // 一般的な楕円
        let ellipse = Ellipse2D::new(Point2D::origin(), 5.0, 3.0, 0.0).unwrap();
        let circumference = ellipse.circumference();
        assert!(circumference > 0.0);
        assert!(circumference > 2.0 * std::f64::consts::PI * 3.0);
        assert!(circumference < 2.0 * std::f64::consts::PI * 5.0);
    }

    #[test]
    fn test_geometry_foundation() {
        let ellipse = Ellipse2D::new(Point2D::new(1.0, 2.0), 3.0, 2.0, 0.0).unwrap();

        // CoreFoundation
        let bbox = ellipse.bounding_box();
        assert_eq!(bbox.min_point(), Point2D::new(-2.0, 0.0));
        assert_eq!(bbox.max_point(), Point2D::new(4.0, 4.0));
    }

    #[test]
    fn test_boundary_distance() {
        let ellipse = Ellipse2D::new(Point2D::origin(), 5.0, 3.0, 0.0).unwrap();

        // 境界上の点
        let boundary_point = ellipse.point_at_parameter(0.0);
        let distance_to_boundary = ellipse.distance_to_point(&boundary_point);
        assert!(distance_to_boundary < TOLERANCE_F64);

        let distance = ellipse.distance_to_point(&Point2D::new(10.0, 0.0));
        assert!(distance > 0.0);
    }

    #[test]
    fn test_basic_parametric() {
        let ellipse = Ellipse2D::new(Point2D::origin(), 4.0, 2.0, 0.0).unwrap();

        // Ellipse の parameter_range / point_at_parameter は local angle domain を使う
        let (start, end) = ellipse.parameter_range();
        assert_eq!(start, 0.0);
        assert!((end - 2.0 * std::f64::consts::PI).abs() < TOLERANCE_F64);

        let point = ellipse.point_at_parameter(std::f64::consts::PI / 2.0);
        assert!((point.x() - 0.0).abs() < TOLERANCE_F64);
        assert!((point.y() - 2.0).abs() < TOLERANCE_F64);

        let tangent = ellipse.tangent_at_parameter(0.0);
        assert!(tangent.length() > 0.0);
    }

    #[test]
    fn test_f32_compatibility() {
        // f32での基本操作
        let ellipse = Ellipse2D::new(Point2D::new(0.0f32, 0.0f32), 3.0f32, 2.0f32, 0.0).unwrap();

        assert_eq!(ellipse.semi_major_internal(), 3.0f32);
        assert_eq!(ellipse.semi_minor_internal(), 2.0f32);

        let area = ellipse.area();
        let expected_area = std::f32::consts::PI * 3.0f32 * 2.0f32;
        assert!((area - expected_area).abs() < TOLERANCE_F32);

        // foundation トレイト
        let bbox = ellipse.bounding_box();
        assert_eq!(bbox.min_point(), Point2D::new(-3.0f32, -2.0f32));
        assert_eq!(bbox.max_point(), Point2D::new(3.0f32, 2.0f32));
    }
}
