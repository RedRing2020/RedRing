//! InfiniteLine3D のテストコード

// InfiniteLine3Dの実装が完了したため、テストを有効化

use crate::geometry3d::{Direction3D, InfiniteLine3D, Point3D, Vector};
use geo_foundation::abstract_types::geometry::InfiniteLine3D as InfiniteLine3DTrait;
use geo_foundation::abstract_types::geometry::Direction;
use geo_foundation::common::constants::GEOMETRIC_TOLERANCE;

#[test]
fn test_infinite_line_3d_creation() {
    let origin = Point3D::new(1.0, 2.0, 3.0);
    let direction = Direction3D::from_vector(Vector::new(1.0, 0.0, 0.0)).unwrap();
    let line = InfiniteLine3D::new(origin, direction);

    assert_eq!(line.origin(), origin);
    assert_eq!(line.direction(), direction);
}

#[test]
fn test_from_two_points_3d() {
    let p1 = Point3D::new(0.0, 0.0, 0.0);
    let p2 = Point3D::new(1.0, 1.0, 1.0);
    let line = InfiniteLine3D::from_two_points(p1, p2).unwrap();

    assert_eq!(line.origin(), p1);

    let dir = line.direction();
    let expected_length = (3.0_f64).sqrt();
    assert!((dir.x() - 1.0 / expected_length).abs() < GEOMETRIC_TOLERANCE);
    assert!((dir.y() - 1.0 / expected_length).abs() < GEOMETRIC_TOLERANCE);
    assert!((dir.z() - 1.0 / expected_length).abs() < GEOMETRIC_TOLERANCE);
}

#[test]
fn test_axis_parallel_lines() {
    let x_line = InfiniteLine3D::along_x_axis(1.0, 2.0);
    assert_eq!(x_line.origin(), Point3D::new(0.0, 1.0, 2.0));
    assert!((x_line.direction().x() - 1.0_f64).abs() < GEOMETRIC_TOLERANCE);

    let y_line = InfiniteLine3D::along_y_axis(1.0, 2.0);
    assert_eq!(y_line.origin(), Point3D::new(1.0, 0.0, 2.0));
    assert!((y_line.direction().y() - 1.0_f64).abs() < GEOMETRIC_TOLERANCE);

    let z_line = InfiniteLine3D::along_z_axis(1.0, 2.0);
    assert_eq!(z_line.origin(), Point3D::new(1.0, 2.0, 0.0));
    assert!((z_line.direction().z() - 1.0_f64).abs() < GEOMETRIC_TOLERANCE);
}

#[test]
fn test_distance_to_point_3d() {
    let line = InfiniteLine3D::along_x_axis(0.0, 0.0); // X軸
    let point = Point3D::new(5.0, 3.0, 4.0);
    let distance = line.distance_to_point(&point);

    // 3Dでの距離は (3^2 + 4^2)^0.5 = 5
    assert!((distance - 5.0_f64).abs() < GEOMETRIC_TOLERANCE);
}

#[test]
fn test_closest_point_3d() {
    let line = InfiniteLine3D::along_x_axis(0.0, 0.0); // X軸
    let point = Point3D::new(5.0, 3.0, 4.0);
    let closest = line.closest_point(&point);

    assert!((closest.x() - 5.0_f64).abs() < GEOMETRIC_TOLERANCE);
    assert!((closest.y() - 0.0_f64).abs() < GEOMETRIC_TOLERANCE);
    assert!((closest.z() - 0.0_f64).abs() < GEOMETRIC_TOLERANCE);
}

#[test]
fn test_line_intersection_3d() {
    let line1 = InfiniteLine3D::along_x_axis(0.0, 0.0); // X軸
    let line2 = InfiniteLine3D::along_y_axis(0.0, 0.0); // Y軸

    let intersection = line1.intersect_line(&line2).unwrap();
    assert!((intersection.x() - 0.0_f64).abs() < GEOMETRIC_TOLERANCE);
    assert!((intersection.y() - 0.0_f64).abs() < GEOMETRIC_TOLERANCE);
    assert!((intersection.z() - 0.0_f64).abs() < GEOMETRIC_TOLERANCE);
}

