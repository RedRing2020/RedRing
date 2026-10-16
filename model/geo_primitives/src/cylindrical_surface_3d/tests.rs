//! CylindricalSurface3D のテストスイート

use crate::{CylindricalSurface3D, Direction3D, Point3D, Vector3D};
use analysis::test_constants::TOLERANCE_F32;
use approx::assert_relative_eq;
use geo_contracts::{
    Bounded, CylindricalSurface3DConstructor, CylindricalSurface3DDistance,
    CylindricalSurface3DEvaluation, CylindricalSurface3DProperties, PrimitiveKind,
    PrimitiveMetadata, TolerantEq,
};
use std::f64::consts::PI;

fn create_test_surface() -> CylindricalSurface3D<f64> {
    CylindricalSurface3D::new_standard(Point3D::new(1.0, 2.0, 3.0), 5.0).unwrap()
}

#[test]
fn test_cylindrical_surface_creation() {
    let surface = CylindricalSurface3D::new(
        Point3D::new(0.0, 0.0, 0.0),
        Direction3D::positive_z(),
        Direction3D::positive_x(),
        5.0,
    )
    .unwrap();
    assert_eq!(surface.center(), Point3D::new(0.0, 0.0, 0.0));
    assert_eq!(surface.radius(), 5.0);
}

#[test]
fn test_cylindrical_surface_invalid_creation() {
    let center = Point3D::origin();
    let axis = Direction3D::positive_z();
    let ref_direction = Direction3D::positive_x();

    assert!(CylindricalSurface3D::new(center, axis, ref_direction, -1.0).is_none());
    assert!(CylindricalSurface3D::new(center, axis, ref_direction, 0.0).is_none());
    // 参照方向が軸と平行
    assert!(CylindricalSurface3D::new(center, axis, axis, 5.0).is_none());
}

#[test]
fn test_cylindrical_surface_axis_constructors() {
    let z_surface = CylindricalSurface3D::new_standard(Point3D::new(1.0, 2.0, 3.0), 5.0).unwrap();
    assert_eq!(z_surface.axis(), Direction3D::positive_z());
    assert_eq!(z_surface.ref_direction(), Direction3D::positive_x());
}

#[test]
fn test_cylindrical_surface_f32() {
    let surface: CylindricalSurface3D<f32> =
        CylindricalSurface3D::new_standard(Point3D::new(1.0, 2.0, 3.0), 5.0).unwrap();

    assert_relative_eq!(surface.radius(), 5.0f32, epsilon = TOLERANCE_F32);
    assert_eq!(surface.center(), Point3D::new(1.0f32, 2.0f32, 3.0f32));
}

#[test]
fn test_ref_direction_orthogonalization() {
    let surface = CylindricalSurface3D::new(
        Point3D::origin(),
        Direction3D::positive_z(),
        Direction3D::new(1.0, 0.0, 0.5).unwrap(),
        5.0,
    )
    .unwrap();

    assert_relative_eq!(
        surface.ref_direction().dot(surface.axis()),
        0.0,
        epsilon = 1e-10
    );
}

#[test]
fn test_point_at_uv() {
    let surface = CylindricalSurface3D::new_standard(Point3D::origin(), 5.0).unwrap();

    let cases = [
        (0.0, 0.0, Point3D::new(5.0, 0.0, 0.0)),
        (PI / 2.0, 0.0, Point3D::new(0.0, 5.0, 0.0)),
        (0.0, 10.0, Point3D::new(5.0, 0.0, 10.0)),
    ];
    for (u, v, expected) in cases {
        assert!(surface.point_at_uv(u, v).distance_to(expected) < 1e-10);
    }
}

#[test]
fn test_normal_at() {
    let surface = CylindricalSurface3D::new_standard(Point3D::origin(), 5.0).unwrap();

    let normal = surface.normal_at(0.0, 0.0).as_vector();
    assert!((normal - Vector3D::new(1.0, 0.0, 0.0)).length() < 1e-10);

    let normal = surface.normal_at(PI / 2.0, 5.0).as_vector();
    assert!((normal - Vector3D::new(0.0, 1.0, 0.0)).length() < 1e-10);
}

#[test]
fn test_distance_to_point() {
    let surface = CylindricalSurface3D::new_standard(Point3D::origin(), 5.0).unwrap();

    // 面上・内側・外側・軸上の点。軸方向の位置は距離に影響しない
    let cases = [
        (Point3D::new(5.0, 0.0, 0.0), 0.0),
        (Point3D::new(3.0, 0.0, 0.0), 2.0),
        (Point3D::new(8.0, 0.0, 10.0), 3.0),
        (Point3D::new(0.0, 0.0, -7.0), 5.0),
    ];
    for (point, expected) in cases {
        assert_relative_eq!(surface.distance_to_point(point), expected, epsilon = 1e-10);
    }
}

#[test]
fn test_step_compliance() {
    let center = Point3D::new(10.0, 20.0, 30.0);
    let axis = Direction3D::new(0.0, 1.0, 0.0).unwrap();
    let ref_direction = Direction3D::positive_x();

    let surface = CylindricalSurface3D::new(center, axis, ref_direction, 15.0).unwrap();

    assert_eq!(surface.center(), center);
    assert_eq!(surface.axis(), axis);
    assert_eq!(surface.ref_direction(), ref_direction);
    assert_eq!(surface.radius(), 15.0);

    // u = π/2 の方向は axis × ref_direction
    let point = surface.point_at_uv(PI / 2.0, 0.0);
    let expected = center + axis.as_vector().cross(ref_direction.as_vector()) * 15.0;
    assert!(point.distance_to(expected) < 1e-10);
}

#[test]
fn test_surface_display() {
    let surface = create_test_surface();
    let display_string = format!("{}", surface);
    assert!(display_string.contains("CylindricalSurface3D"));
    assert!(display_string.contains("center"));
    assert!(display_string.contains("axis"));
    assert!(display_string.contains("radius"));
}

#[test]
fn test_parametric_continuity() {
    let surface = CylindricalSurface3D::new_standard(Point3D::origin(), 5.0).unwrap();

    let point_0 = surface.point_at_uv(0.0, 5.0);
    let point_2pi = surface.point_at_uv(2.0 * PI, 5.0);
    assert!(point_0.distance_to(point_2pi) < 1e-10);
}

#[test]
fn test_tolerant_eq() {
    let surface1 = CylindricalSurface3D::new_standard(Point3D::new(0.0, 0.0, 0.0), 5.0).unwrap();
    let surface2 =
        CylindricalSurface3D::new_standard(Point3D::new(0.0001, 0.0001, 0.0001), 5.0001).unwrap();
    let surface3 = CylindricalSurface3D::new_standard(Point3D::new(10.0, 0.0, 0.0), 5.0).unwrap();

    assert!(surface1.tolerant_eq(&surface2, 1e-3));
    assert!(!surface1.tolerant_eq(&surface2, 1e-5));
    assert!(!surface1.tolerant_eq(&surface3, 1e-3));
}

#[test]
fn test_metadata_and_bounds_capabilities() {
    let surface = create_test_surface();

    assert_eq!(surface.primitive_kind(), PrimitiveKind::CylindricalSurface);

    // 軸方向に無限に続く面のため境界ボックスを持たない
    assert!(surface.aabb().is_none());
}
