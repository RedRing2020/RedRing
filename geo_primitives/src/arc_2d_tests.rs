//! Arc2D の基本テスト
//!
//! 基本機能のみテスト：作成、アクセサ、基本プロパティ

use crate::{Arc2D, Point2D, Vector2D};
use geo_foundation::{abstracts::arc_traits::ArcMetrics, Angle};

#[cfg(test)]
mod tests {
    use super::*;

    // ヘルパー関数：ラジアンから Angle を作成
    fn angle(radians: f64) -> Angle<f64> {
        Angle::from_radians(radians)
    }

    fn angle_f32(radians: f32) -> Angle<f32> {
        Angle::from_radians(radians)
    }

    #[test]
    fn test_basic_creation() {
        // 基本的な円弧作成
        let center = Point2D::new(0.0_f64, 0.0_f64);
        let arc = Arc2D::xy_arc(center, 5.0_f64, angle(0.0), angle(std::f64::consts::PI)).unwrap();

        assert_eq!(arc.center(), center);
        assert_eq!(arc.radius(), 5.0);
        assert_eq!(arc.start_direction(), Vector2D::unit_x());
        assert_eq!(arc.start_angle(), angle(0.0));
        assert_eq!(arc.end_angle(), angle(std::f64::consts::PI));

        // 不正な円弧（負の半径）
        let invalid = Arc2D::xy_arc(center, -1.0_f64, angle(0.0), angle(std::f64::consts::PI));
        assert!(invalid.is_none());

        // ゼロ半径
        let invalid2 = Arc2D::xy_arc(center, 0.0_f64, angle(0.0), angle(std::f64::consts::PI));
        assert!(invalid2.is_none());
    }
    #[test]
    fn test_from_three_points() {
        // 3点を通る円弧
        let start = Point2D::new(1.0_f64, 0.0_f64);
        let middle = Point2D::new(0.0_f64, 1.0_f64);
        let end = Point2D::new(-1.0_f64, 0.0_f64);

        let arc = Arc2D::from_three_points(start, middle, end).unwrap();

        // 中心は原点付近のはず
        assert!((arc.center().x() - 0.0_f64).abs() < 1e-10);
        assert!((arc.center().y() - 0.0_f64).abs() < 1e-10);
        assert!((arc.radius() - 1.0_f64).abs() < 1e-10);

        // 一直線上の点では作成不可
        let collinear_start = Point2D::new(0.0_f64, 0.0_f64);
        let collinear_middle = Point2D::new(1.0_f64, 0.0_f64);
        let collinear_end = Point2D::new(2.0_f64, 0.0_f64);

        let invalid = Arc2D::from_three_points(collinear_start, collinear_middle, collinear_end);
        assert!(invalid.is_none());
    }

    #[test]
    fn test_basic_properties() {
        let center = Point2D::origin();
        let arc = Arc2D::xy_arc(center, 2.0_f64, angle(0.0), angle(std::f64::consts::PI)).unwrap();

        // 角度範囲
        let span = arc.angle_span();
        let expected_span = angle(std::f64::consts::PI);
        assert!(span.is_equivalent_default(&expected_span));

        // 円弧長
        let length = arc.arc_length();
        let expected_length = 2.0 * std::f64::consts::PI; // 半円
        assert!((length - expected_length).abs() < 1e-10);

        // 完全円判定
        assert!(!arc.is_full_circle());

        // 退化判定
        assert!(!arc.is_degenerate());
    }

    #[test]
    fn test_full_circle_detection() {
        let center = Point2D::origin();
        let full_arc = Arc2D::xy_arc(
            center,
            3.0_f64,
            angle(0.0),
            angle(2.0 * std::f64::consts::PI),
        )
        .unwrap();

        assert!(full_arc.is_full_circle());

        let arc_length = full_arc.arc_length();
        let expected_circumference = 2.0 * std::f64::consts::PI * 3.0;
        assert!((arc_length - expected_circumference).abs() < 1e-10);
    }
    #[test]
    fn test_degenerate_arc() {
        let center = Point2D::origin();

        // 非常に小さい半径
        let tiny_arc =
            Arc2D::xy_arc(center, 1e-12_f64, angle(0.0), angle(std::f64::consts::PI)).unwrap();
        assert!(tiny_arc.is_degenerate());

        // 非常に小さい角度範囲
        let narrow_arc = Arc2D::xy_arc(center, 5.0_f64, angle(0.0), angle(1e-12)).unwrap();
        assert!(narrow_arc.is_degenerate());
    }

