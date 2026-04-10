use super::shared::triangle3d_vertex_points;
use crate::{
    Circle3D, CylindricalSolid3D, CylindricalSurface3D, InfiniteLine3D, LineSegment3D, Plane3D,
    Point3D, Ray3D, Triangle3D,
};
use geo_contracts::{
    Circle3DProperties, CylindricalSolid3DProperties, CylindricalSurface3DProperties,
    InfiniteLine3DProperties, Scalar,
};

pub fn cylindrical_solid3d_point3d_collides<T: Scalar>(
    cyl: &CylindricalSolid3D<T>,
    point: &Point3D<T>,
    tolerance: T,
) -> bool {
    crate::distance::cylindrical_solid3d_point3d_distance(cyl, point) <= tolerance
}

pub fn cylindrical_solid3d_circle3d_collides<T: Scalar>(
    cyl: &CylindricalSolid3D<T>,
    circle: &Circle3D<T>,
    tolerance: T,
) -> bool {
    let (cx, cy, cz) = circle.center();
    let center = Point3D::new(cx, cy, cz);
    crate::distance::cylindrical_solid3d_point3d_distance(cyl, &center) <= tolerance
}

pub fn cylindrical_solid3d_line_segment3d_collides<T: Scalar>(
    cyl: &CylindricalSolid3D<T>,
    segment: &LineSegment3D<T>,
    tolerance: T,
) -> bool {
    let s = segment.start();
    let e = segment.end();
    crate::distance::cylindrical_solid3d_point3d_distance(cyl, &s) <= tolerance
        || crate::distance::cylindrical_solid3d_point3d_distance(cyl, &e) <= tolerance
}

pub fn cylindrical_solid3d_infinite_line3d_collides<T: Scalar>(
    cyl: &CylindricalSolid3D<T>,
    line: &InfiniteLine3D<T>,
    tolerance: T,
) -> bool {
    let (px, py, pz) = line.point();
    let point = Point3D::new(px, py, pz);
    crate::distance::cylindrical_solid3d_point3d_distance(cyl, &point) <= tolerance
}

pub fn cylindrical_solid3d_ray3d_collides<T: Scalar>(
    cyl: &CylindricalSolid3D<T>,
    ray: &Ray3D<T>,
    tolerance: T,
) -> bool {
    let o = ray.origin();
    crate::distance::cylindrical_solid3d_point3d_distance(cyl, &o) <= tolerance
}

pub fn cylindrical_solid3d_triangle3d_collides<T: Scalar>(
    cyl: &CylindricalSolid3D<T>,
    triangle: &Triangle3D<T>,
    tolerance: T,
) -> bool {
    triangle3d_vertex_points(triangle).into_iter().any(|point| {
        crate::distance::cylindrical_solid3d_point3d_distance(cyl, &point) <= tolerance
    })
}

pub fn cylindrical_solid3d_plane3d_collides<T: Scalar>(
    cyl: &CylindricalSolid3D<T>,
    plane: &Plane3D<T>,
    tolerance: T,
) -> bool {
    let (cx, cy, cz) = <CylindricalSolid3D<T> as CylindricalSolid3DProperties<T>>::center(cyl);
    let center = Point3D::new(cx, cy, cz);
    let dist_center = plane.distance_to_point(center).abs();
    let radius = <CylindricalSolid3D<T> as CylindricalSolid3DProperties<T>>::radius(cyl);
    let height = <CylindricalSolid3D<T> as CylindricalSolid3DProperties<T>>::height(cyl);
    dist_center <= tolerance + radius + height
}

pub fn cylindrical_solid3d_cylindrical_solid3d_collides<T: Scalar>(
    cyl_a: &CylindricalSolid3D<T>,
    cyl_b: &CylindricalSolid3D<T>,
    tolerance: T,
) -> bool {
    let (ax, ay, az) = <CylindricalSolid3D<T> as CylindricalSolid3DProperties<T>>::center(cyl_a);
    let (bx, by, bz) = <CylindricalSolid3D<T> as CylindricalSolid3DProperties<T>>::center(cyl_b);
    let center_a = Point3D::new(ax, ay, az);
    let center_b = Point3D::new(bx, by, bz);
    let dist = center_a.distance_to(&center_b);
    let max_a = <CylindricalSolid3D<T> as CylindricalSolid3DProperties<T>>::radius(cyl_a)
        + <CylindricalSolid3D<T> as CylindricalSolid3DProperties<T>>::height(cyl_a);
    let max_b = <CylindricalSolid3D<T> as CylindricalSolid3DProperties<T>>::radius(cyl_b)
        + <CylindricalSolid3D<T> as CylindricalSolid3DProperties<T>>::height(cyl_b);
    dist <= max_a + max_b + tolerance
}

