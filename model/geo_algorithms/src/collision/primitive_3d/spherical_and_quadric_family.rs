use super::shared::triangle3d_vertex_points;
use crate::{
    Circle3D, EllipsoidalSolid3D, EllipsoidalSurface3D, InfiniteLine3D, LineSegment3D, Plane3D,
    Point3D, Ray3D, SphericalSolid3D, TorusSolid3D, TorusSurface3D, Triangle3D,
};
use geo_contracts::{
    Circle3DProperties, EllipsoidalSolid3DProperties, InfiniteLine3DProperties, Scalar,
    SphericalSolid3DProperties,
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
    let (cx, cy, cz) = circle.center();
    let center = Point3D::new(cx, cy, cz);
    crate::distance::spherical_solid3d_point3d_distance(sphere, &center)
        <= circle.radius() + tolerance
}

pub fn spherical_solid3d_line_segment3d_collides<T: Scalar>(
    sphere: &SphericalSolid3D<T>,
    segment: &LineSegment3D<T>,
    tolerance: T,
) -> bool {
    sphere.distance_to_line_segment(&segment.start(), &segment.end()) <= tolerance
}

pub fn spherical_solid3d_ray3d_collides<T: Scalar>(
    sphere: &SphericalSolid3D<T>,
    ray: &Ray3D<T>,
    tolerance: T,
) -> bool {
    sphere.distance_to_ray(&ray.origin(), &ray.direction_vector()) <= tolerance
}

pub fn spherical_solid3d_infinite_line3d_collides<T: Scalar>(
    sphere: &SphericalSolid3D<T>,
    line: &InfiniteLine3D<T>,
    tolerance: T,
) -> bool {
    let (px, py, pz) = line.point();
    let line_pt = Point3D::new(px, py, pz);
    let (dx, dy, dz) = line.direction();
    let line_dir = crate::Vector3D::new(dx, dy, dz);
    sphere.distance_to_infinite_line(&line_pt, &line_dir) <= tolerance
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
    let (sc, sy, sz) = SphericalSolid3DProperties::center(sphere);
    let center = Point3D::new(sc, sy, sz);
    let radius = SphericalSolid3DProperties::radius(sphere);
    plane.distance_to_point(center).abs() <= radius + tolerance
}

pub fn spherical_solid3d_spherical_solid3d_collides<T: Scalar>(
    sphere_a: &SphericalSolid3D<T>,
    sphere_b: &SphericalSolid3D<T>,
    tolerance: T,
) -> bool {
    let (ax, ay, az) = SphericalSolid3DProperties::center(sphere_a);
    let (bx, by, bz) = SphericalSolid3DProperties::center(sphere_b);
    let center_a = Point3D::new(ax, ay, az);
    let center_b = Point3D::new(bx, by, bz);
    let dist = center_a.distance_to(&center_b);
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
    let start = segment.start();
    let end = segment.end();
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
    let (px, py, pz) = line.point();
    let pt = Point3D::new(px, py, pz);
    let d = crate::distance::ellipsoidal_solid3d_point3d_distance(ellipsoid, &pt);
    d <= tolerance
}

pub fn ellipsoidal_solid3d_plane3d_collides<T: Scalar>(
    ellipsoid: &EllipsoidalSolid3D<T>,
    plane: &Plane3D<T>,
    tolerance: T,
) -> bool {
    let (cx, cy, cz) =
        <EllipsoidalSolid3D<T> as EllipsoidalSolid3DProperties<T>>::center(ellipsoid);
    let center = Point3D::new(cx, cy, cz);
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
    let (ax, ay, az) = <EllipsoidalSolid3D<T> as EllipsoidalSolid3DProperties<T>>::center(ell_a);
    let (bx, by, bz) = <EllipsoidalSolid3D<T> as EllipsoidalSolid3DProperties<T>>::center(ell_b);
    let center_a = Point3D::new(ax, ay, az);
    let center_b = Point3D::new(bx, by, bz);
    let center_dist = center_a.distance_to(&center_b);
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
