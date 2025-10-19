use crate::linalg::quaternion::Quaternion;
use crate::linalg::vector::Vector3;
use std::f64::consts::PI;

#[test]
fn test_quaternion_basic_creation() {
    let q = Quaternion::<f64>::new(1.0, 2.0, 3.0, 4.0);
    assert_eq!(q.x(), 1.0);
    assert_eq!(q.y(), 2.0);
    assert_eq!(q.z(), 3.0);
    assert_eq!(q.w(), 4.0);
}

#[test]
fn test_quaternion_constants() {
    let identity = Quaternion::<f64>::identity();
    assert_eq!(identity.x(), 0.0);
    assert_eq!(identity.y(), 0.0);
    assert_eq!(identity.z(), 0.0);
    assert_eq!(identity.w(), 1.0);
    assert!(identity.is_unit());

    let zero = Quaternion::<f64>::zero();
    assert!(zero.is_zero());
}

#[test]
fn test_quaternion_norm_and_normalize() {
    let q = Quaternion::<f64>::new(1.0, 2.0, 3.0, 4.0);
    let expected_norm = (1.0 + 4.0 + 9.0 + 16.0_f64).sqrt();
    assert!((q.norm() - expected_norm).abs() < 1e-10_f64);

    let normalized = q.normalize().unwrap();
    assert!((normalized.norm() - 1.0).abs() < 1e-10_f64);
}

#[test]
fn test_quaternion_axis_angle_conversion() {
    // Z軸周りの90度回転
    let axis = Vector3::<f64>::new(0.0, 0.0, 1.0);
    let angle = PI / 2.0;
    let q = Quaternion::from_axis_angle(&axis, angle);

    // 軸角表現に戻す
    let (recovered_axis, recovered_angle) = q.to_axis_angle().unwrap();

    assert!((recovered_angle - angle).abs() < 1e-10_f64);
    assert!((recovered_axis.x() - axis.x()).abs() < 1e-10_f64);
    assert!((recovered_axis.y() - axis.y()).abs() < 1e-10_f64);
    assert!((recovered_axis.z() - axis.z()).abs() < 1e-10_f64);
}

#[test]
fn test_quaternion_vector_rotation() {
    // Z軸周りの90度回転
    let axis = Vector3::<f64>::new(0.0, 0.0, 1.0);
    let angle = PI / 2.0;
    let q = Quaternion::from_axis_angle(&axis, angle);

    // X軸のベクトルを回転 -> Y軸になるはず
    let x_axis = Vector3::<f64>::new(1.0, 0.0, 0.0);
    let rotated = q.rotate_vector(&x_axis);

    assert!((rotated.x() - 0.0).abs() < 1e-10_f64);
    assert!((rotated.y() - 1.0).abs() < 1e-10_f64);
    assert!((rotated.z() - 0.0).abs() < 1e-10_f64);

    // Y軸のベクトルを回転 -> -X軸になるはず
    let y_axis = Vector3::<f64>::new(0.0, 1.0, 0.0);
    let rotated_y = q.rotate_vector(&y_axis);

    assert!((rotated_y.x() - (-1.0)).abs() < 1e-10_f64);
    assert!((rotated_y.y() - 0.0).abs() < 1e-10_f64);
    assert!((rotated_y.z() - 0.0).abs() < 1e-10_f64);
}

#[test]
fn test_quaternion_hamilton_product() {
    // 基本クォータニオン単位での積
    let i = Quaternion::<f64>::new(1.0, 0.0, 0.0, 0.0);
    let j = Quaternion::<f64>::new(0.0, 1.0, 0.0, 0.0);
    let k = Quaternion::<f64>::new(0.0, 0.0, 1.0, 0.0);

    // i * j = k
    let ij = i * j;
    assert!((ij.x() - k.x()).abs() < 1e-10_f64);
    assert!((ij.y() - k.y()).abs() < 1e-10_f64);
    assert!((ij.z() - k.z()).abs() < 1e-10_f64);
    assert!((ij.w() - k.w()).abs() < 1e-10_f64);

    // j * k = i
    let jk = j * k;
    assert!((jk.x() - i.x()).abs() < 1e-10_f64);
    assert!((jk.y() - i.y()).abs() < 1e-10_f64);
    assert!((jk.z() - i.z()).abs() < 1e-10_f64);
    assert!((jk.w() - i.w()).abs() < 1e-10_f64);

    // k * i = j
    let ki = k * i;
    assert!((ki.x() - j.x()).abs() < 1e-10_f64);
    assert!((ki.y() - j.y()).abs() < 1e-10_f64);
    assert!((ki.z() - j.z()).abs() < 1e-10_f64);
    assert!((ki.w() - j.w()).abs() < 1e-10_f64);
}

