//! Ellipse2D のテスト

use crate::{Ellipse2D, Point2D};
use geo_contracts::Scalar;

use analysis::test_constants::{TOLERANCE_F32, TOLERANCE_F64};

#[test]
fn test_ellipse_creation() {
    // 基本的な楕円作成
    let center = Point2D::new(2.0, 3.0);
    let ellipse = Ellipse2D::new(center, 5.0, 3.0, 0.0).unwrap();

    assert_eq!(ellipse.center, center);
    assert_eq!(ellipse.semi_major, 5.0);
    assert_eq!(ellipse.semi_minor, 3.0);
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
    assert_eq!(ellipse.semi_major, 4.0);
    assert_eq!(ellipse.semi_minor, 2.0);
}

#[test]
fn test_ellipse_properties() {
    let ellipse = Ellipse2D::new(Point2D::origin(), 5.0, 3.0, 0.0).unwrap();

    // 面積
    let area = ellipse.area();
    let expected_area = std::f64::consts::PI * 5.0 * 3.0;
    assert!((area - expected_area).abs() < TOLERANCE_F64);

    // 円判定
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
    let center = ellipse.center;
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

    // Ellipse の point_at_parameter は local angle domain を使う

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

    assert_eq!(ellipse.semi_major, 3.0f32);
    assert_eq!(ellipse.semi_minor, 2.0f32);

    let area = ellipse.area();
    let expected_area = std::f32::consts::PI * 3.0f32 * 2.0f32;
    assert!((area - expected_area).abs() < TOLERANCE_F32);

    // foundation トレイト
    let bbox = ellipse.bounding_box();
    assert_eq!(bbox.min_point(), Point2D::new(-3.0f32, -2.0f32));
    assert_eq!(bbox.max_point(), Point2D::new(3.0f32, 2.0f32));
}

#[test]
fn test_basic_ellipse_creation() {
    let center = Point2D::new(0.0, 0.0);
    let ellipse = Ellipse2D::new(center, 5.0, 3.0, 0.0).expect("楕円の作成に失敗");

    assert_eq!(ellipse.center, center);
    assert_eq!(ellipse.semi_major, 5.0);
    assert_eq!(ellipse.semi_minor, 3.0);
    assert_eq!(ellipse.rotation(), 0.0);
}

#[test]
fn test_ellipse_area_calculation() {
    let center = Point2D::new(0.0, 0.0);
    let ellipse = Ellipse2D::new(center, 4.0, 2.0, 0.0).expect("楕円の作成に失敗");

    let area = ellipse.area();
    let expected = std::f64::consts::PI * 4.0 * 2.0; // π * a * b
    assert!((area - expected).abs() < TOLERANCE_F64);
}

#[test]
fn test_ellipse_point_evaluation() {
    let center = Point2D::new(0.0, 0.0);
    let ellipse = Ellipse2D::new(center, 4.0, 2.0, 0.0).expect("楕円の作成に失敗");

    // Ellipse core parameter は local angle domain (0..2π)
    // t=0での点（長軸の正の端）
    let point_0 = ellipse.point_at_parameter(0.0);
    assert!((point_0.x() - 4.0_f64).abs() < TOLERANCE_F64);
    assert!(point_0.y().abs() < TOLERANCE_F64);

    // t=π/2での点（短軸の正の端）
    let point_pi_2 = ellipse.point_at_parameter(std::f64::consts::PI / 2.0);
    assert!(point_pi_2.x().abs() < TOLERANCE_F64);
    assert!((point_pi_2.y() - 2.0_f64).abs() < TOLERANCE_F64);
}

#[test]
fn test_ellipse_contains_point() {
    let center = Point2D::new(0.0, 0.0);
    let ellipse = Ellipse2D::new(center, 4.0, 2.0, 0.0).expect("楕円の作成に失敗");
    let tolerance = TOLERANCE_F64;

    // 楕円上の点
    assert!(ellipse.contains_point(&Point2D::new(4.0, 0.0), tolerance));
    assert!(ellipse.contains_point(&Point2D::new(0.0, -2.0), tolerance));

    // 楕円が囲む領域の内部の点は楕円上にない
    assert!(!ellipse.contains_point(&center, tolerance));
    assert!(!ellipse.contains_point(&Point2D::new(1.0, 0.5), tolerance));

    // 外部の点
    let outside = Point2D::new(5.0, 3.0);
    assert!(!ellipse.contains_point(&outside, tolerance));
}

