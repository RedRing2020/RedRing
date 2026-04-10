use super::planar_and_mesh_family::{
    plane3d_line_segment3d_intersection, plane3d_ray3d_intersection,
};
use super::shared::{point_intersection_if, spherical_surface_intersection_parameters};
use crate::{
    InfiniteLine3D, IntersectionResult, LineSegment3D, Plane3D, Point3D, Ray3D, SphericalSurface3D,
    Vector3D,
};
use geo_contracts::{InfiniteLine3DProperties, Scalar};

fn ray3d_point3d_intersection_raw<T: Scalar>(
    ray: &Ray3D<T>,
    point: &Point3D<T>,
    tolerance: T,
) -> Option<Point3D<T>> {
    point_intersection_if(point, ray.contains_point(point, tolerance))
}

pub fn ray3d_point3d_intersection<T: Scalar>(
    ray: &Ray3D<T>,
    point: &Point3D<T>,
    tolerance: T,
) -> IntersectionResult<T> {
    IntersectionResult::from_option_point(
        ray3d_point3d_intersection_raw(ray, point, tolerance),
        false,
        tolerance,
    )
}

pub fn ray3d_spherical_surface3d_intersections<T: Scalar>(
    ray: &Ray3D<T>,
    sphere: &SphericalSurface3D<T>,
    tolerance: T,
) -> IntersectionResult<T> {
    let mut intersections = Vec::new();
    let start = ray.origin();
    let direction = ray.direction_vector();

    let Some((t1, t2)) =
        spherical_surface_intersection_parameters(&start, &direction, sphere, tolerance)
    else {
        return IntersectionResult::from_option_points(intersections, false, tolerance);
    };

    if t1 >= T::ZERO {
        intersections.push(Point3D::new(
            start.x() + t1 * direction.x(),
            start.y() + t1 * direction.y(),
            start.z() + t1 * direction.z(),
        ));
    }

    if t2 >= T::ZERO && (t2 - t1).abs() > tolerance {
        intersections.push(Point3D::new(
            start.x() + t2 * direction.x(),
            start.y() + t2 * direction.y(),
            start.z() + t2 * direction.z(),
        ));
    }

    IntersectionResult::from_option_points(intersections, false, tolerance)
}

fn ray3d_ray3d_intersection_raw<T: Scalar>(
    ray_a: &Ray3D<T>,
    ray_b: &Ray3D<T>,
    tolerance: T,
) -> Option<Point3D<T>> {
    let origin_a = ray_a.origin();
    let origin_b = ray_b.origin();
    let direction_a = ray_a.direction_vector();
    let direction_b = ray_b.direction_vector();
    let origin_offset = Vector3D::from_points(&origin_b, &origin_a);

    let a = direction_a.dot(&direction_a);
    let b = direction_a.dot(&direction_b);
    let c = direction_b.dot(&direction_b);
    let d = direction_a.dot(&origin_offset);
    let e = direction_b.dot(&origin_offset);

    let denominator = a * c - b * b;
    if denominator.abs() <= tolerance {
        return None;
    }

    let s = (b * e - c * d) / denominator;
    let t = (a * e - b * d) / denominator;
    if s < T::ZERO || t < T::ZERO {
        return None;
    }

    let point_a = ray_a.point_at_parameter(s);
    let point_b = ray_b.point_at_parameter(t);
    if point_a.distance_to(&point_b) <= tolerance {
        Some(point_a)
    } else {
        None
    }
}

pub fn ray3d_ray3d_intersection<T: Scalar>(
    ray_a: &Ray3D<T>,
    ray_b: &Ray3D<T>,
    tolerance: T,
) -> IntersectionResult<T> {
    IntersectionResult::from_option_point(
        ray3d_ray3d_intersection_raw(ray_a, ray_b, tolerance),
        false,
        tolerance,
    )
}

fn ray3d_line_segment3d_intersection_raw<T: Scalar>(
    ray: &Ray3D<T>,
    segment: &LineSegment3D<T>,
    tolerance: T,
) -> Option<Point3D<T>> {
    let ray_line = InfiniteLine3D::new(ray.origin(), ray.direction_vector())?;
    let segment_line = segment.line();

    if ray_line.is_parallel_to(segment_line) || !ray_line.is_coplanar_with(segment_line) {
        return None;
    }

    let point = ray_line.intersection_with_line(segment_line)?;
    if ray.contains_point(&point, tolerance) && segment.contains_point(&point, tolerance) {
        Some(point)
    } else {
        None
    }
}

pub fn ray3d_line_segment3d_intersection<T: Scalar>(
    ray: &Ray3D<T>,
    segment: &LineSegment3D<T>,
    tolerance: T,
) -> IntersectionResult<T> {
    IntersectionResult::from_option_point(
        ray3d_line_segment3d_intersection_raw(ray, segment, tolerance),
        false,
        tolerance,
    )
}

