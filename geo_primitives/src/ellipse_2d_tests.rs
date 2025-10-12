//! Ellipse2D のテスト

use crate::{Circle2D, Ellipse2D, Point2D, Vector2D};
use geo_foundation::{abstract_types::geometry::core_foundation::*, Scalar};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ellipse_creation() {
        // 基本的な楕円作成
        let center = Point2D::new(2.0, 3.0);
        let ellipse = Ellipse2D::new(center, 5.0, 3.0, 0.0).unwrap();

        assert_eq!(ellipse.center(), center);
        assert_eq!(ellipse.semi_major_axis(), 5.0);
        assert_eq!(ellipse.semi_minor_axis(), 3.0);
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
        let ellipse = Ellipse2D::axis_aligned(Point2D::new(0.0, 0.0), 4.0, 2.0).unwrap();

        assert_eq!(ellipse.rotation(), 0.0);
        assert_eq!(ellipse.semi_major_axis(), 4.0);
        assert_eq!(ellipse.semi_minor_axis(), 2.0);
    }

    #[test]
    fn test_from_circle() {
        let circle = Circle2D::new(Point2D::new(1.0, 2.0), 3.0).unwrap();
        let ellipse = Ellipse2D::from_circle(&circle);

        assert_eq!(ellipse.center(), circle.center());
        assert_eq!(ellipse.semi_major_axis(), circle.radius());
        assert_eq!(ellipse.semi_minor_axis(), circle.radius());
        assert!(ellipse.is_circle());
    }

    #[test]
    fn test_unit_ellipse() {
        let ellipse = Ellipse2D::unit_ellipse(0.5).unwrap();

        assert_eq!(ellipse.center(), Point2D::origin());
        assert_eq!(ellipse.semi_major_axis(), 1.0);
        assert_eq!(ellipse.semi_minor_axis(), 0.5);
    }

    #[test]
    fn test_from_five_points() {
        // 楕円状に配置された5点
        let points = [
            Point2D::new(3.0, 0.0),
            Point2D::new(0.0, 2.0),
            Point2D::new(-3.0, 0.0),
            Point2D::new(0.0, -2.0),
            Point2D::new(2.0, 1.0),
        ];

        let ellipse = Ellipse2D::from_five_points(points);
        assert!(ellipse.is_some());

        let ellipse = ellipse.unwrap();
        assert!(ellipse.semi_major_axis() >= ellipse.semi_minor_axis());
    }

    #[test]
    fn test_ellipse_properties() {
        let ellipse = Ellipse2D::axis_aligned(Point2D::origin(), 5.0, 3.0).unwrap();

        // 離心率
        let eccentricity = ellipse.eccentricity();
        let expected_e = (1.0 - (3.0 * 3.0) / (5.0 * 5.0)).sqrt();
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
        let circle_ellipse = Ellipse2D::axis_aligned(Point2D::origin(), 3.0, 3.0).unwrap();
        assert!(circle_ellipse.is_circle());
        assert_eq!(circle_ellipse.eccentricity(), 0.0);

        // 円への変換
        let circle = circle_ellipse.to_circle().unwrap();
        assert_eq!(circle.radius(), 3.0);
    }

    #[test]
    fn test_degenerate_ellipse() {
        let tiny_ellipse = Ellipse2D::axis_aligned(Point2D::origin(), 1e-12, 1e-12).unwrap();
        assert!(tiny_ellipse.is_degenerate());
    }

    #[test]
    fn test_point_at_parameter() {
        let ellipse = Ellipse2D::axis_aligned(Point2D::origin(), 4.0, 2.0).unwrap();

        // 主軸上の点
        let point_0 = ellipse.point_at_parameter(0.0);
        assert!((point_0.x() - 4.0).abs() < 1e-10);
        assert!((point_0.y() - 0.0).abs() < 1e-10);

        let point_pi2 = ellipse.point_at_parameter(std::f64::consts::PI / 2.0);
        assert!((point_pi2.x() - 0.0).abs() < 1e-10);
        assert!((point_pi2.y() - 2.0).abs() < 1e-10);

        let point_pi = ellipse.point_at_parameter(std::f64::consts::PI);
        assert!((point_pi.x() - (-4.0)).abs() < 1e-10);
        assert!((point_pi.y() - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_point_at_angle() {
        let ellipse = Ellipse2D::axis_aligned(Point2D::origin(), 3.0, 1.5).unwrap();

        let point = ellipse.point_at_angle(std::f64::consts::PI / 4.0);

        // 45度での点をチェック
        let expected_x = 3.0 * (std::f64::consts::PI / 4.0).cos();
        let expected_y = 1.5 * (std::f64::consts::PI / 4.0).sin();

        assert!((point.x() - expected_x).abs() < 1e-10);
        assert!((point.y() - expected_y).abs() < 1e-10);
    }

    #[test]
    fn test_tangent_at_parameter() {
        let ellipse = Ellipse2D::axis_aligned(Point2D::origin(), 4.0, 2.0).unwrap();

        // t=0での接線（Y軸方向）
        let tangent_0 = ellipse.tangent_at_parameter(0.0);
        assert!((tangent_0.x() - 0.0).abs() < 1e-10);
        assert!(tangent_0.y() > 0.0); // 正のY方向

        // t=π/2での接線（X軸負方向）
        let tangent_pi2 = ellipse.tangent_at_parameter(std::f64::consts::PI / 2.0);
        assert!(tangent_pi2.x() < 0.0); // 負のX方向
        assert!((tangent_pi2.y() - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_contains_point() {
        let ellipse = Ellipse2D::axis_aligned(Point2D::origin(), 5.0, 3.0).unwrap();

        // 中心点
        assert!(ellipse.contains_point(&Point2D::origin()));

        // 楕円内部の点
        assert!(ellipse.contains_point(&Point2D::new(2.0, 1.0)));

        // 楕円外部の点
        assert!(!ellipse.contains_point(&Point2D::new(6.0, 0.0)));
        assert!(!ellipse.contains_point(&Point2D::new(0.0, 4.0)));

        // 境界上の点（近似）
        let boundary_point = ellipse.point_at_parameter(std::f64::consts::PI / 6.0);
        // 楕円上の点であることを別の方法で確認（楕円方程式を使用）
        let center = ellipse.center();
        let dx = boundary_point.x() - center.x();
        let dy = boundary_point.y() - center.y();
        let normalized_distance_squared = (dx / 5.0) * (dx / 5.0) + (dy / 3.0) * (dy / 3.0);
        assert!((normalized_distance_squared - 1.0).abs() < 1e-10); // 楕円方程式による検証
    }

    #[test]
    fn test_distance_to_point() {
        let ellipse = Ellipse2D::axis_aligned(Point2D::origin(), 4.0, 2.0).unwrap();

        // 中心からの距離（内部点なので0ではないが小さい値）
        let distance_center = ellipse.distance_to_point(&Point2D::origin());
        assert!(distance_center >= 0.0);

        // 楕円上の点への距離（理論的には0に近い）
        let boundary_point = ellipse.point_at_parameter(0.0);
        let distance_boundary = ellipse.distance_to_point(&boundary_point);
        assert!(distance_boundary < 1e-5); // 数値誤差を考慮
    }

    #[test]
    fn test_transformations() {
        let ellipse = Ellipse2D::axis_aligned(Point2D::new(1.0, 1.0), 3.0, 2.0).unwrap();

        // 平行移動
        let offset = Vector2D::new(2.0, 3.0);
        let translated = ellipse.translate(&offset);
        assert_eq!(translated.center(), Point2D::new(3.0, 4.0));
        assert_eq!(translated.semi_major_axis(), 3.0);
        assert_eq!(translated.semi_minor_axis(), 2.0);

        // スケーリング
        let scaled = ellipse.scale(2.0).unwrap();
        assert_eq!(scaled.center(), ellipse.center());
        assert_eq!(scaled.semi_major_axis(), 6.0);
        assert_eq!(scaled.semi_minor_axis(), 4.0);

        // 負のスケーリング
        let invalid_scale = ellipse.scale(-1.0);
        assert!(invalid_scale.is_none());

        // 回転
        let rotated = ellipse.rotate(std::f64::consts::PI / 4.0);
        assert_eq!(rotated.center(), ellipse.center());
        assert!((rotated.rotation() - std::f64::consts::PI / 4.0).abs() < 1e-10);
    }

    #[test]
    fn test_rotation_around_origin() {
        let ellipse = Ellipse2D::axis_aligned(Point2D::new(3.0, 0.0), 2.0, 1.0).unwrap();

        // 90度回転
        let rotated = ellipse.rotate_around_origin(std::f64::consts::PI / 2.0);

        // 中心が回転する
        let expected_center = Point2D::new(0.0, 3.0);
        assert!((rotated.center().x() - expected_center.x()).abs() < 1e-10);
        assert!((rotated.center().y() - expected_center.y()).abs() < 1e-10);

        // 軸の長さは変わらない
        assert_eq!(rotated.semi_major_axis(), 2.0);
        assert_eq!(rotated.semi_minor_axis(), 1.0);
    }

    #[test]
    fn test_bounding_box() {
        // 軸に平行な楕円
        let ellipse = Ellipse2D::axis_aligned(Point2D::new(2.0, 3.0), 4.0, 2.0).unwrap();
        let bbox = ellipse.bounding_box();

        assert_eq!(bbox.min(), Point2D::new(-2.0, 1.0));
        assert_eq!(bbox.max(), Point2D::new(6.0, 5.0));

        // 回転した楕円（より複雑な境界ボックス）
        let rotated_ellipse = ellipse.rotate(std::f64::consts::PI / 4.0);
        let rotated_bbox = rotated_ellipse.bounding_box();

        // 回転により境界ボックスが変化することを確認（必ずしも大きくなるとは限らない）
        // 少なくとも楕円の中心が含まれることを確認
        assert!(rotated_bbox.contains_point(&rotated_ellipse.center()));
        assert!(rotated_bbox.width() > 0.0);
        assert!(rotated_bbox.height() > 0.0);
    }

    #[test]
    fn test_perimeter_approximation() {
        // 円の場合
        let circle_ellipse = Ellipse2D::axis_aligned(Point2D::origin(), 3.0, 3.0).unwrap();
        let circle_perimeter = circle_ellipse.perimeter();
        let expected_circle_perimeter = 2.0 * std::f64::consts::PI * 3.0;
        assert!(
            (circle_perimeter - expected_circle_perimeter).abs() / expected_circle_perimeter < 0.01
        );

        // 一般的な楕円
        let ellipse = Ellipse2D::axis_aligned(Point2D::origin(), 5.0, 3.0).unwrap();
        let perimeter = ellipse.perimeter();
        assert!(perimeter > 0.0);
        assert!(perimeter > 2.0 * std::f64::consts::PI * 3.0); // 最小の円周より大きい
        assert!(perimeter < 2.0 * std::f64::consts::PI * 5.0); // 最大の円周より小さい
    }

    // === Foundation トレイトのテスト ===

    #[test]
    fn test_geometry_foundation() {
        let ellipse = Ellipse2D::axis_aligned(Point2D::new(1.0, 2.0), 3.0, 2.0).unwrap();

        // CoreFoundation
        let bbox = ellipse.bounding_box();
        assert_eq!(bbox.min(), Point2D::new(-2.0, 0.0));
        assert_eq!(bbox.max(), Point2D::new(4.0, 4.0));
    }

    #[test]
    fn test_basic_metrics() {
        let ellipse = Ellipse2D::axis_aligned(Point2D::origin(), 4.0, 3.0).unwrap();

        // BasicMetrics
        let length = ellipse.length().unwrap();
        assert!(length > 0.0);
        assert_eq!(length, ellipse.perimeter());
    }

    #[test]
    fn test_basic_containment() {
        let ellipse = Ellipse2D::axis_aligned(Point2D::origin(), 5.0, 3.0).unwrap();

        // BasicContainment
        assert!(ellipse.contains_point(&Point2D::origin()));
        assert!(!ellipse.contains_point(&Point2D::new(6.0, 0.0)));

        // 境界上の点
        let boundary_point = ellipse.point_at_parameter(0.0);
        let distance_to_boundary = ellipse.distance_to_point(&boundary_point);
        assert!(distance_to_boundary < 1e-5); // 数値誤差を考慮した境界判定

        let distance = ellipse.distance_to_point(&Point2D::new(10.0, 0.0));
        assert!(distance > 0.0);
    }

    #[test]
    fn test_basic_parametric() {
        let ellipse = Ellipse2D::axis_aligned(Point2D::origin(), 4.0, 2.0).unwrap();

        // BasicParametric
        let (start, end) = ellipse.parameter_range();
        assert_eq!(start, 0.0);
        assert!((end - 2.0 * std::f64::consts::PI).abs() < 1e-10);

        let point = ellipse.point_at_parameter(std::f64::consts::PI / 2.0);
        assert!((point.x() - 0.0).abs() < 1e-10);
        assert!((point.y() - 2.0).abs() < 1e-10);

        let tangent = ellipse.tangent_at_parameter(0.0);
        assert!(tangent.length() > 0.0);
    }

    #[test]
    fn test_f32_compatibility() {
        // f32での基本操作
        let ellipse =
            Ellipse2D::axis_aligned(Point2D::new(0.0f32, 0.0f32), 3.0f32, 2.0f32).unwrap();

        assert_eq!(ellipse.semi_major_axis(), 3.0f32);
        assert_eq!(ellipse.semi_minor_axis(), 2.0f32);

        let area = ellipse.area();
        let expected_area = std::f32::consts::PI * 3.0f32 * 2.0f32;
        assert!((area - expected_area).abs() < 1e-6);

        // foundation トレイト
        let bbox = ellipse.bounding_box();
        assert_eq!(bbox.min(), Point2D::new(-3.0f32, -2.0f32));
        assert_eq!(bbox.max(), Point2D::new(3.0f32, 2.0f32));
    }
}
