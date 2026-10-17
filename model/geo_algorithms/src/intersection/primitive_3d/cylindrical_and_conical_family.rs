use super::shared::{
    conical_solid3d_contains_point_with_tolerance, conical_surface3d_filter_params,
    conical_surface3d_intersect_params, point_intersection_if,
};
use crate::{
    ConicalSolid3D, ConicalSurface3D, InfiniteLine3D, IntersectionResult, LineSegment3D, Point3D,
    Ray3D, Vector3D,
};
use geo_contracts::{
    ConicalSurface3DDistance, InfiniteLine3DProperties, LineSegment3DProperties, Ray3DProperties,
    Scalar,
};

fn conical_solid3d_point3d_intersection_raw<T: Scalar>(
    cone: &ConicalSolid3D<T>,
    point: &Point3D<T>,
    tolerance: T,
) -> Option<Point3D<T>> {
    point_intersection_if(
        point,
        conical_solid3d_contains_point_with_tolerance(cone, point, tolerance),
    )
}

pub fn conical_solid3d_point3d_intersection<T: Scalar>(
    cone: &ConicalSolid3D<T>,
    point: &Point3D<T>,
    tolerance: T,
) -> IntersectionResult<T> {
    IntersectionResult::from_option_point(
        conical_solid3d_point3d_intersection_raw(cone, point, tolerance),
        false,
        tolerance,
    )
}

fn conical_surface3d_point3d_intersection_raw<T: Scalar>(
    cone: &ConicalSurface3D<T>,
    point: &Point3D<T>,
    tolerance: T,
) -> Option<Point3D<T>> {
    point_intersection_if(point, cone.distance_to_point(*point) <= tolerance)
}

pub fn conical_surface3d_point3d_intersection<T: Scalar>(
    cone: &ConicalSurface3D<T>,
    point: &Point3D<T>,
    tolerance: T,
) -> IntersectionResult<T> {
    IntersectionResult::from_option_point(
        conical_surface3d_point3d_intersection_raw(cone, point, tolerance),
        false,
        tolerance,
    )
}

pub fn conical_surface3d_infinite_line3d_intersections<T: Scalar>(
    cone: &ConicalSurface3D<T>,
    line: &InfiniteLine3D<T>,
    tolerance: T,
) -> IntersectionResult<T> {
    let origin = InfiniteLine3DProperties::point(line);
    let direction = InfiniteLine3DProperties::direction(line).as_vector();
    let params = conical_surface3d_intersect_params(cone, &origin, &direction, tolerance);
    let t_inf = T::ONE / tolerance;
    IntersectionResult::from_option_points(
        conical_surface3d_filter_params(
            cone, &origin, &direction, params, -t_inf, t_inf, tolerance,
        ),
        false,
        tolerance,
    )
}

pub fn conical_surface3d_ray3d_intersections<T: Scalar>(
    cone: &ConicalSurface3D<T>,
    ray: &Ray3D<T>,
    tolerance: T,
) -> IntersectionResult<T> {
    let origin = ray.origin();
    let direction = ray.direction().as_vector();
    let params = conical_surface3d_intersect_params(cone, &origin, &direction, tolerance);
    let t_inf = T::ONE / tolerance;
    IntersectionResult::from_option_points(
        conical_surface3d_filter_params(
            cone,
            &origin,
            &direction,
            params,
            T::ZERO,
            t_inf,
            tolerance,
        ),
        false,
        tolerance,
    )
}

pub fn conical_surface3d_line_segment3d_intersections<T: Scalar>(
    cone: &ConicalSurface3D<T>,
    segment: &LineSegment3D<T>,
    tolerance: T,
) -> IntersectionResult<T> {
    let start = segment.start_point();
    let end = segment.end_point();
    let direction = Vector3D::from_points(start, end);
    let params = conical_surface3d_intersect_params(cone, &start, &direction, tolerance);
    IntersectionResult::from_option_points(
        conical_surface3d_filter_params(
            cone,
            &start,
            &direction,
            params,
            T::ZERO,
            T::ONE,
            tolerance,
        ),
        false,
        tolerance,
    )
}