#[test]
fn test_quaternion_conjugate_and_inverse() {
    let q = Quaternion::<f64>::new(1.0, 2.0, 3.0, 4.0);
    let conj = q.conjugate();

    assert_eq!(conj.x(), -1.0);
    assert_eq!(conj.y(), -2.0);
    assert_eq!(conj.z(), -3.0);
    assert_eq!(conj.w(), 4.0);

    // q * q^(-1) = 1
    let inv = q.inverse().unwrap();
    let product = q * inv;

    assert!((product.x() - 0.0).abs() < 1e-10_f64);
    assert!((product.y() - 0.0).abs() < 1e-10_f64);
    assert!((product.z() - 0.0).abs() < 1e-10_f64);
    assert!((product.w() - 1.0).abs() < 1e-10_f64);
}

#[test]
fn test_quaternion_rotation_composition() {
    // X軸周りの90度回転（右手系で手前に向かう回転）
    let q1 = Quaternion::from_axis_angle(&Vector3::<f64>::new(1.0, 0.0, 0.0), PI / 2.0);
    // Y軸周りの90度回転
    let q2 = Quaternion::from_axis_angle(&Vector3::<f64>::new(0.0, 1.0, 0.0), PI / 2.0);

    // 合成回転: 先にq1を適用してからq2を適用
    let combined = q2 * q1;

    // Z軸ベクトルを合成回転で変換
    let z_axis = Vector3::<f64>::new(0.0, 0.0, 1.0);
    let after_q1 = q1.rotate_vector(&z_axis); // Z軸をX軸周りに90度回転 -> -Y軸
    let after_combined = combined.rotate_vector(&z_axis);

    // q1: Z(0,0,1) -> -Y(0,-1,0) （X軸周り90度回転）
    assert!((after_q1.x() - 0.0).abs() < 1e-10_f64);
    assert!((after_q1.y() - (-1.0)).abs() < 1e-10_f64);
    assert!((after_q1.z() - 0.0).abs() < 1e-10_f64);

    // 合成回転の結果も同様（Y軸周りの回転は-Y軸に影響しない）
    assert!((after_combined.x() - 0.0).abs() < 1e-10_f64);
    assert!((after_combined.y() - (-1.0)).abs() < 1e-10_f64);
    assert!((after_combined.z() - 0.0).abs() < 1e-10_f64);

    // ベクトルの大きさは保持される
    assert!((after_combined.norm() - 1.0).abs() < 1e-10_f64);
}

#[test]
fn test_quaternion_euler_angles() {
    let pitch = PI / 6.0; // 30度
    let yaw = PI / 4.0; // 45度
    let roll = PI / 3.0; // 60度

    let q = Quaternion::<f64>::from_euler_angles(pitch, yaw, roll);
    let (recovered_pitch, recovered_yaw, recovered_roll) = q.to_euler_angles();

    assert!((pitch - recovered_pitch).abs() < 1e-10_f64);
    assert!((yaw - recovered_yaw).abs() < 1e-10_f64);
    assert!((roll - recovered_roll).abs() < 1e-10_f64);
}

#[test]
fn test_quaternion_from_to_rotation() {
    // X軸からY軸への回転
    let from = Vector3::<f64>::new(1.0, 0.0, 0.0);
    let to = Vector3::<f64>::new(0.0, 1.0, 0.0);

    let q = Quaternion::from_to_rotation(&from, &to).unwrap();
    let rotated = q.rotate_vector(&from);

    assert!((rotated.x() - to.x()).abs() < 1e-10_f64);
    assert!((rotated.y() - to.y()).abs() < 1e-10_f64);
    assert!((rotated.z() - to.z()).abs() < 1e-10_f64);

    // 反対方向のテスト
    let opposite_to = Vector3::<f64>::new(-1.0, 0.0, 0.0);
    let q_opposite = Quaternion::from_to_rotation(&from, &opposite_to).unwrap();
    let rotated_opposite = q_opposite.rotate_vector(&from);

    assert!((rotated_opposite.x() - opposite_to.x()).abs() < 1e-10_f64);
    assert!((rotated_opposite.y() - opposite_to.y()).abs() < 1e-10_f64);
    assert!((rotated_opposite.z() - opposite_to.z()).abs() < 1e-10_f64);
}

