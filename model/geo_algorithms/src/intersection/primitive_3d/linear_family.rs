use super::super::common::{
    line_line_intersection_raw, ray_ray_intersection_raw, segment_segment_intersection_raw,
};
use super::shared::spherical_surface_intersection_parameters;
use crate::{
    InfiniteLine3D, IntersectionResult, LineSegment3D, Point3D, Ray3D, SphericalSurface3D, Vector3D,
};
use geo_contracts::{
    InfiniteLine3DConstructor, InfiniteLine3DDistance, LineSegment3DDistance,
    LineSegment3DProperties, LineSegment3DSupportLine, Ray3DDistance, Ray3DProperties, Scalar,
};

pub fn ray3d_spherical_surface3d_intersections<T: Scalar>(
    ray: &Ray3D<T>,
    sphere: &SphericalSurface3D<T>,
    tolerance: T,
) -> IntersectionResult<T> {
    let mut intersections = Vec::new();
    let start = ray.origin();
    let direction = ray.direction().as_vector();

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
    ray_ray_intersection_raw(ray_a, ray_b, tolerance)
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

pub fn line_segment3d_spherical_surface3d_intersections<T: Scalar>(
    segment: &LineSegment3D<T>,
    sphere: &SphericalSurface3D<T>,
    tolerance: T,
) -> IntersectionResult<T> {
    let mut intersections = Vec::new();
    let start = segment.start_point();
    let end = segment.end_point();
    let direction = Vector3D::from_points(start, end);

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
    segment_segment_intersection_raw(seg_a, seg_b, tolerance)
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

fn infinite_line3d_infinite_line3d_intersection_raw<T: Scalar>(
    line_a: &InfiniteLine3D<T>,
    line_b: &InfiniteLine3D<T>,
    tolerance: T,
) -> Option<Point3D<T>> {
    let point = line_line_intersection_raw(line_a, line_b)?;
    if line_b.distance_to_point(point) <= tolerance {
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
    let segment_line = segment.support_line();
    let point = line_line_intersection_raw(line, segment_line)?;
    if segment.distance_to_point(point) <= tolerance {
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
    let ray_line = InfiniteLine3D::new(ray.origin(), ray.direction().as_vector())?;
    let point = line_line_intersection_raw(line, &ray_line)?;
    if ray.distance_to_point(point) <= tolerance {
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