#[test]
fn test_parallel_lines_3d() {
    let line1 = InfiniteLine3D::along_x_axis(0.0, 0.0);
    let line2 = InfiniteLine3D::along_x_axis(1.0, 1.0);

    assert!(line1.is_parallel_to(&line2, GEOMETRIC_TOLERANCE));
    assert!(!line1.is_coincident_with(&line2, GEOMETRIC_TOLERANCE));
}

#[test]
fn test_skew_lines() {
    let line1 = InfiniteLine3D::along_x_axis(0.0, 0.0);
    let line2 = InfiniteLine3D::along_y_axis(0.0, 1.0); // Z=1でY軸方向

    assert!(line1.is_skew_to(&line2, GEOMETRIC_TOLERANCE));
}

#[test]
fn test_distance_to_line_3d() {
    let line1 = InfiniteLine3D::along_x_axis(0.0, 0.0);
    let line2 = InfiniteLine3D::along_x_axis(0.0, 1.0); // 平行線

    let distance = line1.distance_to_line(&line2);
    assert!((distance - 1.0_f64).abs() < GEOMETRIC_TOLERANCE);
}

#[test]
fn test_plane_intersection() {
    let line = InfiniteLine3D::from_two_points(
        Point3D::new(0.0, 0.0, 0.0),
        Point3D::new(1.0, 1.0, 1.0),
    )
    .unwrap();

    // XY平面との交点
    let plane_normal = Vector::new(0.0, 0.0, 1.0);
    let plane_point = Point3D::new(0.0, 0.0, 0.0);

    let intersection = line.intersect_plane(&plane_point, &plane_normal).unwrap();
    assert!((intersection.z() - 0.0_f64).abs() < GEOMETRIC_TOLERANCE);
}

/*
#[test]
fn test_sample_points() {
    let line = InfiniteLine3D::new(
        Point3D::new(0.0, 0.0, 0.0),
        Direction3D::positive_x(),
    );

    let points = line.sample_points(-2.0, 2.0, 5);
    assert_eq!(points.len(), 5);

    // 最初の点
    let first = &points[0];
    assert!((first.x() - (-2.0)).abs() < GEOMETRIC_TOLERANCE);

    // 最後の点
    let last = &points[4];
    assert!((last.x() - 2.0).abs() < GEOMETRIC_TOLERANCE);
}
*/

#[test]
fn test_parameter_at_point_3d() {
    let line = InfiniteLine3D::from_two_points(
        Point3D::new(0.0, 0.0, 0.0),
        Point3D::new(1.0, 0.0, 0.0),
    )
    .unwrap();

    let point = Point3D::new(5.0, 0.0, 0.0);
    let param = line.parameter_at_point(&point);

    assert!((param - 5.0_f64).abs() < GEOMETRIC_TOLERANCE);
}

#[test]
fn test_project_to_xy() {
    let line3d = InfiniteLine3D::new(
        Point3D::new(1.0, 2.0, 3.0),
        Direction3D::positive_x(),
    );

    let line2d = line3d.project_to_xy();

    // XY平面への投影確認
    use geo_foundation::abstract_types::geometry::InfiniteLine2D as InfiniteLine2DTrait;
    let origin2d = line2d.origin();
    assert!((origin2d.x() - 1.0_f64).abs() < GEOMETRIC_TOLERANCE);
    assert!((origin2d.y() - 2.0_f64).abs() < GEOMETRIC_TOLERANCE);

    // 方向ベクトルがX軸方向であることを確認
    let dir2d = line2d.direction();
    assert!((dir2d.x() - 1.0_f64).abs() < GEOMETRIC_TOLERANCE);
    assert!((dir2d.y() - 0.0_f64).abs() < GEOMETRIC_TOLERANCE);
}
