//! Direction3D のテスト

use crate::{Direction3D, Vector3D};
use analysis::abstract_types::Scalar;

type TestType = f64;

#[test]
fn test_direction_3d_construction() {
    // ベクトルから方向を作成
    let vec = Vector3D::new(1.0, 1.0, 1.0);
    let dir = Direction3D::from_vector(vec).unwrap();

    // 正規化されているかチェック（より緩い許容誤差）
    let length_squared = dir.x() * dir.x() + dir.y() * dir.y() + dir.z() * dir.z();
    assert!((length_squared - 1.0).abs() < 1e-10);
}

#[test]
fn test_direction_normalization() {
    // 異なる長さのベクトルから同じ方向を作成
    let vec1 = Vector3D::new(1.0, 0.0, 0.0);
    let vec2 = Vector3D::new(5.0, 0.0, 0.0);

    let dir1 = Direction3D::from_vector(vec1).unwrap();
    let dir2 = Direction3D::from_vector(vec2).unwrap();

    // 両方とも正規化されて同じ方向になる
    assert!((dir1.x() - dir2.x()).abs() < TestType::EPSILON);
    assert!((dir1.y() - dir2.y()).abs() < TestType::EPSILON);
    assert!((dir1.z() - dir2.z()).abs() < TestType::EPSILON);
}

#[test]
fn a_unit_vector_is_kept_as_it_is() {
    // 正規化した方向の値から作り直しても、値が変わらない
    for index in 0..200 {
        let angle = index as f64 * 0.37;
        let vector = Vector3D::new(
            angle.cos() * 3.0,
            angle.sin() * 0.7,
            (angle * 1.3).sin() + 0.1,
        );
        let direction = Direction3D::from_vector(vector).unwrap();
        let again = Direction3D::from_unit_vector(direction.as_vector()).unwrap();
        assert_eq!(again, direction);
    }
    // 単位ベクトルでない値は、正規化する
    let direction = Direction3D::from_unit_vector(Vector3D::new(0.0, 3.0, 4.0)).unwrap();
    assert_eq!(direction.as_vector(), Vector3D::new(0.0, 0.6, 0.8));
    assert!(Direction3D::<f64>::from_unit_vector(Vector3D::new(0.0, 0.0, 0.0)).is_none());
}

#[test]
fn an_orthogonal_direction_is_kept_as_it_is() {
    let axis = Direction3D::from_vector(Vector3D::new(0.2, 0.6, 0.8)).unwrap();
    // 軸の成分を除いて正規化する
    let direction = Direction3D::from_vector(Vector3D::new(1.0, 1.0, 0.0))
        .unwrap()
        .orthogonal_to(axis)
        .unwrap();
    assert!(direction.as_vector().dot(axis.as_vector()).abs() < 1.0e-15);
    // 直交化した方向から作り直しても、値が変わらない
    assert_eq!(direction.orthogonal_to(axis), Some(direction));
    // 軸と平行なら作れない
    assert!(axis.orthogonal_to(axis).is_none());
}
