//! CylindricalSolid3D のテスト（STEP準拠）

use crate::{CylindricalSolid3D, Direction3D, Point3D};
use geo_contracts::{
    Bounded, CylindricalSolid3DConstructor, CylindricalSolid3DContainment,
    CylindricalSolid3DDistance, CylindricalSolid3DProperties, PrimitiveKind, PrimitiveMetadata,
    TolerantEq,
};

/// 底面の中心が原点、半径 5・高さ 10、Z 軸方向の円柱
fn standard_solid() -> CylindricalSolid3D<f64> {
    CylindricalSolid3D::new_standard(Point3D::origin(), 5.0, 10.0).unwrap()
}

#[test]
fn test_cylindrical_solid_creation() {
    let solid = CylindricalSolid3D::new(
        Point3D::new(1.0, 2.0, 3.0),
        Direction3D::positive_z(),
        Direction3D::positive_x(),
        5.0,
        10.0,
    )
    .unwrap();

    assert_eq!(solid.center(), Point3D::new(1.0, 2.0, 3.0));
    assert_eq!(solid.radius(), 5.0);
    assert_eq!(solid.height(), 10.0);
    assert_eq!(solid.axis(), Direction3D::positive_z());
    assert_eq!(solid.ref_direction(), Direction3D::positive_x());
}

#[test]
fn test_cylindrical_solid_invalid_creation() {
    let center = Point3D::origin();
    let axis = Direction3D::positive_z();
    let ref_direction = Direction3D::positive_x();

    // 半径・高さが正でない
    assert!(CylindricalSolid3D::new(center, axis, ref_direction, -1.0, 5.0).is_none());
    assert!(CylindricalSolid3D::new(center, axis, ref_direction, 0.0, 5.0).is_none());
    assert!(CylindricalSolid3D::new(center, axis, ref_direction, 5.0, -1.0).is_none());
    assert!(CylindricalSolid3D::new(center, axis, ref_direction, 5.0, 0.0).is_none());

    // 参照方向が軸と平行
    assert!(CylindricalSolid3D::new(center, axis, axis, 5.0, 10.0).is_none());
}

#[test]
fn test_cylindrical_solid_f32() {
    let solid: CylindricalSolid3D<f32> =
        CylindricalSolid3D::new_standard(Point3D::origin(), 5.0, 10.0).unwrap();

    assert_eq!(solid.radius(), 5.0f32);
    assert_eq!(solid.height(), 10.0f32);
}

#[test]
fn test_ref_direction_orthogonalization() {
    let solid: CylindricalSolid3D<f64> = CylindricalSolid3D::new(
        Point3D::origin(),
        Direction3D::positive_z(),
        Direction3D::new(1.0, 0.0, 0.1).unwrap(),
        5.0,
        10.0,
    )
    .unwrap();

    assert!(solid.ref_direction().dot(solid.axis()).abs() < 1e-10);
}

#[test]
fn test_contains_point() {
    let solid = standard_solid();

    assert!(solid.contains_point(Point3D::new(2.0, 2.0, 3.0)));
    assert!(!solid.contains_point(Point3D::new(10.0, 0.0, 0.0)));
    // 高さの範囲外
    assert!(!solid.contains_point(Point3D::new(0.0, 0.0, 15.0)));
}

#[test]
fn test_distance_to_point() {
    let solid = standard_solid();

    let cases = [
        // 内部は 0
        (Point3D::new(0.0, 0.0, 5.0), 0.0),
        // 側面の外側
        (Point3D::new(8.0, 0.0, 5.0), 3.0),
        // 上面の真上と底面の真下は、軸方向の距離だけ
        (Point3D::new(0.0, 0.0, 13.0), 3.0),
        (Point3D::new(2.0, 0.0, -4.0), 4.0),
        // 上面の縁の外側斜め上は、縁までの距離
        (Point3D::new(8.0, 0.0, 14.0), 5.0),
    ];
    for (point, expected) in cases {
        assert!(
            (solid.distance_to_point(point) - expected).abs() < 1e-10,
            "{point:?}"
        );
    }
}

#[test]
fn test_tolerant_eq() {
    let solid1 = standard_solid();
    let solid2 = standard_solid();
    let solid3 = CylindricalSolid3D::new_standard(Point3D::new(2.0, 0.0, 0.0), 5.0, 10.0).unwrap();

    assert!(solid1.tolerant_eq(&solid2, 0.01));
    assert!(!solid1.tolerant_eq(&solid3, 0.01));
}

#[test]
fn test_metadata_and_bounds_capabilities() {
    let solid = CylindricalSolid3D::new_standard(Point3D::new(1.0, 2.0, 3.0), 5.0, 10.0).unwrap();

    assert_eq!(solid.primitive_kind(), PrimitiveKind::CylindricalSolid);

    let bbox = solid.aabb().expect("should have aabb");
    assert_eq!(bbox.min(), Point3D::new(-4.0, -3.0, 3.0));
    assert_eq!(bbox.max(), Point3D::new(6.0, 7.0, 13.0));
}

#[test]
fn test_bounds_tilted_axis() {
    // 軸を X 方向に向けた円柱：X は底面から高さ分、Y・Z は半径分の広がり
    let solid = CylindricalSolid3D::new(
        Point3D::origin(),
        Direction3D::positive_x(),
        Direction3D::positive_z(),
        2.0,
        6.0,
    )
    .unwrap();

    let bbox = solid.aabb().unwrap();
    assert!((bbox.min() - Point3D::new(0.0, -2.0, -2.0)).length() < 1e-12);
    assert!((bbox.max() - Point3D::new(6.0, 2.0, 2.0)).length() < 1e-12);
}
