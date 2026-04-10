use super::planar_and_mesh_family::{
    plane3d_infinite_line3d_collides, plane3d_line_segment3d_collides, plane3d_ray3d_collides,
};
use crate::{InfiniteLine3D, LineSegment3D, Plane3D, Point3D, Ray3D, SphericalSurface3D};
use geo_contracts::{Scalar, SphericalSurface3DProperties};

pub fn ray3d_point3d_collides<T: Scalar>(ray: &Ray3D<T>, point: &Point3D<T>, tolerance: T) -> bool {
    ray.contains_point(point, tolerance)
}

pub fn ray3d_spherical_surface3d_collides<T: Scalar>(
    ray: &Ray3D<T>,
    sphere: &SphericalSurface3D<T>,
    tolerance: T,
) -> bool {
    let center_tuple = SphericalSurface3DProperties::center(sphere);
    let center = Point3D::new(center_tuple.0, center_tuple.1, center_tuple.2);
    ray.distance_to_point(&center) <= SphericalSurface3DProperties::radius(sphere) + tolerance
}

pub fn ray3d_ray3d_collides<T: Scalar>(ray_a: &Ray3D<T>, ray_b: &Ray3D<T>, tolerance: T) -> bool {
    ray_a.origin().distance_to(&ray_b.origin()) <= tolerance
}

pub fn ray3d_line_segment3d_collides<T: Scalar>(
    ray: &Ray3D<T>,
    segment: &LineSegment3D<T>,
    tolerance: T,
) -> bool {
    let d1 = segment.distance_to_point(&ray.origin());
    let d2 = ray.distance_to_point(&segment.start());
    let d3 = ray.distance_to_point(&segment.end());
    d1.min(d2).min(d3) <= tolerance
}

pub fn ray3d_infinite_line3d_collides<T: Scalar>(
    ray: &Ray3D<T>,
    line: &InfiniteLine3D<T>,
    tolerance: T,
) -> bool {
    line.distance_to_point(&ray.origin()) <= tolerance
}

pub fn ray3d_plane3d_collides<T: Scalar>(ray: &Ray3D<T>, plane: &Plane3D<T>, tolerance: T) -> bool {
    plane3d_ray3d_collides(plane, ray, tolerance)
}

pub fn line_segment3d_point3d_collides<T: Scalar>(
    segment: &LineSegment3D<T>,
    point: &Point3D<T>,
    tolerance: T,
) -> bool {
    segment.contains_point(point, tolerance)
}

pub fn line_segment3d_spherical_surface3d_collides<T: Scalar>(
    segment: &LineSegment3D<T>,
    sphere: &SphericalSurface3D<T>,
    tolerance: T,
) -> bool {
    let center_tuple = SphericalSurface3DProperties::center(sphere);
    let center = Point3D::new(center_tuple.0, center_tuple.1, center_tuple.2);
    let dist = segment.distance_to_point(&center);
    (dist - SphericalSurface3DProperties::radius(sphere)).max(T::ZERO) <= tolerance
}

pub fn line_segment3d_line_segment3d_collides<T: Scalar>(
    seg_a: &LineSegment3D<T>,
    seg_b: &LineSegment3D<T>,
    tolerance: T,
) -> bool {
    let d1 = seg_a.distance_to_point(&seg_b.start());
    let d2 = seg_a.distance_to_point(&seg_b.end());
    let d3 = seg_b.distance_to_point(&seg_a.start());
    let d4 = seg_b.distance_to_point(&seg_a.end());
    d1.min(d2).min(d3).min(d4) <= tolerance
}

pub fn line_segment3d_ray3d_collides<T: Scalar>(
    segment: &LineSegment3D<T>,
    ray: &Ray3D<T>,
    tolerance: T,
) -> bool {
    let d1 = ray.distance_to_point(&segment.start());
    let d2 = ray.distance_to_point(&segment.end());
    let d3 = segment.distance_to_point(&ray.origin());
    d1.min(d2).min(d3) <= tolerance
}

pub fn line_segment3d_infinite_line3d_collides<T: Scalar>(
    segment: &LineSegment3D<T>,
    line: &InfiniteLine3D<T>,
    tolerance: T,
) -> bool {
    let d1 = line.distance_to_point(&segment.start());
    let d2 = line.distance_to_point(&segment.end());
    d1.min(d2) <= tolerance
}

pub fn line_segment3d_plane3d_collides<T: Scalar>(
    segment: &LineSegment3D<T>,
    plane: &Plane3D<T>,
    tolerance: T,
) -> bool {
    plane3d_line_segment3d_collides(plane, segment, tolerance)
}

pub fn infinite_line3d_point3d_collides<T: Scalar>(
    line: &InfiniteLine3D<T>,
    point: &Point3D<T>,
    tolerance: T,
) -> bool {
    line.contains_point(point, tolerance)
}

pub fn infinite_line3d_spherical_surface3d_collides<T: Scalar>(
    line: &InfiniteLine3D<T>,
    sphere: &SphericalSurface3D<T>,
    tolerance: T,
) -> bool {
    let center_tuple = SphericalSurface3DProperties::center(sphere);
    let center = Point3D::new(center_tuple.0, center_tuple.1, center_tuple.2);
    line.distance_to_point(&center) <= SphericalSurface3DProperties::radius(sphere) + tolerance
}

pub fn infinite_line3d_infinite_line3d_collides<T: Scalar>(
    line_a: &InfiniteLine3D<T>,
    line_b: &InfiniteLine3D<T>,
    _tolerance: T,
) -> bool {
    !line_a.is_parallel_to(line_b) && line_a.is_coplanar_with(line_b)
}

pub fn infinite_line3d_line_segment3d_collides<T: Scalar>(
    line: &InfiniteLine3D<T>,
    segment: &LineSegment3D<T>,
    tolerance: T,
) -> bool {
    let d1 = line.distance_to_point(&segment.start());
    let d2 = line.distance_to_point(&segment.end());
    d1.min(d2) <= tolerance
}

pub fn infinite_line3d_ray3d_collides<T: Scalar>(
    line: &InfiniteLine3D<T>,
    ray: &Ray3D<T>,
    tolerance: T,
) -> bool {
    line.distance_to_point(&ray.origin()) <= tolerance
}

pub fn infinite_line3d_plane3d_collides<T: Scalar>(
    line: &InfiniteLine3D<T>,
    plane: &Plane3D<T>,
    tolerance: T,
) -> bool {
    plane3d_infinite_line3d_collides(plane, line, tolerance)
}