fn ray3d_infinite_line3d_intersection_raw<T: Scalar>(
    ray: &Ray3D<T>,
    line: &InfiniteLine3D<T>,
    tolerance: T,
) -> Option<Point3D<T>> {
    let ray_line = InfiniteLine3D::new(ray.origin(), ray.direction_vector())?;

    if ray_line.is_parallel_to(line) || !ray_line.is_coplanar_with(line) {
        return None;
    }

    let point = ray_line.intersection_with_line(line)?;
    if ray.contains_point(&point, tolerance) {
        Some(point)
    } else {
        None
    }
}

pub fn ray3d_infinite_line3d_intersection<T: Scalar>(
    ray: &Ray3D<T>,
    line: &InfiniteLine3D<T>,
    tolerance: T,
) -> IntersectionResult<T> {
    IntersectionResult::from_option_point(
        ray3d_infinite_line3d_intersection_raw(ray, line, tolerance),
        false,
        tolerance,
    )
}

pub fn ray3d_plane3d_intersection<T: Scalar>(
    ray: &Ray3D<T>,
    plane: &Plane3D<T>,
    tolerance: T,
) -> IntersectionResult<T> {
    plane3d_ray3d_intersection(plane, ray, tolerance)
}

fn line_segment3d_point3d_intersection_raw<T: Scalar>(
    segment: &LineSegment3D<T>,
    point: &Point3D<T>,
    tolerance: T,
) -> Option<Point3D<T>> {
    point_intersection_if(point, segment.contains_point(point, tolerance))
}

pub fn line_segment3d_point3d_intersection<T: Scalar>(
    segment: &LineSegment3D<T>,
    point: &Point3D<T>,
    tolerance: T,
) -> IntersectionResult<T> {
    IntersectionResult::from_option_point(
        line_segment3d_point3d_intersection_raw(segment, point, tolerance),
        false,
        tolerance,
    )
}

pub fn line_segment3d_spherical_surface3d_intersections<T: Scalar>(
    segment: &LineSegment3D<T>,
    sphere: &SphericalSurface3D<T>,
    tolerance: T,
) -> IntersectionResult<T> {
    let mut intersections = Vec::new();
    let start = segment.start();
    let end = segment.end();
    let direction = Vector3D::from_points(&start, &end);

    let Some((t1, t2)) =
        spherical_surface_intersection_parameters(&start, &direction, sphere, tolerance)
    else {
        return IntersectionResult::from_option_points(intersections, false, tolerance);
    };

    if t1 >= T::ZERO && t1 <= T::ONE {
        intersections.push(Point3D::new(
            start.x() + t1 * direction.x(),
            start.y() + t1 * direction.y(),
            start.z() + t1 * direction.z(),
        ));
    }

    if t2 >= T::ZERO && t2 <= T::ONE && (t2 - t1).abs() > tolerance {
        intersections.push(Point3D::new(
            start.x() + t2 * direction.x(),
            start.y() + t2 * direction.y(),
            start.z() + t2 * direction.z(),
        ));
    }

    IntersectionResult::from_option_points(intersections, false, tolerance)
}

fn line_segment3d_line_segment3d_intersection_raw<T: Scalar>(
    seg_a: &LineSegment3D<T>,
    seg_b: &LineSegment3D<T>,
    tolerance: T,
) -> Option<Point3D<T>> {
    let p1 = seg_a.start();
    let p2 = seg_a.end();
    let p3 = seg_b.start();
    let p4 = seg_b.end();

    let d1 = Vector3D::from_points(&p1, &p2);
    let d2 = Vector3D::from_points(&p3, &p4);
    let r = Vector3D::from_points(&p3, &p1);

    let a = d1.dot(&d1);
    let b = d1.dot(&d2);
    let c = d2.dot(&d2);
    let d = d1.dot(&r);
    let e = d2.dot(&r);

    let denom = a * c - b * b;
    if denom.abs() <= tolerance {
        return None;
    }

    let s = (b * e - c * d) / denom;
    let t = (a * e - b * d) / denom;

    if s >= T::ZERO && s <= T::ONE && t >= T::ZERO && t <= T::ONE {
        let point1 = Point3D::new(
            p1.x() + s * d1.x(),
            p1.y() + s * d1.y(),
            p1.z() + s * d1.z(),
        );
        let point2 = Point3D::new(
            p3.x() + t * d2.x(),
            p3.y() + t * d2.y(),
            p3.z() + t * d2.z(),
        );

        if point1.distance_to(&point2) <= tolerance {
            Some(point1)
        } else {
            None
        }
    } else {
        None
    }
}

pub fn line_segment3d_line_segment3d_intersection<T: Scalar>(
    seg_a: &LineSegment3D<T>,
    seg_b: &LineSegment3D<T>,
    tolerance: T,
) -> IntersectionResult<T> {
    IntersectionResult::from_option_point(
        line_segment3d_line_segment3d_intersection_raw(seg_a, seg_b, tolerance),
        false,
        tolerance,
    )
}

pub fn line_segment3d_ray3d_intersection<T: Scalar>(
    segment: &LineSegment3D<T>,
    ray: &Ray3D<T>,
    tolerance: T,
) -> IntersectionResult<T> {
    ray3d_line_segment3d_intersection(ray, segment, tolerance)
}

