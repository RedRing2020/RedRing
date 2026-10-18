//! Circle3D のテスト

use crate::{Circle3D, Direction3D, Point3D, Vector3D};
use analysis::test_constants::TOLERANCE_F64;
use geo_contracts::{
    default_distance_tolerance, Bounded, Circle3DConstructor, Circle3DContainment,
    Circle3DDistance, Circle3DEvaluation, Circle3DProjection, Circle3DProperties,
    PointClassification, PrimitiveKind, PrimitiveMetadata, TolerantEq,
};

// 近似等価性チェック用のヘルパー関数
fn assert_approx_eq(a: f64, b: f64, epsilon: f64) {
    assert!((a - b).abs() < epsilon, "Expected {}, got {}", b, a);
}

#[test]
fn test_circle_creation() {
    // 基本的な円の作成
    let center = Point3D::new(1.0, 2.0, 3.0);
    let normal = Direction3D::positive_z();
    let radius = 5.0;

    let circle = Circle3D::new(center, normal, radius).unwrap();
    assert_eq!(circle.center(), center);
    assert_eq!(circle.axis(), normal);
    assert_eq!(circle.radius(), radius);

    // 参照方向は法線に直交する
    assert_approx_eq(circle.ref_direction().dot(normal), 0.0, TOLERANCE_F64);

    // 無効な半径での作成失敗
    assert!(Circle3D::new(center, normal, 0.0).is_none());
    assert!(Circle3D::new(center, normal, -1.0).is_none());
}

#[test]
fn test_plane_constructors() {
    let center = Point3D::new(1.0, 2.0, 3.0);
    let radius = 4.0;

    // XY平面の円
    let xy_circle = Circle3D::new_xy_plane(center, radius).unwrap();
    assert_eq!(xy_circle.axis(), Direction3D::positive_z());
}

#[test]
fn test_point_at_parameter_uses_local_angle_parameter() {
    let center = Point3D::new(0.0, 0.0, 0.0);
    let normal = Direction3D::positive_z();
    let ref_direction = Direction3D::positive_x();
    let radius = 2.0;
    let circle = Circle3D::new_with_ref_direction(center, normal, ref_direction, radius).unwrap();

    for (t, expected) in [
        // 角度0での点（参照方向）
        (0.0, Point3D::new(2.0, 0.0, 0.0)),
        // 角度π/2での点（法線 × 参照方向）
        (std::f64::consts::PI / 2.0, Point3D::new(0.0, 2.0, 0.0)),
        // t = 2π は周期端として t = 0 と同じ点
        (std::f64::consts::TAU, Point3D::new(2.0, 0.0, 0.0)),
    ] {
        assert!(circle.point_at_parameter(t).distance_to(expected) < TOLERANCE_F64);
    }
}

#[test]
fn test_contains_and_classify_point() {
    // XY 平面上、中心 (1, 2, 3)・半径 2 の円
    let circle = Circle3D::new(
        Point3D::new(1.0_f64, 2.0, 3.0),
        Direction3D::positive_z(),
        2.0,
    )
    .unwrap();
    let tolerance = default_distance_tolerance::<f64>();

    // contains_point は円周上にあるかを判定する
    assert!(circle.contains_point(Point3D::new(3.0, 2.0, 3.0)));
    assert!(!circle.contains_point(Point3D::new(1.0, 2.0, 3.0)));

    let classify = |x: f64, y: f64, z: f64| circle.classify_point(Point3D::new(x, y, z));
    assert_eq!(classify(1.0, 2.0, 3.0), PointClassification::Inside);
    assert_eq!(classify(3.0, 2.0, 3.0), PointClassification::OnBoundary);
    assert_eq!(classify(4.0, 2.0, 3.0), PointClassification::Outside);

    // 円周までの距離が許容誤差以内なら、平面外でも OnBoundary
    assert_eq!(
        classify(3.0, 2.0, 3.0 + 0.5 * tolerance),
        PointClassification::OnBoundary
    );
    assert_eq!(
        classify(3.0 - 0.5 * tolerance, 2.0, 3.0),
        PointClassification::OnBoundary
    );

    // 平面からの距離が許容誤差を超える点は、円の内側の上方でも Outside
    assert_eq!(
        classify(1.0, 2.0, 3.0 + 2.0 * tolerance),
        PointClassification::Outside
    );
    assert_eq!(
        classify(1.0, 2.0, 3.0 + 0.5 * tolerance),
        PointClassification::Inside
    );
}

