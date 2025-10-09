//! Arc tests - 完全分離型テスト構造
//!
//! arc.rs から分離されたテストコード。
//! test_utils.rs を活用した統一されたテスト環境。

#[cfg(test)]
mod tests {
    // テストユーティリティを使用（統一された型とヘルパー）
    use crate::unit_tests::test_utils::helpers::*;
    use crate::unit_tests::test_utils::*;

    // マクロのインポート
    use crate::assert_approx_eq;

    // 必要な型のみを明示的にインポート
    use crate::geometry2d::{arc::ArcKind, Arc};
    use crate::traits::Circle2D; // Circle の contains_point メソッド用
    use geo_foundation::PI;
    use geo_foundation::Angle;

    #[test]
    fn test_arc_creation() {
        let center = TestPoint2D::new(0.0, 0.0);
        let circle = TestCircle::new(center, 5.0);
        let arc = Arc::from_radians(circle, 0.0, PI);

        assert_approx_eq!(arc.center().x(), 0.0);
        assert_approx_eq!(arc.center().y(), 0.0);
        assert_approx_eq!(arc.radius(), 5.0);
        assert_approx_eq!(arc.start_angle().to_radians(), 0.0);
        assert_approx_eq!(arc.end_angle().to_radians(), PI);
    }

    #[test]
    fn test_arc_points() {
        let center = TestPoint2D::new(2.0, 3.0);
        let circle = TestCircle::new(center, 4.0);
        let arc = Arc::from_radians(circle, 0.0, PI / 2.0);

        let start = arc.start_point();
        let end = arc.end_point();

        assert!(approx_eq_f64(start.x(), 6.0));
        assert!(approx_eq_f64(start.y(), 3.0));
        assert!(approx_eq_f64(end.x(), 2.0));
        assert!(approx_eq_f64(end.y(), 7.0));
    }

    #[test]
    fn test_arc_length() {
        let center = TestPoint2D::new(0.0, 0.0);
        let circle = TestCircle::new(center, 3.0);
        let arc = Arc::from_radians(circle, 0.0, PI);

        let expected_length = 3.0 * PI;
        assert_approx_eq!(arc.arc_length(), expected_length);
    }

    #[test]
    fn test_arc_kind() {
        let center = TestPoint2D::new(0.0, 0.0);
        let circle = TestCircle::new(center, 1.0);

        let minor_arc = Arc::from_radians(circle.clone(), 0.0, PI / 3.0);
        assert_eq!(minor_arc.arc_kind(), ArcKind::MinorArc);

        let major_arc = Arc::from_radians(circle.clone(), 0.0, 4.0 * PI / 3.0);
        assert_eq!(major_arc.arc_kind(), ArcKind::MajorArc);

        let semicircle = Arc::from_radians(circle.clone(), 0.0, PI);
        assert_eq!(semicircle.arc_kind(), ArcKind::Semicircle);

        let full_circle = Arc::from_radians(circle, 0.0, 2.0 * PI);
        assert_eq!(full_circle.arc_kind(), ArcKind::FullCircle);
    }

    #[test]
    fn test_from_three_points() {
        let p1 = TestPoint2D::new(1.0, 0.0);
        let p2 = TestPoint2D::new(0.0, 1.0);
        let p3 = TestPoint2D::new(-1.0, 0.0);

        let arc = Arc::from_three_points(p1, p2, p3).expect("円弧作成に失敗");

        assert!(approx_eq_f64(arc.center().x(), 0.0));
        assert!(approx_eq_f64(arc.center().y(), 0.0));
        assert!(approx_eq_f64(arc.radius(), 1.0));
    }

    #[test]
    fn test_angle_contains() {
        let center = TestPoint2D::new(0.0, 0.0);
        let circle = TestCircle::new(center, 1.0);
        let arc = Arc::from_radians(circle, 0.0, PI);

        assert!(arc.angle_contains(Angle::from_radians(PI / 4.0)));
        assert!(arc.angle_contains(Angle::from_radians(PI / 2.0)));
        assert!(!arc.angle_contains(Angle::from_radians(3.0 * PI / 2.0)));
    }