pub fn cylindrical_surface3d_point3d_collides<T: Scalar>(
    cyl: &CylindricalSurface3D<T>,
    point: &Point3D<T>,
    tolerance: T,
) -> bool {
    crate::distance::cylindrical_surface3d_point3d_distance(cyl, point) <= tolerance
}

pub fn cylindrical_surface3d_circle3d_collides<T: Scalar>(
    cyl: &CylindricalSurface3D<T>,
    circle: &Circle3D<T>,
    tolerance: T,
) -> bool {
    let (cx, cy, cz) = circle.center();
    let center = Point3D::new(cx, cy, cz);
    crate::distance::cylindrical_surface3d_point3d_distance(cyl, &center) <= tolerance
}

pub fn cylindrical_surface3d_line_segment3d_collides<T: Scalar>(
    cyl: &CylindricalSurface3D<T>,
    segment: &LineSegment3D<T>,
    tolerance: T,
) -> bool {
    let s = segment.start();
    let e = segment.end();
    crate::distance::cylindrical_surface3d_point3d_distance(cyl, &s) <= tolerance
        || crate::distance::cylindrical_surface3d_point3d_distance(cyl, &e) <= tolerance
}

pub fn cylindrical_surface3d_ray3d_collides<T: Scalar>(
    cyl: &CylindricalSurface3D<T>,
    ray: &Ray3D<T>,
    tolerance: T,
) -> bool {
    let o = ray.origin();
    crate::distance::cylindrical_surface3d_point3d_distance(cyl, &o) <= tolerance
}

pub fn cylindrical_surface3d_infinite_line3d_collides<T: Scalar>(
    cyl: &CylindricalSurface3D<T>,
    line: &InfiniteLine3D<T>,
    tolerance: T,
) -> bool {
    let (px, py, pz) = line.point();
    let point = Point3D::new(px, py, pz);
    crate::distance::cylindrical_surface3d_point3d_distance(cyl, &point) <= tolerance
}

pub fn cylindrical_surface3d_triangle3d_collides<T: Scalar>(
    cyl: &CylindricalSurface3D<T>,
    triangle: &Triangle3D<T>,
    tolerance: T,
) -> bool {
    triangle3d_vertex_points(triangle).into_iter().any(|point| {
        crate::distance::cylindrical_surface3d_point3d_distance(cyl, &point) <= tolerance
    })
}

pub fn cylindrical_surface3d_plane3d_collides<T: Scalar>(
    cyl: &CylindricalSurface3D<T>,
    plane: &Plane3D<T>,
    tolerance: T,
) -> bool {
    let (cx, cy, cz) = <CylindricalSurface3D<T> as CylindricalSurface3DProperties<T>>::center(cyl);
    let center = Point3D::new(cx, cy, cz);
    let dist = plane.distance_to_point(center).abs();
    let radius = <CylindricalSurface3D<T> as CylindricalSurface3DProperties<T>>::radius(cyl);
    dist <= radius + tolerance
}

pub fn cylindrical_surface3d_cylindrical_surface3d_collides<T: Scalar>(
    cyl_a: &CylindricalSurface3D<T>,
    cyl_b: &CylindricalSurface3D<T>,
    tolerance: T,
) -> bool {
    let (ax, ay, az) =
        <CylindricalSurface3D<T> as CylindricalSurface3DProperties<T>>::center(cyl_a);
    let (bx, by, bz) =
        <CylindricalSurface3D<T> as CylindricalSurface3DProperties<T>>::center(cyl_b);
    let center_a = Point3D::new(ax, ay, az);
    let center_b = Point3D::new(bx, by, bz);
    let dist = center_a.distance_to(&center_b);
    let radius_sum = <CylindricalSurface3D<T> as CylindricalSurface3DProperties<T>>::radius(cyl_a)
        + <CylindricalSurface3D<T> as CylindricalSurface3DProperties<T>>::radius(cyl_b);
    dist <= radius_sum + tolerance
}
