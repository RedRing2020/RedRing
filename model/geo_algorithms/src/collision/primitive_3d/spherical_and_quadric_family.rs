use super::shared::triangle3d_vertex_points;
use crate::{
    Circle3D, EllipsoidalSolid3D, EllipsoidalSurface3D, InfiniteLine3D, LineSegment3D, Plane3D,
    Point3D, Ray3D, SphericalSolid3D, TorusSolid3D, TorusSurface3D, Triangle3D,
};
use geo_contracts::{
    Circle3DProperties, EllipsoidalSolid3DProperties, InfiniteLine3DDistance,
    InfiniteLine3DProperties, LineSegment3DDistance, LineSegment3DProperties, Plane3DDistance,
    Ray3DDistance, Ray3DProperties, Scalar, SphericalSolid3DProperties,
};

pub fn spherical_solid3d_point3d_collides<T: Scalar>(
    sphere: &SphericalSolid3D<T>,
    point: &Point3D<T>,
    tolerance: T,
) -> bool {
    crate::distance::spherical_solid3d_point3d_distance(sphere, point) <= tolerance
}

pub fn spherical_solid3d_circle3d_collides<T: Scalar>(
    sphere: &SphericalSolid3D<T>,
    circle: &Circle3D<T>,
    tolerance: T,
) -> bool {
    let center = circle.center();
    crate::distance::spherical_solid3d_point3d_distance(sphere, &center)
        <= circle.radius() + tolerance
}

pub fn spherical_solid3d_line_segment3d_collides<T: Scalar>(
    sphere: &SphericalSolid3D<T>,
    segment: &LineSegment3D<T>,
    tolerance: T,
) -> bool {
    segment.distance_to_point(sphere.center()) <= sphere.radius() + tolerance
}

pub fn spherical_solid3d_ray3d_collides<T: Scalar>(
    sphere: &SphericalSolid3D<T>,
    ray: &Ray3D<T>,
    tolerance: T,
) -> bool {
    ray.distance_to_point(sphere.center()) <= sphere.radius() + tolerance
}

pub fn spherical_solid3d_infinite_line3d_collides<T: Scalar>(
    sphere: &SphericalSolid3D<T>,
    line: &InfiniteLine3D<T>,
    tolerance: T,
) -> bool {
    line.distance_to_point(sphere.center()) <= sphere.radius() + tolerance
}

pub fn spherical_solid3d_triangle3d_collides<T: Scalar>(
    sphere: &SphericalSolid3D<T>,
    triangle: &Triangle3D<T>,
    tolerance: T,
) -> bool {
    triangle3d_vertex_points(triangle).into_iter().any(|point| {
        crate::distance::spherical_solid3d_point3d_distance(sphere, &point) <= tolerance
    })
}

pub fn spherical_solid3d_plane3d_collides<T: Scalar>(
    sphere: &SphericalSolid3D<T>,
    plane: &Plane3D<T>,
    tolerance: T,
) -> bool {
    let center = SphericalSolid3DProperties::center(sphere);
    let radius = SphericalSolid3DProperties::radius(sphere);
    plane.distance_to_point(center).abs() <= radius + tolerance
}

pub fn spherical_solid3d_spherical_solid3d_collides<T: Scalar>(
    sphere_a: &SphericalSolid3D<T>,
    sphere_b: &SphericalSolid3D<T>,
    tolerance: T,
) -> bool {
    let center_a = SphericalSolid3DProperties::center(sphere_a);
    let center_b = SphericalSolid3DProperties::center(sphere_b);
    let dist = center_a.distance_to(center_b);
    let r1 = SphericalSolid3DProperties::radius(sphere_a);
    let r2 = SphericalSolid3DProperties::radius(sphere_b);
    dist <= r1 + r2 + tolerance
}

pub fn ellipsoidal_solid3d_point3d_collides<T: Scalar>(
    ellipsoid: &EllipsoidalSolid3D<T>,
    point: &Point3D<T>,
    tolerance: T,
) -> bool {
    crate::distance::ellipsoidal_solid3d_point3d_distance(ellipsoid, point) <= tolerance
}

pub fn ellipsoidal_solid3d_line_segment3d_collides<T: Scalar>(
    ellipsoid: &EllipsoidalSolid3D<T>,
    segment: &LineSegment3D<T>,
    tolerance: T,
) -> bool {
    let start = segment.start_point();
    let end = segment.end_point();
    let d1 = crate::distance::ellipsoidal_solid3d_point3d_distance(ellipsoid, &start);
    let d2 = crate::distance::ellipsoidal_solid3d_point3d_distance(ellipsoid, &end);
    d1 <= tolerance || d2 <= tolerance
}