    #[test]
    fn test_contains_point() {
        let center = TestPoint2D::new(0.0, 0.0);
        let circle = TestCircle::new(center, 1.0);
        let arc = Arc::from_radians(circle, 0.0, PI);

        // 円弧上の点（開始点）
        let point_on_arc_start = TestPoint2D::new(1.0, 0.0);
        assert!(arc.contains_point(point_on_arc_start));

        // 円弧上の点（終了点）
        let point_on_arc_end = TestPoint2D::new(-1.0, 0.0);
        assert!(arc.contains_point(point_on_arc_end));

        // 円上だが角度範囲外の点（270度の位置）
        let point_off_arc = TestPoint2D::new(0.0, -1.0);
        assert!(!arc.contains_point(point_off_arc));

        // 円外の点
        let point_outside = TestPoint2D::new(2.0, 0.0);
        assert!(!arc.contains_point(point_outside));
    }

    #[test]
    fn test_arc_reverse() {
        let center = TestPoint2D::new(0.0, 0.0);
        let circle = TestCircle::new(center, 1.0);
        let arc = Arc::from_radians(circle, 0.0, PI / 2.0);
        let reversed = arc.reverse();

        assert_approx_eq!(
            reversed.start_angle().to_radians(),
            arc.end_angle().to_radians()
        );
        assert_approx_eq!(
            reversed.end_angle().to_radians(),
            arc.start_angle().to_radians()
        );
    }

    #[test]
    fn test_arc_midpoint() {
        let center = TestPoint2D::new(0.0, 0.0);
        let circle = TestCircle::new(center, 1.0);
        let arc = Arc::from_radians(circle, 0.0, PI / 2.0);
        let mid = arc.midpoint();

        let expected_angle = PI / 4.0;
        let expected_x = expected_angle.cos();
        let expected_y = expected_angle.sin();

        assert!(approx_eq_f64(mid.x(), expected_x));
        assert!(approx_eq_f64(mid.y(), expected_y));
    }

    #[test]
    fn test_approximate_with_points() {
        let center = TestPoint2D::new(0.0, 0.0);
        let circle = TestCircle::new(center, 1.0);
        let arc = Arc::from_radians(circle, 0.0, PI / 2.0);

        let points = arc.approximate_with_points(4);
        assert_eq!(points.len(), 5); // 4セグメント = 5点

        // 最初と最後の点をチェック
        let first_point = points.first().unwrap();
        let last_point = points.last().unwrap();

        assert!(approx_eq_f64(first_point.x(), 1.0));
        assert!(approx_eq_f64(first_point.y(), 0.0));
        assert!(approx_eq_f64(last_point.x(), 0.0));
        assert!(approx_eq_f64(last_point.y(), 1.0));
    }

    #[test]
    fn test_arc_integration_with_circles() {
        // 円弧と円の統合テスト（test_utils のおかげで簡単）
        let circles = test_circles();
        let unit_circle = &circles[0]; // test_utils の unit_circle

        // 半円弧を作成
        let semicircle = Arc::from_radians(unit_circle.clone(), 0.0, PI);

        // 半円弧の特性を確認
        assert_eq!(semicircle.arc_kind(), ArcKind::Semicircle);
        assert_approx_eq!(semicircle.arc_length(), PI); // 半径1なので弧長はπ

        // 開始点と終了点が円周上にあることを確認
        assert!(unit_circle.contains_point(&semicircle.start_point()));
        assert!(unit_circle.contains_point(&semicircle.end_point()));
    }

    #[test]
    fn test_arc_with_test_helpers() {
        // test_utils のヘルパー関数を活用
        let points = test_points_2d();
        let center = points[0]; // 原点

        let circle = TestCircle::new(center, 2.0);
        let arc = Arc::from_radians(circle, 0.0, PI / 3.0);

        // 基本プロパティの確認
        assert!(arc.arc_length() > 0.0);
        assert!(arc.radius() > 0.0);
        assert_eq!(arc.arc_kind(), ArcKind::MinorArc);

        // 角度範囲の確認
        assert!(arc.start_angle().to_radians() < arc.end_angle().to_radians());
    }
}
