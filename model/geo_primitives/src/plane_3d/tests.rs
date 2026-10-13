//! Plane3D テストファイル

use crate::{Direction3D, Plane3D, Point3D};
use geo_contracts::{
    Plane3DConstructor, Plane3DContainment, Plane3DDistance, Plane3DProjection, Plane3DProperties,
    PrimitiveKind, PrimitiveMetadata, TolerantEq,
};

use analysis::test_constants::TOLERANCE_F64;
use approx::{assert_abs_diff_eq, assert_relative_eq};

fn dir(x: f64, y: f64, z: f64) -> Direction3D<f64> {
    Direction3D::new(x, y, z).unwrap()
}

#[test]
fn test_from_point_and_normal() {
    let point = Point3D::new(1.0, 2.0, 3.0);
    let normal = dir(0.0, 0.0, 1.0);

    let plane = Plane3D::from_point_and_normal(point, normal).unwrap();

    assert_eq!(plane.origin(), point);
    assert_relative_eq!(plane.normal().x(), 0.0);
    assert_relative_eq!(plane.normal().y(), 0.0);
    assert_relative_eq!(plane.normal().z(), 1.0);
}

#[test]
fn test_from_point_and_normal_normalization() {
    let point = Point3D::new(0.0, 0.0, 0.0);
    let normal = dir(3.0, 4.0, 0.0); // length = 5.0

    let plane = Plane3D::from_point_and_normal(point, normal).unwrap();

    // 法線ベクトルが正規化されているかチェック
    assert_relative_eq!(plane.normal().length(), 1.0, epsilon = 1e-10);
    assert_relative_eq!(plane.normal().x(), 0.6); // 3/5
    assert_relative_eq!(plane.normal().y(), 0.8); // 4/5
    assert_relative_eq!(plane.normal().z(), 0.0);
}

#[test]
fn test_from_three_points() {
    let p1 = Point3D::new(0.0, 0.0, 0.0);
    let p2 = Point3D::new(1.0, 0.0, 0.0);
    let p3 = Point3D::new(0.0, 1.0, 0.0);

    let plane = Plane3D::from_three_points(p1, p2, p3).unwrap();

    // XY平面になるはず
    assert_relative_eq!(plane.normal().x(), 0.0, epsilon = 1e-10);
    assert_relative_eq!(plane.normal().y(), 0.0, epsilon = 1e-10);
    assert_relative_eq!(plane.normal().z(), 1.0, epsilon = 1e-10);
}

#[test]
fn test_from_three_points_collinear() {
    let p1 = Point3D::new(0.0, 0.0, 0.0);
    let p2 = Point3D::new(1.0, 0.0, 0.0);
    let p3 = Point3D::new(2.0, 0.0, 0.0); // 一直線上

    let result = Plane3D::from_three_points(p1, p2, p3);

    assert!(result.is_none());
}

#[test]
fn test_coordinate_planes() {
    // XY平面
    let plane =
        Plane3D::from_point_and_normal(Point3D::new(0.0, 0.0, 5.0), Direction3D::positive_z())
            .unwrap();
    assert_eq!(plane.origin(), Point3D::new(0.0, 0.0, 5.0));
    assert_eq!(plane.normal(), Direction3D::positive_z());
}

#[test]
fn test_contains_point() {
    let plane =
        Plane3D::from_point_and_normal(Point3D::new(0.0, 0.0, 5.0), Direction3D::positive_z())
            .unwrap();

    // 平面上の点
    assert!(plane.contains_point(Point3D::new(1.0, 2.0, 5.0)));
    assert!(plane.contains_point(Point3D::new(-3.0, 7.0, 5.0)));

    // 平面外の点
    assert!(!plane.contains_point(Point3D::new(1.0, 2.0, 5.1)));
    assert!(!plane.contains_point(Point3D::new(1.0, 2.0, 4.9)));
}

#[test]
fn test_distance_to_point() {
    let plane =
        Plane3D::from_point_and_normal(Point3D::new(0.0, 0.0, 3.0), Direction3D::positive_z())
            .unwrap();

    // 平面上の点
    assert_relative_eq!(plane.distance_to_point(Point3D::new(1.0, 2.0, 3.0)), 0.0);

    // 法線方向の点（正の距離）
    assert_relative_eq!(plane.distance_to_point(Point3D::new(1.0, 2.0, 5.0)), 2.0);

    // 法線逆方向の点（負の距離）
    assert_relative_eq!(plane.distance_to_point(Point3D::new(1.0, 2.0, 1.0)), -2.0);
}

#[test]
fn test_project_point() {
    let plane =
        Plane3D::from_point_and_normal(Point3D::new(0.0, 0.0, 2.0), Direction3D::positive_z())
            .unwrap();

    let point = Point3D::new(3.0, 4.0, 7.0);
    let projected = plane.project_point(point);

    assert_relative_eq!(projected.x(), 3.0);
    assert_relative_eq!(projected.y(), 4.0);
    assert_relative_eq!(projected.z(), 2.0);

    // 投影点は平面上にあるはず
    assert!(plane.contains_point(projected));
}

#[test]
fn test_display() {
    let plane =
        Plane3D::from_point_and_normal(Point3D::new(0.0, 0.0, 1.0), Direction3D::positive_z())
            .unwrap();
    let display_str = format!("{}", plane);

    assert!(display_str.contains("Plane3D"));
    assert!(display_str.contains("origin"));
    assert!(display_str.contains("normal"));
}