pub fn ellipsoidal_solid3d_ray3d_collides<T: Scalar>(
    ellipsoid: &EllipsoidalSolid3D<T>,
    ray: &Ray3D<T>,
    tolerance: T,
) -> bool {
    let origin = ray.origin();
    let d = crate::distance::ellipsoidal_solid3d_point3d_distance(ellipsoid, &origin);
    d <= tolerance
}

pub fn ellipsoidal_solid3d_infinite_line3d_collides<T: Scalar>(
    ellipsoid: &EllipsoidalSolid3D<T>,
    line: &InfiniteLine3D<T>,
    tolerance: T,
) -> bool {
    let pt = line.point();
    let d = crate::distance::ellipsoidal_solid3d_point3d_distance(ellipsoid, &pt);
    d <= tolerance
}

pub fn ellipsoidal_solid3d_plane3d_collides<T: Scalar>(
    ellipsoid: &EllipsoidalSolid3D<T>,
    plane: &Plane3D<T>,
    tolerance: T,
) -> bool {
    let center = <EllipsoidalSolid3D<T> as EllipsoidalSolid3DProperties<T>>::center(ellipsoid);
    let dist = plane.distance_to_point(center).abs();
    let a = <EllipsoidalSolid3D<T> as EllipsoidalSolid3DProperties<T>>::a_radius(ellipsoid);
    let b = <EllipsoidalSolid3D<T> as EllipsoidalSolid3DProperties<T>>::b_radius(ellipsoid);
    let c = <EllipsoidalSolid3D<T> as EllipsoidalSolid3DProperties<T>>::c_radius(ellipsoid);
    let max_radius = a.max(b).max(c);
    dist <= max_radius + tolerance
}

pub fn ellipsoidal_solid3d_ellipsoidal_solid3d_collides<T: Scalar>(
    ell_a: &EllipsoidalSolid3D<T>,
    ell_b: &EllipsoidalSolid3D<T>,
    tolerance: T,
) -> bool {
    let center_a = <EllipsoidalSolid3D<T> as EllipsoidalSolid3DProperties<T>>::center(ell_a);
    let center_b = <EllipsoidalSolid3D<T> as EllipsoidalSolid3DProperties<T>>::center(ell_b);
    let center_dist = center_a.distance_to(center_b);
    let max_1 = {
        let a = <EllipsoidalSolid3D<T> as EllipsoidalSolid3DProperties<T>>::a_radius(ell_a);
        let b = <EllipsoidalSolid3D<T> as EllipsoidalSolid3DProperties<T>>::b_radius(ell_a);
        let c = <EllipsoidalSolid3D<T> as EllipsoidalSolid3DProperties<T>>::c_radius(ell_a);
        a.max(b).max(c)
    };
    let max_2 = {
        let a = <EllipsoidalSolid3D<T> as EllipsoidalSolid3DProperties<T>>::a_radius(ell_b);
        let b = <EllipsoidalSolid3D<T> as EllipsoidalSolid3DProperties<T>>::b_radius(ell_b);
        let c = <EllipsoidalSolid3D<T> as EllipsoidalSolid3DProperties<T>>::c_radius(ell_b);
        a.max(b).max(c)
    };
    center_dist <= max_1 + max_2 + tolerance
}

pub fn ellipsoidal_surface3d_point3d_collides<T: Scalar>(
    ellipsoid: &EllipsoidalSurface3D<T>,
    point: &Point3D<T>,
    tolerance: T,
) -> bool {
    crate::distance::ellipsoidal_surface3d_point3d_distance(ellipsoid, point) <= tolerance
}

pub fn torus_solid3d_point3d_collides<T: Scalar>(
    torus: &TorusSolid3D<T>,
    point: &Point3D<T>,
    tolerance: T,
) -> bool {
    crate::distance::torus_solid3d_point3d_distance(torus, point) <= tolerance
}

pub fn torus_surface3d_point3d_collides<T: Scalar>(
    torus: &TorusSurface3D<T>,
    point: &Point3D<T>,
    tolerance: T,
) -> bool {
    crate::distance::torus_surface3d_point3d_distance(torus, point) <= tolerance
}
