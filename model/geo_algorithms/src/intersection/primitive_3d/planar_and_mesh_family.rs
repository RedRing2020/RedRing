use super::shared::point_intersection_if;
use crate::{
    InfiniteLine3D, IntersectionResult, LineSegment3D, Plane3D, Point3D, Ray3D, Triangle3D,
    TriangleMesh3D, Vector3D,
};
use geo_contracts::{InfiniteLine3DProperties, Scalar, Triangle3DBoundaryAccess};

fn triangle3d_point3d_intersection_raw<T: Scalar>(
    triangle: &Triangle3D<T>,
    point: &Point3D<T>,
    tolerance: T,
) -> Option<Point3D<T>> {
    point_intersection_if(
        point,
        crate::distance::triangle3d_point3d_distance(triangle, point) <= tolerance,
    )
}

pub fn triangle3d_point3d_intersection<T: Scalar>(
    triangle: &Triangle3D<T>,
    point: &Point3D<T>,
    tolerance: T,
) -> IntersectionResult<T> {
    IntersectionResult::from_option_point(
        triangle3d_point3d_intersection_raw(triangle, point, tolerance),
        false,
        tolerance,
    )
}

fn triangle3d_line_segment3d_intersection_raw<T: Scalar>(
    triangle: &Triangle3D<T>,
    segment: &LineSegment3D<T>,
    tolerance: T,
) -> Option<Point3D<T>> {
    let start = segment.start();
    let end = segment.end();
    let dir = end - start;
    let length = dir.length();
    if length <= tolerance {
        return None;
    }
    let ray = Ray3D::new(start, dir)?;
    let point = triangle3d_ray3d_intersection_raw(triangle, &ray, tolerance)?;

    let t = ray.parameter_for_point(&point);
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

pub fn line_segment3d_triangle3d_intersection<T: Scalar>(
    segment: &LineSegment3D<T>,
    triangle: &Triangle3D<T>,
    tolerance: T,
) -> IntersectionResult<T> {
    triangle3d_line_segment3d_intersection(triangle, segment, tolerance)
}

fn triangle3d_ray3d_intersection_raw<T: Scalar>(
    triangle: &Triangle3D<T>,
    ray: &Ray3D<T>,
    tolerance: T,
) -> Option<Point3D<T>> {
    let va = triangle.vertex_a();
    let vb = triangle.vertex_b();
    let vc = triangle.vertex_c();

    let v0 = Point3D::new(va.0, va.1, va.2);
    let v1 = Point3D::new(vb.0, vb.1, vb.2);
    let v2 = Point3D::new(vc.0, vc.1, vc.2);

    let edge1 = Vector3D::from_points(&v0, &v1);
    let edge2 = Vector3D::from_points(&v0, &v2);

    let ray_dir = ray.direction_vector();
    let h = ray_dir.cross(&edge2);
    let a = edge1.dot(&h);

    if a.abs() <= tolerance {
        return None;
    }

    let f = T::ONE / a;
    let s = Vector3D::from_points(&v0, &ray.origin());
    let u = f * s.dot(&h);

    if u < T::ZERO || u > T::ONE {
        return None;
    }

    let q = s.cross(&edge1);
    let v = f * ray_dir.dot(&q);

    if v < T::ZERO || u + v > T::ONE {
        return None;
    }

    let t = f * edge2.dot(&q);
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

pub fn ray3d_triangle3d_intersection<T: Scalar>(
    ray: &Ray3D<T>,
    triangle: &Triangle3D<T>,
    tolerance: T,
) -> IntersectionResult<T> {
    triangle3d_ray3d_intersection(triangle, ray, tolerance)
}

fn triangle_mesh3d_point3d_intersection_raw<T: Scalar>(
    mesh: &TriangleMesh3D<T>,
    point: &Point3D<T>,
    tolerance: T,
) -> Option<Point3D<T>> {
    point_intersection_if(
        point,
        crate::distance::triangle_mesh3d_point3d_distance(mesh, point) <= tolerance,
    )
}

pub fn triangle_mesh3d_point3d_intersection<T: Scalar>(
    mesh: &TriangleMesh3D<T>,
    point: &Point3D<T>,
    tolerance: T,
) -> IntersectionResult<T> {
    IntersectionResult::from_option_point(
        triangle_mesh3d_point3d_intersection_raw(mesh, point, tolerance),
        false,
        tolerance,
    )
}

fn plane3d_point3d_intersection_raw<T: Scalar>(
    plane: &Plane3D<T>,
    point: &Point3D<T>,
    tolerance: T,
) -> Option<Point3D<T>> {
    point_intersection_if(point, plane.contains_point(*point, tolerance))
}

pub fn plane3d_point3d_intersection<T: Scalar>(
    plane: &Plane3D<T>,
    point: &Point3D<T>,
    tolerance: T,
) -> IntersectionResult<T> {
    IntersectionResult::from_option_point(
        plane3d_point3d_intersection_raw(plane, point, tolerance),
        false,
        tolerance,
    )
}

fn plane3d_line_segment3d_intersection_raw<T: Scalar>(
    plane: &Plane3D<T>,
    segment: &LineSegment3D<T>,
    tolerance: T,
) -> Option<Point3D<T>> {
    let start = segment.start();
    let end = segment.end();
    let direction = Vector3D::from_points(&start, &end);

    if direction.is_zero() {
        return if plane.contains_point(start, tolerance) {
            Some(start)
        } else {
            None
        };
    }

    let normal = plane.normal().as_vector();
    let denom = direction.dot(&normal);

    if denom.abs() <= tolerance {
        return if plane.contains_point(start, tolerance) {
            Some(start)
        } else {
            None
        };
    }

    let to_plane = Vector3D::from_points(&start, &plane.origin());
    let t = to_plane.dot(&normal) / denom;

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
    let direction = ray.direction_vector();
    let normal = plane.normal().as_vector();
    let denom = direction.dot(&normal);

    if denom.abs() <= tolerance {
        return if plane.contains_point(origin, tolerance) {
            Some(origin)
        } else {
            None
        };
    }

    let to_plane = Vector3D::from_points(&origin, &plane.origin());
    let t = to_plane.dot(&normal) / denom;

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

fn plane3d_infinite_line3d_intersection_raw<T: Scalar>(
    plane: &Plane3D<T>,
    line: &InfiniteLine3D<T>,
    tolerance: T,
) -> Option<Point3D<T>> {
    let lp = line.point();
    let ld = line.direction();
    let line_point = Point3D::new(lp.0, lp.1, lp.2);
    let line_dir_vec = Vector3D::new(ld.0, ld.1, ld.2);
    let normal = plane.normal().as_vector();
    let denom = line_dir_vec.dot(&normal);

    if denom.abs() <= tolerance {
        return if plane.distance_to_point(line_point).abs() <= tolerance {
            Some(line_point)
        } else {
            None
        };
    }

    let to_plane = Vector3D::from_points(&line_point, &plane.origin());
    let t = to_plane.dot(&normal) / denom;
    Some(Point3D::new(
        line_point.x() + t * line_dir_vec.x(),
        line_point.y() + t * line_dir_vec.y(),
        line_point.z() + t * line_dir_vec.z(),
    ))
}

pub fn plane3d_infinite_line3d_intersection<T: Scalar>(
    plane: &Plane3D<T>,
    line: &InfiniteLine3D<T>,
    tolerance: T,
) -> IntersectionResult<T> {
    IntersectionResult::from_option_point(
        plane3d_infinite_line3d_intersection_raw(plane, line, tolerance),
        false,
        tolerance,
    )
}
