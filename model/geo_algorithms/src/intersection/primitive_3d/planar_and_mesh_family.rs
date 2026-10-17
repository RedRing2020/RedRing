use crate::{IntersectionResult, LineSegment3D, Plane3D, Point3D, Ray3D, Triangle3D, Vector3D};
use geo_contracts::{
    LineSegment3DProperties, Plane3DDistance, Plane3DProperties, Ray3DConstructor, Ray3DEvaluation,
    Ray3DProperties, Scalar, Triangle3DBoundaryAccess,
};

fn triangle3d_line_segment3d_intersection_raw<T: Scalar>(
    triangle: &Triangle3D<T>,
    segment: &LineSegment3D<T>,
    tolerance: T,
) -> Option<Point3D<T>> {
    let start = segment.start_point();
    let end = segment.end_point();
    let dir = end - start;
    let length = dir.length();
    if length <= tolerance {
        return None;
    }
    let ray = Ray3D::new(start, dir)?;
    let point = triangle3d_ray3d_intersection_raw(triangle, &ray, tolerance)?;

    let t = ray.parameter_for_point(point);
    if t >= T::ZERO && t <= length {
        Some(point)
    } else {
        None
    }
}

pub fn triangle3d_line_segment3d_intersection<T: Scalar>(
    triangle: &Triangle3D<T>,
    segment: &LineSegment3D<T>,
    tolerance: T,
) -> IntersectionResult<T> {
    IntersectionResult::from_option_point(
        triangle3d_line_segment3d_intersection_raw(triangle, segment, tolerance),
        false,
        tolerance,
    )
}

fn triangle3d_ray3d_intersection_raw<T: Scalar>(
    triangle: &Triangle3D<T>,
    ray: &Ray3D<T>,
    tolerance: T,
) -> Option<Point3D<T>> {
    let v0 = triangle.vertex_a();
    let v1 = triangle.vertex_b();
    let v2 = triangle.vertex_c();

    let edge1 = Vector3D::from_points(v0, v1);
    let edge2 = Vector3D::from_points(v0, v2);

    let ray_dir = ray.direction().as_vector();
    let h = ray_dir.cross(edge2);
    let a = edge1.dot(h);

    if a.abs() <= tolerance {
        return None;
    }

    let f = T::ONE / a;
    let s = Vector3D::from_points(v0, ray.origin());
    let u = f * s.dot(h);

    if u < T::ZERO || u > T::ONE {
        return None;
    }

    let q = s.cross(edge1);
    let v = f * ray_dir.dot(q);

    if v < T::ZERO || u + v > T::ONE {
        return None;
    }

    let t = f * edge2.dot(q);
    if t < T::ZERO {
        return None;
    }

    Some(Point3D::new(
        ray.origin().x() + t * ray_dir.x(),
        ray.origin().y() + t * ray_dir.y(),
        ray.origin().z() + t * ray_dir.z(),
    ))
}

pub fn triangle3d_ray3d_intersection<T: Scalar>(
    triangle: &Triangle3D<T>,
    ray: &Ray3D<T>,
    tolerance: T,
) -> IntersectionResult<T> {
    IntersectionResult::from_option_point(
        triangle3d_ray3d_intersection_raw(triangle, ray, tolerance),
        false,
        tolerance,
    )
}

fn plane3d_line_segment3d_intersection_raw<T: Scalar>(
    plane: &Plane3D<T>,
    segment: &LineSegment3D<T>,
    tolerance: T,
) -> Option<Point3D<T>> {
    let start = segment.start_point();
    let end = segment.end_point();
    let direction = Vector3D::from_points(start, end);

    if direction.is_zero() {
        return if plane.distance_to_point(start).abs() <= tolerance {
            Some(start)
        } else {
            None
        };
    }

    let normal = plane.normal().as_vector();
    let denom = direction.dot(normal);

    if denom.abs() <= tolerance {
        return if plane.distance_to_point(start).abs() <= tolerance {
            Some(start)
        } else {
            None
        };
    }

    let to_plane = Vector3D::from_points(start, plane.origin());
    let t = to_plane.dot(normal) / denom;

    if t >= T::ZERO && t <= T::ONE {
        Some(Point3D::new(
            start.x() + t * direction.x(),
            start.y() + t * direction.y(),
            start.z() + t * direction.z(),
        ))
    } else {
        None
    }
}

pub fn plane3d_line_segment3d_intersection<T: Scalar>(
    plane: &Plane3D<T>,
    segment: &LineSegment3D<T>,
    tolerance: T,
) -> IntersectionResult<T> {
    IntersectionResult::from_option_point(
        plane3d_line_segment3d_intersection_raw(plane, segment, tolerance),
        false,
        tolerance,
    )
}

fn plane3d_ray3d_intersection_raw<T: Scalar>(
    plane: &Plane3D<T>,
    ray: &Ray3D<T>,
    tolerance: T,
) -> Option<Point3D<T>> {
    let origin = ray.origin();
    let direction = ray.direction().as_vector();
    let normal = plane.normal().as_vector();
    let denom = direction.dot(normal);

    if denom.abs() <= tolerance {
        return if plane.distance_to_point(origin).abs() <= tolerance {
            Some(origin)
        } else {
            None
        };
    }

    let to_plane = Vector3D::from_points(origin, plane.origin());
    let t = to_plane.dot(normal) / denom;

    if t >= T::ZERO {
        Some(Point3D::new(
            origin.x() + t * direction.x(),
            origin.y() + t * direction.y(),
            origin.z() + t * direction.z(),
        ))
    } else {
        None
    }
}

pub fn plane3d_ray3d_intersection<T: Scalar>(
    plane: &Plane3D<T>,
    ray: &Ray3D<T>,
    tolerance: T,
) -> IntersectionResult<T> {
    IntersectionResult::from_option_point(
        plane3d_ray3d_intersection_raw(plane, ray, tolerance),
        false,
        tolerance,
    )
}
