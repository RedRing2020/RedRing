use super::shared::triangle3d_vertex_points;
use crate::{InfiniteLine3D, LineSegment3D, Plane3D, Point3D, Ray3D, Triangle3D, TriangleMesh3D};
use geo_contracts::{InfiniteLine3DProperties, Scalar};

pub fn triangle3d_point3d_collides<T: Scalar>(
    triangle: &Triangle3D<T>,
    point: &Point3D<T>,
    tolerance: T,
) -> bool {
    crate::distance::triangle3d_point3d_distance(triangle, point) <= tolerance
}

pub fn triangle3d_line_segment3d_collides<T: Scalar>(
    triangle: &Triangle3D<T>,
    segment: &LineSegment3D<T>,
    tolerance: T,
) -> bool {
    crate::distance::triangle3d_point3d_distance(triangle, &segment.start()) <= tolerance
        || crate::distance::triangle3d_point3d_distance(triangle, &segment.end()) <= tolerance
}

pub fn line_segment3d_triangle3d_collides<T: Scalar>(
    segment: &LineSegment3D<T>,
    triangle: &Triangle3D<T>,
    tolerance: T,
) -> bool {
    triangle3d_line_segment3d_collides(triangle, segment, tolerance)
}

pub fn triangle3d_ray3d_collides<T: Scalar>(
    triangle: &Triangle3D<T>,
    ray: &Ray3D<T>,
    tolerance: T,
) -> bool {
    crate::distance::triangle3d_point3d_distance(triangle, &ray.origin()) <= tolerance
}

pub fn ray3d_triangle3d_collides<T: Scalar>(
    ray: &Ray3D<T>,
    triangle: &Triangle3D<T>,
    tolerance: T,
) -> bool {
    triangle3d_ray3d_collides(triangle, ray, tolerance)
}

pub fn triangle3d_triangle3d_collides<T: Scalar>(
    triangle_a: &Triangle3D<T>,
    triangle_b: &Triangle3D<T>,
    tolerance: T,
) -> bool {
    triangle3d_vertex_points(triangle_a)
        .into_iter()
        .any(|point| crate::distance::triangle3d_point3d_distance(triangle_b, &point) <= tolerance)
        || triangle3d_vertex_points(triangle_b)
            .into_iter()
            .any(|point| {
                crate::distance::triangle3d_point3d_distance(triangle_a, &point) <= tolerance
            })
}

pub fn triangle_mesh3d_point3d_collides<T: Scalar>(
    mesh: &TriangleMesh3D<T>,
    point: &Point3D<T>,
    tolerance: T,
) -> bool {
    crate::distance::triangle_mesh3d_point3d_distance(mesh, point) <= tolerance
}

pub fn plane3d_point3d_collides<T: Scalar>(
    plane: &Plane3D<T>,
    point: &Point3D<T>,
    tolerance: T,
) -> bool {
    plane.contains_point(*point, tolerance)
}

pub fn plane3d_line_segment3d_collides<T: Scalar>(
    plane: &Plane3D<T>,
    segment: &LineSegment3D<T>,
    tolerance: T,
) -> bool {
    let start_dist = plane.distance_to_point(segment.start());
    let end_dist = plane.distance_to_point(segment.end());
    (start_dist * end_dist <= T::ZERO)
        || (start_dist.abs() <= tolerance)
        || (end_dist.abs() <= tolerance)
}

pub fn plane3d_ray3d_collides<T: Scalar>(plane: &Plane3D<T>, ray: &Ray3D<T>, tolerance: T) -> bool {
    let ray_dir = ray.direction_vector();
    let normal_vec = plane.normal().as_vector();
    let denom = ray_dir.dot(&normal_vec);
    if denom.abs() > tolerance {
        let origin_dist = plane.distance_to_point(ray.origin());
        origin_dist.abs() <= tolerance || (origin_dist * denom <= T::ZERO)
    } else {
        plane.contains_point(ray.origin(), tolerance)
    }
}

pub fn plane3d_infinite_line3d_collides<T: Scalar>(
    plane: &Plane3D<T>,
    line: &InfiniteLine3D<T>,
    tolerance: T,
) -> bool {
    let (dx, dy, dz) = InfiniteLine3DProperties::direction(line);
    let line_dir_vec = crate::Vector3D::new(dx, dy, dz);
    let normal_vec = plane.normal().as_vector();
    let denom = line_dir_vec.dot(&normal_vec);
    if denom.abs() > tolerance {
        true
    } else {
        let (px, py, pz) = InfiniteLine3DProperties::point(line);
        plane.contains_point(crate::Point3D::new(px, py, pz), tolerance)
    }
}

pub fn plane3d_plane3d_collides<T: Scalar>(
    plane_a: &Plane3D<T>,
    plane_b: &Plane3D<T>,
    tolerance: T,
) -> bool {
    let normal1 = plane_a.normal().as_vector();
    let normal2 = plane_b.normal().as_vector();
    let cross = normal1.cross(&normal2);
    if cross.length() > tolerance {
        true
    } else {
        let dist = plane_a.distance_to_point(plane_b.origin());
        dist.abs() <= tolerance
    }
}