    #[test]
    fn test_parametric_points() {
        let center = Point2D::origin();
        let arc = Arc2D::xy_arc(center, 4.0_f64, angle(0.0), angle(std::f64::consts::PI)).unwrap();

        // 開始点 (t=0)
        let start = arc.point_at_parameter(0.0);
        assert!((start.x() - 4.0_f64).abs() < 1e-10);
        assert!((start.y() - 0.0_f64).abs() < 1e-10);

        // 中点 (t=0.5)
        let mid = arc.point_at_parameter(0.5);
        assert!((mid.x() - 0.0_f64).abs() < 1e-10);
        assert!((mid.y() - 4.0_f64).abs() < 1e-10);

        // 終了点 (t=1.0)
        let end = arc.point_at_parameter(1.0);
        assert!((end.x() - (-4.0_f64)).abs() < 1e-10);
        assert!((end.y() - 0.0_f64).abs() < 1e-10);
    }

    #[test]
    fn test_endpoint_methods() {
        let center = Point2D::origin();
        let arc = Arc2D::xy_arc(
            center,
            3.0_f64,
            angle(0.0),
            angle(std::f64::consts::PI / 2.0),
        )
        .unwrap();

        // 開始点
        let start = arc.start_point();
        assert!((start.x() - 3.0_f64).abs() < 1e-10);
        assert!((start.y() - 0.0_f64).abs() < 1e-10);

        // 終了点
        let end = arc.end_point();
        assert!((end.x() - 0.0_f64).abs() < 1e-10);
        assert!((end.y() - 3.0_f64).abs() < 1e-10);

        // 中点
        let mid = arc.mid_point();
        let expected_mid_angle = std::f64::consts::PI / 4.0;
        let expected_x = 3.0 * expected_mid_angle.cos();
        let expected_y = 3.0 * expected_mid_angle.sin();
        assert!((mid.x() - expected_x).abs() < 1e-10);
        assert!((mid.y() - expected_y).abs() < 1e-10);
    }

    #[test]
    fn test_angle_containment() {
        let center = Point2D::origin();
        let arc = Arc2D::xy_arc(center, 2.0_f64, angle(0.0), angle(std::f64::consts::PI)).unwrap();

        // 範囲内の角度
        assert!(arc.contains_angle(angle(std::f64::consts::PI / 2.0)));
        assert!(arc.contains_angle(angle(0.0)));
        assert!(arc.contains_angle(angle(std::f64::consts::PI)));

        // 範囲外の角度
        assert!(!arc.contains_angle(angle(3.0 * std::f64::consts::PI / 2.0)));
        assert!(!arc.contains_angle(angle(-std::f64::consts::PI / 2.0)));
    }

    #[test]
    fn test_angle_span_calculation() {
        let center = Point2D::origin();

        // 正の角度範囲
        let arc1 = Arc2D::xy_arc(center, 2.0_f64, angle(0.0), angle(std::f64::consts::PI)).unwrap();
        let expected_span1 = angle(std::f64::consts::PI);
        assert!(arc1.angle_span().is_equivalent_default(&expected_span1));

        // 0度をまたぐ角度範囲
        let arc2 = Arc2D::xy_arc(
            center,
            2.0_f64,
            angle(3.0 * std::f64::consts::PI / 2.0),
            angle(std::f64::consts::PI / 2.0),
        )
        .unwrap();
        let expected_span2 = angle(std::f64::consts::PI);
        assert!(arc2.angle_span().is_equivalent_default(&expected_span2));
    }

    #[test]
    fn test_circle_conversion() {
        let center = Point2D::origin();

        // 完全円の場合は Circle2D に変換可能
        let full_arc = Arc2D::xy_arc(
            center,
            3.0_f64,
            angle(0.0),
            angle(2.0 * std::f64::consts::PI),
        )
        .unwrap();
        let circle = full_arc.to_circle().unwrap();
        assert_eq!(circle.center(), center);
        assert_eq!(circle.radius(), 3.0);

        // 部分円弧は変換不可
        let partial_arc =
            Arc2D::xy_arc(center, 3.0_f64, angle(0.0), angle(std::f64::consts::PI)).unwrap();
        assert!(partial_arc.to_circle().is_none());
    }

    #[test]
    fn test_f32_compatibility() {
        // f32での基本操作
        let center = Point2D::new(0.0f32, 0.0f32);
        let arc = Arc2D::xy_arc(
            center,
            3.0f32,
            angle_f32(0.0f32),
            angle_f32(std::f32::consts::PI),
        )
        .unwrap();

        assert_eq!(arc.radius(), 3.0f32);

        let length = arc.arc_length();
        let expected_length = 3.0f32 * std::f32::consts::PI;
        assert!((length - expected_length).abs() < 1e-6);

        let start = arc.start_point();
        assert!((start.x() - 3.0f32).abs() < 1e-6);
        assert!((start.y() - 0.0f32).abs() < 1e-6);
    }
}