#[test]
fn test_ellipse_bounding_box() {
    let center = Point2D::new(1.0, 2.0);
    let ellipse = Ellipse2D::new(center, 4.0, 2.0, 0.0).expect("楕円の作成に失敗");

    let bbox = ellipse.bounding_box();

    // 境界ボックスに中心が含まれる
    assert!(bbox.contains_point(center));
}

#[test]
fn test_ellipse_distance_and_closest_point_to_curve() {
    // 長半軸 2・短半軸 1 の楕円を 30° 回転し、中心を (1, -1) に置く
    let rotation = std::f64::consts::FRAC_PI_6;
    let center = Point2D::new(1.0, -1.0);
    let ellipse = Ellipse2D::new(center, 2.0, 1.0, rotation).expect("楕円の作成に失敗");
    let to_world = |x: f64, y: f64| {
        Point2D::new(
            center.x() + x * rotation.cos() - y * rotation.sin(),
            center.y() + x * rotation.sin() + y * rotation.cos(),
        )
    };

    // 中心から楕円までの距離は短半軸の長さ
    assert!((ellipse.distance_to_point(&center) - 1.0).abs() < TOLERANCE_F64);

    // 局所座標 (1, 0.5) の内部の点。距離と最近点は曲線のパラメータ方程式の数値解で求めた値
    let inside = to_world(1.0, 0.5);
    assert!((ellipse.distance_to_point(&inside) - 0.349_605_694_569_673).abs() < TOLERANCE_F64);
    let closest = ellipse.closest_point(&inside);
    let expected = to_world(1.110_726_977_965_884, 0.831_607_717_078_608);
    assert!(closest.distance_to(expected) < TOLERANCE_F64);
    assert!(
        (closest.distance_to(inside) - ellipse.distance_to_point(&inside)).abs() < TOLERANCE_F64
    );
}

#[test]
fn test_ellipse_classify_point() {
    use geo_contracts::{Ellipse2DContainment, PointClassification};

    // 長半軸 4・短半軸 2 の楕円を 90° 回転（長軸が Y 軸方向）
    let ellipse = Ellipse2D::new(
        Point2D::new(0.0, 0.0),
        4.0,
        2.0,
        std::f64::consts::FRAC_PI_2,
    )
    .expect("楕円の作成に失敗");
    let tolerance = 1e-3;
    let classify = |x: f64, y: f64| ellipse.classify_point(&Point2D::new(x, y), tolerance);

    assert_eq!(classify(0.0, 0.0), PointClassification::Inside);
    assert_eq!(classify(0.0, 4.0), PointClassification::OnBoundary);
    assert_eq!(classify(-2.0, 0.0), PointClassification::OnBoundary);
    assert_eq!(classify(3.0, 0.0), PointClassification::Outside);

    // 許容誤差の境目
    assert_eq!(
        classify(0.0, 4.0 - 0.5 * tolerance),
        PointClassification::OnBoundary
    );
    assert_eq!(
        classify(0.0, 4.0 + 0.5 * tolerance),
        PointClassification::OnBoundary
    );
    assert_eq!(
        classify(0.0, 4.0 - 2.0 * tolerance),
        PointClassification::Inside
    );
    assert_eq!(
        classify(0.0, 4.0 + 2.0 * tolerance),
        PointClassification::Outside
    );

    // classify_point が OnBoundary のときだけ contains_point が真
    for (x, y) in [
        (0.0, 0.0),
        (0.0, 4.0),
        (3.0, 0.0),
        (0.0, 4.0 + 2.0 * tolerance),
    ] {
        let point = Point2D::new(x, y);
        assert_eq!(
            ellipse.contains_point(&point, tolerance),
            ellipse.classify_point(&point, tolerance) == PointClassification::OnBoundary
        );
    }

    // trait定義は既定の距離トレランスで判定する
    assert!(Ellipse2DContainment::contains_point(&ellipse, (0.0, -4.0)));
    assert_eq!(
        Ellipse2DContainment::classify_point(&ellipse, (1.0, 1.0)),
        PointClassification::Inside
    );
}
