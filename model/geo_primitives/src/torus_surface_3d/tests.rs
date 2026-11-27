//! `TorusSurface3D` のテスト

use super::*;
use crate::{Direction3D, Vector3D};
use analysis::test_constants::TOLERANCE_F64;
use geo_contracts::{Bounded, PrimitiveKind, PrimitiveMetadata};
use std::f64::consts::PI;

#[test]
fn test_primitive_kind() {
    let torus = TorusSurface3D::new_standard(Point3D::origin(), 3.0, 1.0).unwrap();
    assert_eq!(torus.primitive_kind(), PrimitiveKind::TorusSurface);
}

#[test]
fn test_bounding_box_standard_torus() {
    let major_radius = 3.0;
    let minor_radius = 1.0;
    let torus =
        TorusSurface3D::new_standard(Point3D::origin(), major_radius, minor_radius).unwrap();

    let bbox = torus.aabb().expect("should have aabb");
    let outer_radius = major_radius + minor_radius; // 4.0

    // 標準トーラス（XY平面、Z軸中心）の境界ボックス
    assert!((bbox.min().x() + outer_radius).abs() < TOLERANCE_F64);
    assert!((bbox.max().x() - outer_radius).abs() < TOLERANCE_F64);
    assert!((bbox.min().y() + outer_radius).abs() < TOLERANCE_F64);
    assert!((bbox.max().y() - outer_radius).abs() < TOLERANCE_F64);
    assert!((bbox.min().z() + minor_radius).abs() < TOLERANCE_F64);
    assert!((bbox.max().z() - minor_radius).abs() < TOLERANCE_F64);
}

#[test]
fn test_bounding_box_rotated_torus() {
    let major_radius = 2.0;
    let minor_radius = 0.5;

    // Y軸回転したトーラス（Z軸がX方向）
    let origin = Point3D::new(1.0, 2.0, 3.0);
    let z_axis = Direction3D::from_vector(Vector3D::new(1.0, 0.0, 0.0)).unwrap(); // X方向
    let x_axis = Direction3D::from_vector(Vector3D::new(0.0, 0.0, 1.0)).unwrap(); // Z方向

    let torus = TorusSurface3D::new(origin, z_axis, x_axis, major_radius, minor_radius).unwrap();
    let bbox = torus.aabb().expect("should have aabb");

    let outer_radius = major_radius + minor_radius; // 2.5

    // 回転後の境界ボックスをチェック
    // X方向（元のZ軸方向）に拡張
    assert!((bbox.min().x() - (origin.x() - minor_radius)).abs() < TOLERANCE_F64);
    assert!((bbox.max().x() - (origin.x() + minor_radius)).abs() < TOLERANCE_F64);

    // Y方向は変化なし
    assert!((bbox.min().y() - (origin.y() - outer_radius)).abs() < TOLERANCE_F64);
    assert!((bbox.max().y() - (origin.y() + outer_radius)).abs() < TOLERANCE_F64);

    // Z方向（元のX軸方向）に拡張
    assert!((bbox.min().z() - (origin.z() - outer_radius)).abs() < TOLERANCE_F64);
    assert!((bbox.max().z() - (origin.z() + outer_radius)).abs() < TOLERANCE_F64);
}

#[test]
fn test_geometric_properties() {
    let torus = TorusSurface3D::new_standard(Point3D::origin(), 5.0, 1.0).unwrap();

    // 境界ボックスが有効
    let bbox = torus.aabb().expect("should have aabb");
    assert!(bbox.min().x() < bbox.max().x());
    assert!(bbox.min().y() < bbox.max().y());
    assert!(bbox.min().z() < bbox.max().z());

    // プリミティブ種類が正しい
    assert_eq!(torus.primitive_kind(), PrimitiveKind::TorusSurface);
}

