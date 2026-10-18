//! `TorusSolid3D` のテスト

use super::*;
use crate::{Direction3D, Point3D, Vector3D};
use analysis::test_constants::TOLERANCE_F64;
use geo_contracts::{Bounded, PrimitiveKind, PrimitiveMetadata};

#[test]
fn test_primitive_kind() {
    let torus = TorusSolid3D::new_standard(Point3D::origin(), 3.0, 1.0).unwrap();
    assert_eq!(torus.primitive_kind(), PrimitiveKind::TorusSolid);
}

#[test]
fn test_bounding_box_standard_torus() {
    let torus = TorusSolid3D::new_standard(Point3D::origin(), 3.0, 1.0).unwrap();
    let bbox = torus.aabb().expect("should have aabb");

    // 標準トーラス（Z軸中心）の境界ボックス
    // 総半径 = 3.0 + 1.0 = 4.0
    assert!((bbox.min().x() - (-4.0)).abs() < TOLERANCE_F64);
    assert!((bbox.max().x() - 4.0).abs() < TOLERANCE_F64);
    assert!((bbox.min().y() - (-4.0)).abs() < TOLERANCE_F64);
    assert!((bbox.max().y() - 4.0).abs() < TOLERANCE_F64);
    assert!((bbox.min().z() - (-1.0)).abs() < TOLERANCE_F64);
    assert!((bbox.max().z() - 1.0).abs() < TOLERANCE_F64);
}

#[test]
fn test_bounding_box_rotated_torus() {
    // X軸方向に45度回転したトーラス
    let sqrt2_half = (2.0_f64).sqrt() / 2.0;
    let z_axis = Direction3D::from_vector(Vector3D::new(sqrt2_half, 0.0, sqrt2_half)).unwrap();
    let x_axis = Direction3D::from_vector(Vector3D::new(-sqrt2_half, 0.0, sqrt2_half)).unwrap();

    let torus = TorusSolid3D::new(Point3D::origin(), z_axis, x_axis, 3.0, 1.0).unwrap();

    let bbox = torus.aabb().expect("should have aabb");

    // 回転により境界が変化することを確認
    assert!(bbox.max().x() > 3.5);
    assert!(bbox.max().z() > 3.5);
}

#[test]
fn test_geometric_properties() {
    let torus = TorusSolid3D::new_standard(Point3D::origin(), 2.0, 0.5).unwrap();

    assert_eq!(torus.primitive_kind(), PrimitiveKind::TorusSolid);

    // 外径 2.5、軸方向の厚み ±0.5 の境界ボックス
    let bbox = torus.aabb().expect("should have aabb");
    assert_eq!(bbox.min(), Point3D::new(-2.5, -2.5, -0.5));
    assert_eq!(bbox.max(), Point3D::new(2.5, 2.5, 0.5));
}

#[test]
fn test_new_rejects_spindle_torus() {
    // 大半径が小半径以下の場合は立体にならない
    assert!(TorusSolid3D::new_standard(Point3D::origin(), 1.0, 1.0).is_none());
    assert!(TorusSolid3D::new_standard(Point3D::origin(), 1.0, 2.0).is_none());
}
