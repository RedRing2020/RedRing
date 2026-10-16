//! `TorusSurface3D` のテスト

use super::*;
use crate::{Direction3D, Vector3D};
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
    assert!((bbox.min().x() + outer_radius).abs() < 1e-10);
    assert!((bbox.max().x() - outer_radius).abs() < 1e-10);
    assert!((bbox.min().y() + outer_radius).abs() < 1e-10);
    assert!((bbox.max().y() - outer_radius).abs() < 1e-10);
    assert!((bbox.min().z() + minor_radius).abs() < 1e-10);
    assert!((bbox.max().z() - minor_radius).abs() < 1e-10);
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
    assert!((bbox.min().x() - (origin.x() - minor_radius)).abs() < 1e-10);
    assert!((bbox.max().x() - (origin.x() + minor_radius)).abs() < 1e-10);

    // Y方向は変化なし
    assert!((bbox.min().y() - (origin.y() - outer_radius)).abs() < 1e-10);
    assert!((bbox.max().y() - (origin.y() + outer_radius)).abs() < 1e-10);

    // Z方向（元のX軸方向）に拡張
    assert!((bbox.min().z() - (origin.z() - outer_radius)).abs() < 1e-10);
    assert!((bbox.max().z() - (origin.z() + outer_radius)).abs() < 1e-10);
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
        assert!(torus.point_at_uv(u, v).distance_to(point) < 1e-12);
        assert!((torus.normal_at(u, v).as_vector() - normal).length() < 1e-12);
    }
}

#[test]
fn test_distance_to_point() {
    let torus = TorusSurface3D::new_standard(Point3D::origin(), 3.0, 1.0).unwrap();

    // 断面の円の中心では小半径、中心軸上では中心からの距離は √(R² + 0) - r
    assert!((torus.distance_to_point(Point3D::new(3.0, 0.0, 0.0)) - 1.0).abs() < 1e-12);
    assert!((torus.distance_to_point(Point3D::origin()) - 2.0).abs() < 1e-12);
    assert!(torus.distance_to_point(Point3D::new(0.0, 4.0, 0.0)) < 1e-12);
}
