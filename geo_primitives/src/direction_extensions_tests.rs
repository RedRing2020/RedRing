//! Direction2D/3D Extensions Tests
//!
//! Core + Extensions パターンの検証テスト

use crate::{Direction2D, Direction3D};
// use geo_foundation::Scalar; // 未使用

type FloatType = f64;

#[test]
fn test_direction2d_extensions() {
    let dir = Direction2D::positive_x();

    // Extension methodsのテスト
    let from_angle = Direction2D::from_angle_radians(std::f64::consts::FRAC_PI_4);
    assert!((from_angle.x() - std::f64::consts::FRAC_1_SQRT_2).abs() < FloatType::EPSILON);
    assert!((from_angle.y() - std::f64::consts::FRAC_1_SQRT_2).abs() < FloatType::EPSILON);

    let perpendicular = dir.perpendicular();
    assert_eq!(perpendicular, Direction2D::positive_y());

    let angle: FloatType = dir.to_angle_radians();
    assert!((angle - 0.0).abs() < FloatType::EPSILON);
}

#[test]
fn test_direction2d_rotations() {
    let dir_x = Direction2D::positive_x();

    // 90度回転のテスト
    let rotated = dir_x.rotated_by_angle(std::f64::consts::FRAC_PI_2);
    assert!((rotated.x() - 0.0).abs() < FloatType::EPSILON);
    assert!((rotated.y() - 1.0).abs() < FloatType::EPSILON);

    // 角度計算のテスト
    let dir_y = Direction2D::positive_y();
    let angle_between = dir_x.angle_between(&dir_y);
    assert!((angle_between - std::f64::consts::FRAC_PI_2).abs() < FloatType::EPSILON);
}

#[test]
fn test_direction3d_extensions() {
    let dir = Direction3D::positive_x();

    // Extension methodsのテスト
    let azimuth = dir.azimuth_angle();
    let azimuth_val: FloatType = azimuth.to_radians();
    assert!((azimuth_val - 0.0f64).abs() < FloatType::EPSILON);

    let elevation = dir.elevation_angle();
    let elevation_val: FloatType = elevation.to_radians();
    assert!((elevation_val - std::f64::consts::FRAC_PI_2).abs() < FloatType::EPSILON);
}

#[test]
fn test_direction_relations() {
    let dir1 = Direction2D::positive_x();
    let dir2 = Direction2D::positive_y();
    let dir3 = Direction2D::negative_x();

    // Core relations (should work)
    assert!(dir1.is_parallel_to(&dir1));
    assert!(dir1.is_perpendicular_to(&dir2));

    // Extension relations
    assert!(dir1.is_same_direction_with_tolerance(&dir1, FloatType::EPSILON));
    assert!(dir1.is_opposite_direction_with_tolerance(&dir3, FloatType::EPSILON));
    assert!(!dir1.is_same_direction_with_tolerance(&dir2, FloatType::EPSILON));
}

#[test]
fn test_direction_slerp() {
    let dir1 = Direction2D::positive_x();
    let dir2 = Direction2D::positive_y();

    // 50%補間点のテスト
    let mid = dir1.slerp(&dir2, 0.5);
    let expected_angle = std::f64::consts::FRAC_PI_4; // 45度
    let actual_angle = mid.to_angle_radians();
    assert!((actual_angle - expected_angle).abs() < FloatType::EPSILON);
}
