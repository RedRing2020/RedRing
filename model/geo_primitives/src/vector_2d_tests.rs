//! Vector2D のテスト

use crate::{Point2D, Vector2D};
use geo_foundation::Angle;
use std::f64;

/// 基本作成テスト
#[test]
fn test_vector2d_creation() {
    let v = Vector2D::new(3.0, 4.0);
    assert_eq!(v.x(), 3.0);
    assert_eq!(v.y(), 4.0);

    let components = v.components();
    assert_eq!(components, [3.0, 4.0]);
}

/// 特殊ベクトルテスト
#[test]
fn test_special_vectors() {
    let zero = Vector2D::<f64>::zero();
    assert_eq!(zero.x(), 0.0);
    assert_eq!(zero.y(), 0.0);

    let unit_x = Vector2D::<f64>::unit_x();
    assert_eq!(unit_x.x(), 1.0);
    assert_eq!(unit_x.y(), 0.0);

    let unit_y = Vector2D::<f64>::unit_y();
    assert_eq!(unit_y.x(), 0.0);
    assert_eq!(unit_y.y(), 1.0);
}

/// 点からベクトル生成テスト
#[test]
fn test_from_points() {
    let p1 = Point2D::new(1.0, 2.0);
    let p2 = Point2D::new(4.0, 6.0);
    let v = Vector2D::from_points(p1, p2);

    assert_eq!(v.x(), 3.0);
    assert_eq!(v.y(), 4.0);
}

/// 長さ計算テスト
#[test]
fn test_vector2d_length() {
    let v = Vector2D::new(3.0, 4.0);
    assert_eq!(v.length_squared(), 25.0);
    assert_eq!(v.length(), 5.0);

    let zero = Vector2D::<f64>::zero();
    assert_eq!(zero.length(), 0.0);
}

/// 単位ベクトル判定テスト
#[test]
fn test_is_unit() {
    let unit = Vector2D::new(1.0, 0.0);
    assert!(unit.is_unit(f64::EPSILON));

    let non_unit = Vector2D::new(2.0, 0.0);
    assert!(!non_unit.is_unit(f64::EPSILON));

    let zero = Vector2D::<f64>::zero();
    assert!(!zero.is_unit(f64::EPSILON));
}

/// ゼロベクトル判定テスト
#[test]
fn test_is_zero() {
    let zero = Vector2D::<f64>::zero();
    assert!(zero.is_zero(f64::EPSILON));

    let non_zero = Vector2D::new(1.0, 0.0);
    assert!(!non_zero.is_zero(f64::EPSILON));

    let small = Vector2D::new(1e-10, 1e-10);
    assert!(small.is_zero(1e-9));
    assert!(!small.is_zero(1e-11));
}

/// 正規化テスト
#[test]
fn test_normalize() {
    let v = Vector2D::new(3.0, 4.0);
    let normalized = v.normalize();

    assert!((normalized.length() - 1.0f64).abs() < f64::EPSILON);
    assert_eq!(normalized.x(), 0.6);
    assert_eq!(normalized.y(), 0.8);

    // ゼロベクトルの正規化
    let zero = Vector2D::<f64>::zero();
    let normalized_zero = zero.normalize();
    assert!(normalized_zero.is_zero(f64::EPSILON));
}

/// 安全な正規化テスト
#[test]
fn test_try_normalize() {
    let v = Vector2D::new(3.0, 4.0);
    let normalized = v.try_normalize().unwrap();
    assert!((normalized.length() - 1.0f64).abs() < f64::EPSILON);

    let zero = Vector2D::<f64>::zero();
    assert!(zero.try_normalize().is_none());
}

/// 長さ指定テスト
#[test]
fn test_with_length() {
    let v = Vector2D::new(3.0, 4.0);
    let scaled = v.with_length(10.0).unwrap();

    assert!((scaled.length() - 10.0f64).abs() < f64::EPSILON);

    let zero = Vector2D::<f64>::zero();
    assert!(zero.with_length(5.0).is_none());
}

/// 内積テスト
#[test]
fn test_dot_product() {
    let v1 = Vector2D::new(1.0, 2.0);
    let v2 = Vector2D::new(3.0, 4.0);

    assert_eq!(v1.dot(&v2), 11.0); // 1*3 + 2*4 = 11

    // 垂直ベクトルの内積は0
    let v3 = Vector2D::new(1.0, 0.0);
    let v4 = Vector2D::new(0.0, 1.0);
    assert_eq!(v3.dot(&v4), 0.0);
}

/// 外積（Z成分）テスト
#[test]
fn test_cross_product() {
    let v1 = Vector2D::new(1.0, 0.0);
    let v2 = Vector2D::new(0.0, 1.0);

    assert_eq!(v1.cross(&v2), 1.0); // 正の回転
    assert_eq!(v2.cross(&v1), -1.0); // 負の回転

    // 平行ベクトルの外積は0
    let v3 = Vector2D::new(2.0, 4.0);
    let v4 = Vector2D::new(1.0, 2.0);
    assert_eq!(v3.cross(&v4), 0.0);
}

