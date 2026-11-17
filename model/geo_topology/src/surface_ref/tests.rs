use super::*;
use geo_contracts::{
    ConicalSurface3DConstructor, CylindricalSurface3DConstructor, Plane3DConstructor,
    SphericalSurface3DConstructor, TorusSurface3DConstructor,
};
use geo_primitives::Angle;

fn surfaces() -> Vec<SurfaceRef<f64>> {
    let origin = Point3D::new(1.0, 2.0, 3.0);
    vec![
        SurfaceRef::Plane(
            TopoPlane3D::from_three_points(
                origin,
                Point3D::new(2.0, 2.0, 3.0),
                Point3D::new(1.0, 3.0, 4.0),
            )
            .unwrap(),
        ),
        SurfaceRef::Cylinder(TopoCylindricalSurface3D::new_standard(origin, 2.0).unwrap()),
        SurfaceRef::Cone(
            TopoConicalSurface3D::new_standard(origin, 2.0, Angle::from_degrees(30.0)).unwrap(),
        ),
        SurfaceRef::Sphere(TopoSphericalSurface3D::new_standard(origin, 2.0).unwrap()),
        SurfaceRef::Torus(TopoTorusSurface3D::new_standard(origin, 5.0, 2.0).unwrap()),
    ]
}

#[test]
fn the_normal_points_along_the_cross_product_of_the_parameter_derivatives() {
    let step = 1.0e-6;
    for surface in surfaces() {
        for (u, v) in [(0.3, 0.4), (2.0, -0.7), (4.5, 1.1)] {
            let point = |u: f64, v: f64| surface.point_at_uv(u, v).unwrap();
            let du = (point(u + step, v) - point(u - step, v)) * (0.5 / step);
            let dv = (point(u, v + step) - point(u, v - step)) * (0.5 / step);
            let expected = du.cross(dv).normalize();
            let normal = surface.normal_at_uv(u, v).unwrap();
            assert!((normal.length() - 1.0).abs() < 1.0e-9);
            assert!(
                normal.dot(expected) > 1.0 - 1.0e-6,
                "{surface:?} at ({u}, {v})"
            );
        }
    }
}

#[test]
fn the_poles_are_the_singular_lines() {
    for surface in surfaces() {
        for pole in surface.poles() {
            let a = surface.point_at_uv(0.0, pole).unwrap();
            let b = surface.point_at_uv(2.0, pole).unwrap();
            assert!(a.distance_to(b) < 1.0e-9);
            assert_eq!(surface.singular_v(a, 1.0e-9), Some(pole));
        }
    }
    assert_eq!(surfaces()[3].poles().len(), 2);
    assert_eq!(surfaces()[2].poles().len(), 1);
}
