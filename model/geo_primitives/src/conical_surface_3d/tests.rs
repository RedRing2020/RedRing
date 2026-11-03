//! ConicalSurface3D のテスト

use super::*;
use analysis::test_constants::TOLERANCE_F64;
use geo_contracts::{Bounded, PrimitiveKind, PrimitiveMetadata};
use std::f64::consts::{FRAC_PI_2, FRAC_PI_4, PI};

/// 基準点が原点、半径 1、半頂角 45° の Z 軸方向の円錐面（頂点は (0, 0, -1)）
fn cone_45() -> ConicalSurface3D<f64> {
    ConicalSurface3D::new_standard(Point3D::origin(), 1.0, Angle::from_radians(FRAC_PI_4)).unwrap()
}

#[test]
fn test_creation_and_properties() {
    let cone = cone_45();
    assert_eq!(cone.primitive_kind(), PrimitiveKind::ConicalSurface);
    assert_eq!(cone.center(), Point3D::origin());
    assert_eq!(cone.radius(), 1.0);
    assert_eq!(cone.semi_angle(), Angle::from_radians(FRAC_PI_4));
    assert!(cone.apex().distance_to(Point3D::new(0.0, 0.0, -1.0)) < TOLERANCE_F64);
}

#[test]
fn test_invalid_creation() {
    let origin = Point3D::origin();
    assert!(ConicalSurface3D::new_standard(origin, 0.0, Angle::from_radians(0.3)).is_none());
    assert!(ConicalSurface3D::new_standard(origin, 1.0, Angle::from_radians(0.0)).is_none());
    assert!(ConicalSurface3D::new_standard(origin, 1.0, Angle::from_radians(FRAC_PI_2)).is_none());
}

#[test]
fn test_point_at_uv() {
    let cone = cone_45();

    // 半径は軸方向に tan(半頂角) の割合で増え、頂点の先では反対側に出る
    let cases = [
        (0.0, 0.0, Point3D::new(1.0, 0.0, 0.0)),
        (FRAC_PI_2, 1.0, Point3D::new(0.0, 2.0, 1.0)),
        (0.0, -1.0, Point3D::new(0.0, 0.0, -1.0)),
        (0.0, -3.0, Point3D::new(-2.0, 0.0, -3.0)),
    ];
    for (u, v, expected) in cases {
        assert!(cone.point_at_uv(u, v).distance_to(expected) < TOLERANCE_F64);
    }
}

#[test]
fn test_normal_at() {
    let cone = cone_45();
    let s = 0.5_f64.sqrt();

    // 母線に直交し、径方向に外向き
    let normal = cone.normal_at(0.0, 0.0).unwrap().as_vector();
    assert!((normal - Vector3D::new(s, 0.0, -s)).length() < TOLERANCE_F64);

    // 頂点の反対側の円錐でも、∂u × ∂v の向き（その点での径方向に外向き）
    let normal = cone.normal_at(0.0, -3.0).unwrap().as_vector();
    assert!((normal - Vector3D::new(-s, 0.0, s)).length() < TOLERANCE_F64);

    // 頂点では定まらない
    assert!(cone.normal_at(PI, -1.0).is_none());
}

#[test]
fn test_distance_to_point() {
    let cone = cone_45();
    let s = 0.5_f64.sqrt();

    let cases = [
        // 面上の点
        (Point3D::new(2.0, 0.0, 1.0), 0.0),
        // 母線 ρ = z + 1 から断面で外向きに離れた点 (ρ, z) = (3, 1) は、距離 1/√2
        (Point3D::new(0.0, 3.0, 1.0), s),
        // 軸上の点は両側の母線まで (z + 1) sin(45°)
        (Point3D::new(0.0, 0.0, 1.0), 2.0 * s),
        // 頂点の反対側の円錐の内側の軸上の点
        (Point3D::new(0.0, 0.0, -3.0), 2.0 * s),
        // 頂点
        (Point3D::new(0.0, 0.0, -1.0), 0.0),
    ];
    for (point, expected) in cases {
        assert!(
            (cone.distance_to_point(point) - expected).abs() < TOLERANCE_F64,
            "{point:?}"
        );
    }
}

#[test]
fn test_has_no_bounding_box() {
    // 頂点の両側に無限に続く面のため境界ボックスを持たない
    assert!(cone_45().aabb().is_none());
}