#[test]
fn test_point_and_normal_at_uv() {
    let torus = TorusSurface3D::new_standard(Point3D::origin(), 3.0, 1.0).unwrap();

    // u は軸回りの角度、v は断面の円の角度
    let cases = [
        (
            0.0,
            0.0,
            Point3D::new(4.0, 0.0, 0.0),
            Vector3D::new(1.0, 0.0, 0.0),
        ),
        (
            0.0,
            PI,
            Point3D::new(2.0, 0.0, 0.0),
            Vector3D::new(-1.0, 0.0, 0.0),
        ),
        (
            PI / 2.0,
            PI / 2.0,
            Point3D::new(0.0, 3.0, 1.0),
            Vector3D::new(0.0, 0.0, 1.0),
        ),
    ];
    for (u, v, point, normal) in cases {
        assert!(torus.point_at_uv(u, v).distance_to(point) < TOLERANCE_F64);
        assert!((torus.normal_at(u, v).as_vector() - normal).length() < TOLERANCE_F64);
    }
}

#[test]
fn a_negative_major_radius_is_the_inner_lemon_part() {
    // 大半径 −3・小半径 5: 管の中心は軸の向こう側。v = 0 で軸から 2、軸の上の先は高さ ±4
    let torus = TorusSurface3D::new_standard(Point3D::origin(), -3.0, 5.0).unwrap();
    assert!(
        torus
            .point_at_uv(0.0, 0.0)
            .distance_to(Point3D::new(2.0, 0.0, 0.0))
            < TOLERANCE_F64
    );
    // 法線は管の中心（軸の向こう側）から離れる向き
    assert!(
        (torus.normal_at(0.0, 0.0).as_vector() - Vector3D::new(1.0, 0.0, 0.0)).length()
            < TOLERANCE_F64
    );
    let tip = (4.0_f64).atan2(3.0);
    assert!(
        torus
            .point_at_uv(1.0, tip)
            .distance_to(Point3D::new(0.0, 0.0, 4.0))
            < TOLERANCE_F64
    );
    // 外形の箱は、レモン形の最も太い所（軸から 2）を含む
    let bbox = torus.aabb().unwrap();
    assert!((bbox.max().x() - 2.0).abs() < TOLERANCE_F64);
    // 大半径が 0・−小半径以下は作らない
    assert!(TorusSurface3D::new_standard(Point3D::origin(), 0.0, 5.0).is_none());
    assert!(TorusSurface3D::new_standard(Point3D::origin(), -5.0, 5.0).is_none());
    assert!(TorusSurface3D::new_standard(Point3D::origin(), -6.0, 5.0).is_none());
}

#[test]
fn distances_to_a_torus_whose_tube_crosses_the_axis_are_to_its_part() {
    let lemon = TorusSurface3D::new_standard(Point3D::origin(), -3.0, 5.0).unwrap();
    // 面の上の点・軸の上の中心（最も太い所まで 2）・先の上の点（先まで 1）
    assert!(lemon.distance_to_point(Point3D::new(0.0, 2.0, 0.0)) < TOLERANCE_F64);
    assert!((lemon.distance_to_point(Point3D::origin()) - 2.0).abs() < TOLERANCE_F64);
    assert!((lemon.distance_to_point(Point3D::new(0.0, 0.0, 5.0)) - 1.0).abs() < TOLERANCE_F64);
    // りんご形（大半径 3・小半径 5）: 軸の上の中心から、へこみの先（高さ ±4）まで 4
    let apple = TorusSurface3D::new_standard(Point3D::origin(), 3.0, 5.0).unwrap();
    assert!((apple.distance_to_point(Point3D::origin()) - 4.0).abs() < TOLERANCE_F64);
    assert!(apple.distance_to_point(Point3D::new(8.0, 0.0, 0.0)) < TOLERANCE_F64);
}

#[test]
fn test_distance_to_point() {
    let torus = TorusSurface3D::new_standard(Point3D::origin(), 3.0, 1.0).unwrap();

    // 断面の円の中心では小半径、中心軸上では中心からの距離は √(R² + 0) - r
    assert!((torus.distance_to_point(Point3D::new(3.0, 0.0, 0.0)) - 1.0).abs() < TOLERANCE_F64);
    assert!((torus.distance_to_point(Point3D::origin()) - 2.0).abs() < TOLERANCE_F64);
    assert!(torus.distance_to_point(Point3D::new(0.0, 4.0, 0.0)) < TOLERANCE_F64);
}