#[test]
fn test_step_plane_creation() {
    // XY平面をZ軸法線、X軸方向でU軸として作成
    let origin = Point3D::new(0.0, 0.0, 0.0);
    let normal = dir(0.0, 0.0, 1.0);
    let u_direction = dir(1.0, 0.0, 0.0);

    let plane_sys = Plane3D::from_origin_and_axes(origin, normal, u_direction).unwrap();

    assert_eq!(plane_sys.origin(), origin);
    assert_abs_diff_eq!(plane_sys.normal().z(), 1.0, epsilon = TOLERANCE_F64);
}

#[test]
fn test_three_point_construction() {
    // 3点から平面座標系を作成
    let origin = Point3D::new(0.0, 0.0, 0.0);
    let point_u = Point3D::new(1.0, 0.0, 0.0);
    let point_v = Point3D::new(0.0, 1.0, 0.0);

    let plane_sys = Plane3D::from_three_points(origin, point_u, point_v).unwrap();

    // Z軸が法線になることを確認
    let normal = plane_sys.normal();
    assert_abs_diff_eq!(normal.x(), 0.0, epsilon = TOLERANCE_F64);
    assert_abs_diff_eq!(normal.y(), 0.0, epsilon = TOLERANCE_F64);
    assert_abs_diff_eq!(normal.z(), 1.0, epsilon = TOLERANCE_F64);
}

#[test]
fn test_coordinate_system_orthogonality() {
    // 座標系の直交性検証
    let origin = Point3D::new(1.0, 2.0, 3.0);
    let normal = dir(0.0, 0.0, 1.0);
    let u_direction = dir(1.0, 0.0, 0.0);

    let plane_sys = Plane3D::from_origin_and_axes(origin, normal, u_direction).unwrap();

    // U軸とV軸の直交性確認
    let u_axis = plane_sys.u_axis();
    let v_axis = plane_sys.v_axis();
    let normal_axis = plane_sys.normal();

    let dot_uv = u_axis.as_vector().dot(v_axis.as_vector());
    let dot_un = u_axis.as_vector().dot(normal_axis.as_vector());
    let dot_vn = v_axis.as_vector().dot(normal_axis.as_vector());

    assert_abs_diff_eq!(dot_uv, 0.0, epsilon = TOLERANCE_F64);
    assert_abs_diff_eq!(dot_un, 0.0, epsilon = TOLERANCE_F64);
    assert_abs_diff_eq!(dot_vn, 0.0, epsilon = TOLERANCE_F64);

    // 正規化確認
    assert_abs_diff_eq!(u_axis.as_vector().length(), 1.0, epsilon = TOLERANCE_F64);
    assert_abs_diff_eq!(v_axis.as_vector().length(), 1.0, epsilon = TOLERANCE_F64);
    assert_abs_diff_eq!(
        normal_axis.as_vector().length(),
        1.0,
        epsilon = TOLERANCE_F64
    );
}

#[test]
fn test_gram_schmidt_orthogonalization() {
    // グラム・シュミット正規直交化のテスト（U軸が法線と平行でない場合）
    let origin = Point3D::new(0.0, 0.0, 0.0);
    let normal = dir(0.0, 0.0, 1.0);
    let u_direction = dir(1.0, 1.0, 0.5); // 法線と平行でない

    let plane_sys = Plane3D::from_origin_and_axes(origin, normal, u_direction).unwrap();

    // 結果の座標系が直交系になることを確認
    let u_axis = plane_sys.u_axis();
    let v_axis = plane_sys.v_axis();
    let normal_axis = plane_sys.normal();

    // 直交性確認
    let dot_uv = u_axis.as_vector().dot(v_axis.as_vector());
    let dot_un = u_axis.as_vector().dot(normal_axis.as_vector());
    let dot_vn = v_axis.as_vector().dot(normal_axis.as_vector());

    assert_abs_diff_eq!(dot_uv, 0.0, epsilon = TOLERANCE_F64);
    assert_abs_diff_eq!(dot_un, 0.0, epsilon = TOLERANCE_F64);
    assert_abs_diff_eq!(dot_vn, 0.0, epsilon = TOLERANCE_F64);
}

#[test]
fn test_error_cases() {
    // エラーケースのテスト
    let origin = Point3D::new(0.0, 0.0, 0.0);

    let normal = dir(0.0, 0.0, 1.0);

    // U軸が法線と平行（直交成分がゼロ）
    let parallel_u = dir(0.0, 0.0, 1.0);

    let result = Plane3D::from_origin_and_axes(origin, normal, parallel_u);
    assert!(result.is_none());
}

#[test]
fn test_plane_metadata() {
    let point = Point3D::new(0.0, 0.0, 0.0);
    let normal = dir(0.0, 0.0, 1.0);
    let plane = Plane3D::from_point_and_normal(point, normal).unwrap();

    assert_eq!(plane.primitive_kind(), PrimitiveKind::Plane);
}

#[test]
fn test_plane_tolerant_eq() {
    let plane1 =
        Plane3D::from_point_and_normal(Point3D::new(0.0, 0.0, 0.0), dir(0.0, 0.0, 1.0)).unwrap();
    let plane2 =
        Plane3D::from_point_and_normal(Point3D::new(0.0, 0.0, 0.001), dir(0.0, 0.0, 1.0)).unwrap();

    assert!(plane1.tolerant_eq(&plane2, 0.01));
    assert_eq!(
        plane1.tolerant_eq(&plane2, 0.01),
        plane2.tolerant_eq(&plane1, 0.01)
    );
}

#[test]
fn test_plane_tolerant_eq_different_planes() {
    let plane1 =
        Plane3D::from_point_and_normal(Point3D::new(0.0, 0.0, 0.0), dir(0.0, 0.0, 1.0)).unwrap();
    let plane2 =
        Plane3D::from_point_and_normal(Point3D::new(0.0, 0.0, 0.0), dir(1.0, 0.0, 0.0)).unwrap();

    assert!(!plane1.tolerant_eq(&plane2, 0.01));
}
