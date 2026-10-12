//! Direction2D のテスト

use crate::{Direction2D, Vector2D};

type TestType = f64;

#[test]
fn test_angle_operations() {
    // 基本的な角度操作をテスト
    let _right = Direction2D::<TestType>::positive_x();
    let _up = Direction2D::<TestType>::new(0.0, 1.0).unwrap();

    // 角度から方向を作成
    let from_radians = Direction2D::from_angle_radians(std::f64::consts::PI / 2.0);
    assert!((from_radians.x() - 0.0).abs() < TestType::EPSILON);
    assert!((from_radians.y() - 1.0).abs() < TestType::EPSILON);
}

#[test]
fn test_slerp_interpolation() {
    let start = Direction2D::<TestType>::positive_x();
    let end = Direction2D::<TestType>::new(0.0, 1.0).unwrap();

    // 中点での補間
    let mid = start.slerp(end, 0.5);
    let expected_angle = std::f64::consts::PI / 4.0; // 45度
    let actual_angle = mid.to_angle_radians();

    assert!((actual_angle - expected_angle).abs() < TestType::EPSILON);
}

#[test]
fn test_perpendicular_direction() {
    let dir = Direction2D::<TestType>::positive_x();
    let perp = dir.rotate_90();

    // 垂直方向は Y軸正方向
    assert!((perp.x() - 0.0).abs() < TestType::EPSILON);
    assert!((perp.y() - 1.0).abs() < TestType::EPSILON);
}

#[test]
fn test_direction_from_vector() {
    let dir = Direction2D::from_angle_radians(std::f64::consts::PI / 6.0); // 30度
    let vec = Vector2D::new(dir.x() * 5.0, dir.y() * 5.0);
    let dir_from_vec = Direction2D::from_vector(vec).unwrap();

    // 正規化されているので元の方向と一致すべき
    assert!((dir.x() - dir_from_vec.x()).abs() < TestType::EPSILON);
    assert!((dir.y() - dir_from_vec.y()).abs() < TestType::EPSILON);
}