pub fn line_segment3d_infinite_line3d_intersection<T: Scalar>(
    segment: &LineSegment3D<T>,
    line: &InfiniteLine3D<T>,
    tolerance: T,
) -> IntersectionResult<T> {
    infinite_line3d_line_segment3d_intersection(line, segment, tolerance)
}

pub fn line_segment3d_plane3d_intersection<T: Scalar>(
    segment: &LineSegment3D<T>,
    plane: &Plane3D<T>,
    tolerance: T,
) -> IntersectionResult<T> {
    plane3d_line_segment3d_intersection(plane, segment, tolerance)
}

fn infinite_line3d_point3d_intersection_raw<T: Scalar>(
    line: &InfiniteLine3D<T>,
    point: &Point3D<T>,
    tolerance: T,
) -> Option<Point3D<T>> {
    point_intersection_if(point, line.contains_point(point, tolerance))
}

pub fn infinite_line3d_point3d_intersection<T: Scalar>(
    line: &InfiniteLine3D<T>,
    point: &Point3D<T>,
    tolerance: T,
) -> IntersectionResult<T> {
    IntersectionResult::from_option_point(
        infinite_line3d_point3d_intersection_raw(line, point, tolerance),
        false,
        tolerance,
    )
}

pub fn infinite_line3d_spherical_surface3d_intersections<T: Scalar>(
    line: &InfiniteLine3D<T>,
    sphere: &SphericalSurface3D<T>,
    tolerance: T,
) -> IntersectionResult<T> {
    let mut intersections = Vec::new();
    let point = line.point();
    let direction = line.direction();
    let start = Point3D::new(point.0, point.1, point.2);
    let direction = Vector3D::new(direction.0, direction.1, direction.2);

    let Some((t1, t2)) =
        spherical_surface_intersection_parameters(&start, &direction, sphere, tolerance)
    else {
        return IntersectionResult::from_option_points(intersections, false, tolerance);
    };

    intersections.push(Point3D::new(
        start.x() + t1 * direction.x(),
        start.y() + t1 * direction.y(),
        start.z() + t1 * direction.z(),
    ));

    if (t2 - t1).abs() > tolerance {
        intersections.push(Point3D::new(
            start.x() + t2 * direction.x(),
            start.y() + t2 * direction.y(),
            start.z() + t2 * direction.z(),
        ));
    }

    IntersectionResult::from_option_points(intersections, false, tolerance)
}

fn infinite_line3d_infinite_line3d_intersection_raw<T: Scalar>(
    line_a: &InfiniteLine3D<T>,
    line_b: &InfiniteLine3D<T>,
    tolerance: T,
) -> Option<Point3D<T>> {
    let point = line_a.intersection_with_line(line_b)?;
    if line_b.distance_to_point(&point) <= tolerance {
        Some(point)
    } else {
        None
    }
}

pub fn infinite_line3d_infinite_line3d_intersection<T: Scalar>(
    line_a: &InfiniteLine3D<T>,
    line_b: &InfiniteLine3D<T>,
    tolerance: T,
) -> IntersectionResult<T> {
    IntersectionResult::from_option_point(
        infinite_line3d_infinite_line3d_intersection_raw(line_a, line_b, tolerance),
        false,
        tolerance,
    )
}

fn infinite_line3d_line_segment3d_intersection_raw<T: Scalar>(
    line: &InfiniteLine3D<T>,
    segment: &LineSegment3D<T>,
    tolerance: T,
) -> Option<Point3D<T>> {
    let segment_line = segment.line();
    let point = line.intersection_with_line(segment_line)?;
    if segment.contains_point(&point, tolerance) {
        Some(point)
    } else {
        None
    }
}

pub fn infinite_line3d_line_segment3d_intersection<T: Scalar>(
    line: &InfiniteLine3D<T>,
    segment: &LineSegment3D<T>,
    tolerance: T,
) -> IntersectionResult<T> {
    IntersectionResult::from_option_point(
        infinite_line3d_line_segment3d_intersection_raw(line, segment, tolerance),
        false,
        tolerance,
    )
}

fn infinite_line3d_ray3d_intersection_raw<T: Scalar>(
    line: &InfiniteLine3D<T>,
    ray: &Ray3D<T>,
    tolerance: T,
) -> Option<Point3D<T>> {
    let ray_line = InfiniteLine3D::new(ray.origin(), ray.direction_vector())?;
    let point = line.intersection_with_line(&ray_line)?;
    if ray.contains_point(&point, tolerance) {
        Some(point)
    } else {
        None
    }
}

pub fn infinite_line3d_ray3d_intersection<T: Scalar>(
    line: &InfiniteLine3D<T>,
    ray: &Ray3D<T>,
    tolerance: T,
) -> IntersectionResult<T> {
    IntersectionResult::from_option_point(
        infinite_line3d_ray3d_intersection_raw(line, ray, tolerance),
        false,
        tolerance,
    )
}
