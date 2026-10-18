//! Ellipse3D の基本テスト
//!
//! 基本機能のみテスト：作成、アクセサ、基本プロパティ

use crate::{Direction3D, Ellipse3D, Point3D, Vector3D};
use geo_contracts::{
    default_distance_tolerance, Ellipse3DConstructor, Ellipse3DContainment, Ellipse3DDistance,
    Ellipse3DEvaluation, Ellipse3DProjection, Ellipse3DProperties, PointClassification,
};

use analysis::test_constants::{TOLERANCE_F32, TOLERANCE_F64};

#[test]
fn test_basic_creation() {
    // XY平面上の基本的な楕円作成
    let center = Point3D::new(1.0, 2.0, 3.0);
    let ellipse = Ellipse3D::new(
        center,
        5.0,
        3.0,
        Direction3D::positive_z(),
        Direction3D::positive_x(),
    )
    .unwrap();

    assert_eq!(ellipse.center(), center);
    assert_eq!(ellipse.semi_major_axis(), 5.0);
    assert_eq!(ellipse.semi_minor_axis(), 3.0);
    assert_eq!(
        ellipse.normal(),
        Direction3D::from_vector(Vector3D::unit_z()).unwrap()
    );
    assert_eq!(
        ellipse.major_axis_direction(),
        Direction3D::from_vector(Vector3D::unit_x()).unwrap()
    );

    // 不正な楕円（短軸が長軸より大きい）
    let invalid = Ellipse3D::new(
        center,
        3.0,
        5.0,
        Direction3D::positive_z(),
        Direction3D::positive_x(),
    );
    assert!(invalid.is_none());

    // 負の半軸
    let invalid2 = Ellipse3D::new(
        center,
        -1.0,
        2.0,
        Direction3D::positive_z(),
        Direction3D::positive_x(),
    );
    assert!(invalid2.is_none());
}

#[test]
fn test_axis_directions() {
    let ellipse = Ellipse3D::new(
        Point3D::origin(),
        4.0,
        2.0,
        Direction3D::positive_z(),
        Direction3D::positive_x(),
    )
    .unwrap();

    // 長軸方向
    let major_dir = ellipse.major_axis_direction();
    assert_eq!(
        major_dir,
        Direction3D::from_vector(Vector3D::unit_x()).unwrap()
    );

    // 短軸方向
    let minor_dir = ellipse.minor_axis_direction();
    assert_eq!(
        minor_dir,
        Direction3D::from_vector(Vector3D::unit_y()).unwrap()
    );

    // 基本的な直交性確認
    let dot1: f64 = major_dir.dot(minor_dir);
    let dot2: f64 = major_dir.dot(ellipse.normal());
    let dot3: f64 = minor_dir.dot(ellipse.normal());
    assert!(dot1.abs() < TOLERANCE_F64);
    assert!(dot2.abs() < TOLERANCE_F64);
    assert!(dot3.abs() < TOLERANCE_F64);
}

#[test]
fn test_simple_parametric() {
    let ellipse = Ellipse3D::new(
        Point3D::origin(),
        4.0,
        2.0,
        Direction3D::positive_z(),
        Direction3D::positive_x(),
    )
    .unwrap();

    // 基本的なパラメータでの点
    let point_0 = ellipse.point_at_parameter(0.0);
    assert!((point_0.x() - 4.0f64).abs() < TOLERANCE_F64);
    assert!((point_0.y() - 0.0f64).abs() < TOLERANCE_F64);
    assert!((point_0.z() - 0.0f64).abs() < TOLERANCE_F64);

    let point_pi2 = ellipse.point_at_parameter(std::f64::consts::PI / 2.0);
    assert!((point_pi2.x() - 0.0f64).abs() < TOLERANCE_F64);
    assert!((point_pi2.y() - 2.0f64).abs() < TOLERANCE_F64);
    assert!((point_pi2.z() - 0.0f64).abs() < TOLERANCE_F64);
}

#[test]
fn test_f32_compatibility() {
    // f32での基本操作
    let ellipse = Ellipse3D::new(
        Point3D::new(0.0f32, 0.0f32, 0.0f32),
        3.0f32,
        2.0f32,
        Direction3D::positive_z(),
        Direction3D::positive_x(),
    )
    .unwrap();

    assert_eq!(ellipse.semi_major_axis(), 3.0f32);
    assert_eq!(ellipse.semi_minor_axis(), 2.0f32);
    assert!(ellipse.distance_to_point(Point3D::new(3.0f32, 0.0, 0.0)) < TOLERANCE_F32);
}

#[test]
fn test_closest_point_projection() {
    let ellipse = Ellipse3D::new(
        Point3D::origin(),
        4.0,
        2.0,
        Direction3D::positive_z(),
        Direction3D::positive_x(),
    )
    .unwrap();
    let closest = ellipse.closest_point(Point3D::new(10.0, 0.0, 3.0));

    assert!(closest.distance_to(Point3D::new(4.0, 0.0, 0.0)) < TOLERANCE_F64);
}