#[test]
fn test_quaternion_slerp() {
    let q1 = Quaternion::<f64>::identity();
    let axis = Vector3::<f64>::new(0.0, 0.0, 1.0);
    let q2 = Quaternion::from_axis_angle(&axis, PI / 2.0);

    // 中間点での補間
    let interpolated = q1.slerp(&q2, 0.5).unwrap();
    let expected_angle = PI / 4.0; // 45度

    assert!((interpolated.angle() - expected_angle).abs() < 1e-10_f64);

    // 端点でのテスト
    let start = q1.slerp(&q2, 0.0).unwrap();
    let end = q1.slerp(&q2, 1.0).unwrap();

    assert!((start.dot(&q1) - 1.0).abs() < 1e-10_f64);
    assert!((end.dot(&q2) - 1.0).abs() < 1e-10_f64);
}

#[test]
fn test_quaternion_lerp_and_nlerp() {
    let q1 = Quaternion::<f64>::identity();
    let axis = Vector3::<f64>::new(0.0, 0.0, 1.0);
    let q2 = Quaternion::from_axis_angle(&axis, PI / 2.0);

    let lerp_result = q1.lerp(&q2, 0.5);
    let nlerp_result = q1.nlerp(&q2, 0.5).unwrap();

    // NLERPは正規化されている
    assert!((nlerp_result.norm() - 1.0).abs() < 1e-10_f64);

    // LERPとNLERPは似た結果を与える（角度が小さい場合）
    let dot_product = lerp_result.dot(&nlerp_result);
    assert!(dot_product > 0.99); // 非常に似ている
}

#[test]
fn test_quaternion_angle_calculation() {
    // 90度回転
    let axis = Vector3::<f64>::new(0.0, 0.0, 1.0);
    let angle = PI / 2.0;
    let q = Quaternion::from_axis_angle(&axis, angle);

    assert!((q.angle() - angle).abs() < 1e-10_f64);

    // 単位クォータニオンは角度0
    let identity = Quaternion::<f64>::identity();
    assert!(identity.angle().abs() < 1e-10_f64);
}

#[test]
fn test_quaternion_arithmetic_operations() {
    let q1 = Quaternion::<f64>::new(1.0, 2.0, 3.0, 4.0);
    let q2 = Quaternion::<f64>::new(5.0, 6.0, 7.0, 8.0);

    // 加算
    let sum = q1 + q2;
    assert_eq!(sum.x(), 6.0);
    assert_eq!(sum.y(), 8.0);
    assert_eq!(sum.z(), 10.0);
    assert_eq!(sum.w(), 12.0);

    // 減算
    let diff = q2 - q1;
    assert_eq!(diff.x(), 4.0);
    assert_eq!(diff.y(), 4.0);
    assert_eq!(diff.z(), 4.0);
    assert_eq!(diff.w(), 4.0);

    // スカラー倍
    let scaled = q1 * 2.0;
    assert_eq!(scaled.x(), 2.0);
    assert_eq!(scaled.y(), 4.0);
    assert_eq!(scaled.z(), 6.0);
    assert_eq!(scaled.w(), 8.0);

    // 負符号
    let negated = -q1;
    assert_eq!(negated.x(), -1.0);
    assert_eq!(negated.y(), -2.0);
    assert_eq!(negated.z(), -3.0);
    assert_eq!(negated.w(), -4.0);
}

#[test]
fn test_quaternion_type_conversions() {
    let q = Quaternion::<f64>::new(1.0, 2.0, 3.0, 4.0);

    // Vector4への変換
    let vec4 = q.to_vector4();
    assert_eq!(vec4.x(), 1.0);
    assert_eq!(vec4.y(), 2.0);
    assert_eq!(vec4.z(), 3.0);
    assert_eq!(vec4.w(), 4.0);

    // Vector4からの変換
    let q_from_vec4 = Quaternion::from_vector4(vec4);
    assert_eq!(q_from_vec4.x(), q.x());
    assert_eq!(q_from_vec4.y(), q.y());
    assert_eq!(q_from_vec4.z(), q.z());
    assert_eq!(q_from_vec4.w(), q.w());

    // ベクトル部とスカラー部の分離
    let vector_part = q.vector_part();
    let scalar_part = q.scalar_part();

    assert_eq!(vector_part.x(), 1.0);
    assert_eq!(vector_part.y(), 2.0);
    assert_eq!(vector_part.z(), 3.0);
    assert_eq!(scalar_part, 4.0);
}