#[test]
fn test_distance_and_closest_point() {
    let circle = Circle3D::new_xy_plane(Point3D::new(0.0_f64, 0.0, 0.0), 2.0).unwrap();

    // 平面からの距離と平面上での円周までの距離を合成する
    assert_approx_eq(
        circle.distance_to_point(Point3D::new(5.0, 0.0, 4.0)),
        5.0,
        TOLERANCE_F64,
    );

    assert!(
        circle
            .closest_point(Point3D::new(0.0, 5.0, 3.0))
            .distance_to(Point3D::new(0.0, 2.0, 0.0))
            < TOLERANCE_F64
    );

    // 中心軸上の点は参照方向の点を最近点とする
    let on_axis = circle.closest_point(Point3D::new(0.0, 0.0, 1.0));
    assert!(on_axis.distance_to(circle.point_at_parameter(0.0)) < TOLERANCE_F64);
}

#[test]
fn test_from_three_points_passes_through_all_points() {
    // 中心 (0, 0, 0)・半径 1 の XY 平面上の円
    let circle = Circle3D::from_three_points(
        Point3D::new(1.0_f64, 0.0, 0.0),
        Point3D::new(0.0, 1.0, 0.0),
        Point3D::new(-1.0, 0.0, 0.0),
    )
    .unwrap();
    assert!(circle.center().distance_to(Point3D::new(0.0, 0.0, 0.0)) < TOLERANCE_F64);
    assert!((circle.radius() - 1.0).abs() < TOLERANCE_F64);

    // 傾いた平面上の 3 点のいずれも円上にある
    let points = [
        Point3D::new(1.0, 2.0, 3.0),
        Point3D::new(4.0, -1.0, 2.0),
        Point3D::new(0.0, 5.0, -1.0),
    ];
    let circle = Circle3D::from_three_points(points[0], points[1], points[2]).unwrap();
    for point in points {
        assert!(circle.distance_to_point(point) <= TOLERANCE_F64);
    }

    // 一直線上の 3 点は円を作れない
    assert!(Circle3D::from_three_points(
        Point3D::new(0.0, 0.0, 0.0),
        Point3D::new(1.0, 1.0, 1.0),
        Point3D::new(2.0, 2.0, 2.0),
    )
    .is_none());
}

#[test]
fn test_tolerant_eq() {
    let center = Point3D::new(0.0, 0.0, 0.0);
    let normal = Direction3D::from_vector(Vector3D::new(0.0, 0.0, 1.0)).unwrap();

    let circle1 = Circle3D::new(center, normal, 5.0).unwrap();
    let circle2 = Circle3D::new(center, normal, 5.0).unwrap();
    let circle3 = Circle3D::new(Point3D::new(2.0, 0.0, 0.0), normal, 5.0).unwrap();

    assert!(circle1.tolerant_eq(&circle2, 0.01));
    assert!(!circle1.tolerant_eq(&circle3, 0.01));
}

#[test]
fn test_metadata_and_bounds_capabilities() {
    let center = Point3D::new(1.0_f64, 2.0, 3.0);
    let circle = Circle3D::new(center, Direction3D::positive_z(), 5.0).unwrap();

    assert_eq!(circle.primitive_kind(), PrimitiveKind::Circle);

    let aabb = circle.aabb().expect("Circle should have an AABB");
    // XY平面の円なので、Z方向の範囲は0
    assert!((aabb.min().z() - center.z()).abs() < TOLERANCE_F64);
    assert!((aabb.max().z() - center.z()).abs() < TOLERANCE_F64);
    assert!((aabb.min().x() - (center.x() - 5.0)).abs() < TOLERANCE_F64);
    assert!((aabb.max().y() - (center.y() + 5.0)).abs() < TOLERANCE_F64);
}

#[test]
fn test_circle_3d_aabb_respects_plane_orientation() {
    let center = Point3D::new(1.0_f64, 2.0, 3.0);
    let circle = Circle3D::new(center, Direction3D::positive_x(), 2.0).unwrap();

    let aabb = circle.aabb().expect("Circle should have an AABB");
    assert!((aabb.min().x() - center.x()).abs() < TOLERANCE_F64);
    assert!((aabb.max().x() - center.x()).abs() < TOLERANCE_F64);
    assert!((aabb.min().y() - (center.y() - 2.0)).abs() < TOLERANCE_F64);
    assert!((aabb.max().z() - (center.z() + 2.0)).abs() < TOLERANCE_F64);
}