#[test]
fn test_distance_and_closest_point_to_curve() {
    // XY 平面上、長半軸 2（X 軸方向）・短半軸 1 の楕円
    let ellipse = Ellipse3D::new(
        Point3D::new(0.0_f64, 0.0, 0.0),
        2.0,
        1.0,
        Direction3D::positive_z(),
        Direction3D::positive_x(),
    )
    .unwrap();

    // 中心から楕円までの距離は短半軸の長さ
    let center_distance = ellipse.distance_to_point(Point3D::new(0.0, 0.0, 0.0));
    assert!((center_distance - 1.0).abs() < TOLERANCE_F64);

    // 平面内部の点。距離と最近点は曲線のパラメータ方程式の数値解で求めた値
    let inside_distance = ellipse.distance_to_point(Point3D::new(1.0, 0.5, 0.0));
    assert!((inside_distance - 0.349_605_694_569_673).abs() < TOLERANCE_F64);

    // 平面外の点は、平面へ投影した点の最近点を最近点とする
    let closest = ellipse.closest_point(Point3D::new(1.0, 0.5, 3.0));
    assert!(
        closest.distance_to(Point3D::new(
            1.110_726_977_965_884,
            0.831_607_717_078_608,
            0.0
        )) < TOLERANCE_F64
    );
    let off_plane_distance = ellipse.distance_to_point(Point3D::new(1.0, 0.5, 3.0));
    let expected = (0.349_605_694_569_673_f64.powi(2) + 9.0).sqrt();
    assert!((off_plane_distance - expected).abs() < TOLERANCE_F64);
}

#[test]
fn test_contains_and_classify_point() {
    // XY 平面上、長半軸 2（X 軸方向）・短半軸 1 の楕円
    let ellipse = Ellipse3D::new(
        Point3D::new(0.0_f64, 0.0, 0.0),
        2.0,
        1.0,
        Direction3D::positive_z(),
        Direction3D::positive_x(),
    )
    .unwrap();
    let tolerance = default_distance_tolerance::<f64>();

    // contains_point は楕円上にあるかを判定する（同名で領域を判定するメソッドはない）
    assert!(ellipse.contains_point(Point3D::new(2.0, 0.0, 0.0)));
    assert!(!ellipse.contains_point(Point3D::new(0.0, 0.0, 0.0)));

    let classify = |x: f64, y: f64, z: f64| ellipse.classify_point(Point3D::new(x, y, z));
    assert_eq!(classify(0.0, 0.0, 0.0), PointClassification::Inside);
    assert_eq!(classify(0.0, 1.0, 0.0), PointClassification::OnBoundary);
    assert_eq!(classify(0.0, 1.5, 0.0), PointClassification::Outside);

    // 平面外の点
    assert_eq!(
        classify(2.0, 0.0, 0.5 * tolerance),
        PointClassification::OnBoundary
    );
    assert_eq!(
        classify(0.0, 0.0, 0.5 * tolerance),
        PointClassification::Inside
    );
    assert_eq!(
        classify(0.0, 0.0, 2.0 * tolerance),
        PointClassification::Outside
    );

    assert!(ellipse.contains_point(Point3D::new(-2.0, 0.0, 0.0)));
    assert_eq!(
        ellipse.classify_point(Point3D::new(0.5, 0.0, 0.0)),
        PointClassification::Inside
    );
}

#[test]
fn test_parameter_for_point_returns_parameter_angle_of_closest_point() {
    // XY 平面上、長半軸 2（X 軸方向）・短半軸 1 の楕円
    let ellipse = Ellipse3D::new(
        Point3D::new(0.0_f64, 0.0, 0.0),
        2.0,
        1.0,
        Direction3D::positive_z(),
        Direction3D::positive_x(),
    )
    .unwrap();

    // 楕円上の点は、その点を与えるパラメータ角（中心から見た方向の角度ではない）
    let t = std::f64::consts::FRAC_PI_4;
    let on_ellipse = ellipse.point_at_parameter(t);
    assert!((ellipse.parameter_for_point(on_ellipse) - t).abs() < TOLERANCE_F64);

    // 楕円の内部・平面外の点は、最近点のパラメータ角
    for point in [Point3D::new(1.0, 0.5, 0.0), Point3D::new(-1.5, -0.2, 2.0)] {
        let parameter = ellipse.parameter_for_point(point);
        let closest = ellipse.closest_point(point);
        assert!(ellipse.point_at_parameter(parameter).distance_to(closest) < TOLERANCE_F64);
        assert!((0.0..std::f64::consts::TAU).contains(&parameter));
    }
}