/// 平行・垂直判定テスト
#[test]
fn test_parallel_perpendicular() {
    let v1 = Vector2D::new(1.0, 2.0);
    let v2 = Vector2D::new(2.0, 4.0); // 平行
    let v3 = Vector2D::new(-2.0, 1.0); // 垂直

    assert!(v1.is_parallel(&v2, f64::EPSILON));
    assert!(v1.is_perpendicular(&v3, f64::EPSILON));
    assert!(!v1.is_parallel(&v3, f64::EPSILON));
    assert!(!v1.is_perpendicular(&v2, f64::EPSILON));
}

/// 角度計算テスト
#[test]
fn test_angles() {
    // X軸ベクトルの角度は0
    let x_axis = Vector2D::new(1.0, 0.0);
    assert!((x_axis.angle().to_radians() - 0.0f64).abs() < f64::EPSILON);

    // Y軸ベクトルの角度はπ/2
    let y_axis = Vector2D::new(0.0, 1.0);
    assert!((y_axis.angle().to_radians() - std::f64::consts::FRAC_PI_2).abs() < f64::EPSILON);

    // ベクトル間の角度
    let angle = x_axis.angle_to(&y_axis);
    assert!((angle.to_radians() - std::f64::consts::FRAC_PI_2).abs() < f64::EPSILON);
}

/// 角度からベクトル生成テスト
#[test]
fn test_from_angle() {
    use std::f64::consts::{FRAC_PI_2, PI};

    let v1 = Vector2D::from_angle(Angle::from_radians(0.0));
    assert!((v1.x() - 1.0f64).abs() < f64::EPSILON);
    assert!((v1.y() - 0.0f64).abs() < f64::EPSILON);

    let v2 = Vector2D::from_angle(Angle::from_radians(FRAC_PI_2));
    assert!((v2.x() - 0.0f64).abs() < f64::EPSILON);
    assert!((v2.y() - 1.0f64).abs() < f64::EPSILON);

    let v3 = Vector2D::from_angle_length(Angle::from_radians(PI), 2.0);
    assert!((v3.x() - (-2.0f64)).abs() < f64::EPSILON);
    assert!(v3.y().abs() < 1e-15); // sin(PI)の浮動小数点誤差を考慮
}

/// 回転テスト
#[test]
fn test_rotation() {
    use std::f64::consts::FRAC_PI_2;

    let v = Vector2D::new(1.0, 0.0);

    // 90度回転
    let rotated = v.rotate(Angle::from_radians(FRAC_PI_2));
    assert!((rotated.x() - 0.0_f64).abs() < f64::EPSILON);
    assert!((rotated.y() - 1.0_f64).abs() < f64::EPSILON);

    // 90度回転（専用メソッド）
    let rot90 = v.rotate_90();
    assert!((rot90.x() - 0.0_f64).abs() < f64::EPSILON);
    assert!((rot90.y() - 1.0_f64).abs() < f64::EPSILON);

    // -90度回転
    let rot_neg90 = v.rotate_neg_90();
    assert!((rot_neg90.x() - 0.0_f64).abs() < f64::EPSILON);
    assert!((rot_neg90.y() - (-1.0_f64)).abs() < f64::EPSILON);
}

/// 反射テスト
#[test]
fn test_reflection() {
    let v = Vector2D::new(1.0, 1.0);
    let normal = Vector2D::new(1.0, 0.0); // X軸法線

    let reflected = v.reflect(&normal);
    assert!((reflected.x() - (-1.0f64)).abs() < f64::EPSILON);
    assert!((reflected.y() - 1.0f64).abs() < f64::EPSILON);
}

/// 線形補間テスト
#[test]
fn test_lerp() {
    let v1 = Vector2D::new(0.0, 0.0);
    let v2 = Vector2D::new(4.0, 2.0);

    let mid = v1.lerp(&v2, 0.5);
    assert_eq!(mid.x(), 2.0);
    assert_eq!(mid.y(), 1.0);

    let start = v1.lerp(&v2, 0.0);
    assert_eq!(start, v1);

    let end = v1.lerp(&v2, 1.0);
    assert_eq!(end, v2);
}

/// 投影・拒絶テスト
#[test]
fn test_projection() {
    let v = Vector2D::new(3.0, 4.0);
    let target = Vector2D::new(1.0, 0.0); // X軸

    let projected = v.project_onto(&target);
    assert_eq!(projected.x(), 3.0);
    assert_eq!(projected.y(), 0.0);

    let rejected = v.reject_from(&target);
    assert_eq!(rejected.x(), 0.0);
    assert_eq!(rejected.y(), 4.0);

    // 投影と拒絶の合計は元のベクトル
    let sum = projected + rejected;
    let diff_x: f64 = sum.x() - v.x();
    let diff_y: f64 = sum.y() - v.y();
    assert!(diff_x.abs() < f64::EPSILON);
    assert!(diff_y.abs() < f64::EPSILON);
}

/// 成分演算テスト
#[test]
fn test_component_operations() {
    let v1 = Vector2D::new(1.0, 4.0);
    let v2 = Vector2D::new(3.0, 2.0);

    let min = v1.min(&v2);
    assert_eq!(min.x(), 1.0);
    assert_eq!(min.y(), 2.0);

    let max = v1.max(&v2);
    assert_eq!(max.x(), 3.0);
    assert_eq!(max.y(), 4.0);

    let v3 = Vector2D::new(-2.0, 3.0);
    let abs = v3.abs();
    assert_eq!(abs.x(), 2.0);
    assert_eq!(abs.y(), 3.0);
}

/// 算術演算子テスト
#[test]
fn test_vector2d_operations() {
    let v1 = Vector2D::new(1.0, 2.0);
    let v2 = Vector2D::new(3.0, 4.0);

    // 加算
    let sum = v1 + v2;
    assert_eq!(sum.x(), 4.0);
    assert_eq!(sum.y(), 6.0);

    // 減算
    let diff = v2 - v1;
    assert_eq!(diff.x(), 2.0);
    assert_eq!(diff.y(), 2.0);

    // スカラー乗算
    let scaled = v1 * 2.0;
    assert_eq!(scaled.x(), 2.0);
    assert_eq!(scaled.y(), 4.0);

    // 負号
    let neg = -v1;
    assert_eq!(neg.x(), -1.0);
    assert_eq!(neg.y(), -2.0);
}

/// Point2DとVector2Dの演算テスト
#[test]
fn test_point_vector_operations() {
    let p = Point2D::new(1.0, 2.0);
    let v = Vector2D::new(3.0, 4.0);

    // Point + Vector = Point
    let p2 = p + v;
    assert_eq!(p2.x(), 4.0);
    assert_eq!(p2.y(), 6.0);

    // Point - Vector = Point
    let p3 = p2 - v;
    assert_eq!(p3.x(), 1.0);
    assert_eq!(p3.y(), 2.0);

    // Point - Point = Vector
    let v2 = p2 - p;
    assert_eq!(v2.x(), 3.0);
    assert_eq!(v2.y(), 4.0);
}

/// 3D変換テスト
#[test]
fn test_to_3d() {
    let v2d = Vector2D::new(3.0, 4.0);
    let v3d = v2d.to_3d();

    assert_eq!(v3d.x(), 3.0);
    assert_eq!(v3d.y(), 4.0);
    assert_eq!(v3d.z(), 0.0);

    let v3d_z = v2d.to_3d_with_z(5.0);
    assert_eq!(v3d_z.x(), 3.0);
    assert_eq!(v3d_z.y(), 4.0);
    assert_eq!(v3d_z.z(), 5.0);
}

/// f32での動作テスト
#[test]
fn test_vector2d_f32() {
    let v: Vector2D<f32> = Vector2D::new(3.0f32, 4.0f32);
    assert_eq!(v.length(), 5.0f32);

    let normalized = v.normalize();
    assert!((normalized.length() - 1.0f32).abs() < f32::EPSILON);
}

// ============================================================================
// Transform テスト (vector_2d_transform.rs の機能テスト)
// ============================================================================

#[test]
fn test_2d_rotation() {
    use std::f64::consts::PI;

    let v = Vector2D::new(1.0, 0.0);
    // rotate_zは未実装のため、実装済みrotateメソッドを使用
    // Angle<T>型への変換が必要
    let rotated = v.rotate((PI / 2.0).into());

    assert!((rotated.x() - 0.0).abs() < 1e-10);
    assert!((rotated.y() - 1.0).abs() < 1e-10);
}

#[test]
#[ignore] // 実装待ち
fn test_transform_vector_trait() {
    // 一時的にテストをスキップ（TransformVector2Dトレイトの実装待ち）
    // テストは実装待ち
}

#[test]
#[ignore] // 実装待ち
fn test_transform_point_trait() {
    // 一時的にテストをスキップ（TransformPoint2Dトレイトの実装待ち）
    // テストは実装待ち
}

#[test]
fn test_rotation_identity() {
    use std::f64::consts::PI;

    let v = Vector2D::new(1.0, 2.0);

    // 2π回転は元のベクトルと同じ
    // rotate_zは未実装のため、実装済みrotateメソッドを使用
    let rotated = v.rotate((2.0 * PI).into());

    assert!((rotated.x() - v.x()).abs() < 1e-10_f64);
    assert!((rotated.y() - v.y()).abs() < 1e-10_f64);
}
