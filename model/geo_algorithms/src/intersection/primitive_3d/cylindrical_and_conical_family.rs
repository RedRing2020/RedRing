use super::shared::{
    conical_solid3d_contains_point_with_tolerance, conical_surface3d_filter_params,
    conical_surface3d_intersect_params, convex_solid_linear_intersection, point_intersection_if,
};
use crate::{
    ConicalSolid3D, ConicalSurface3D, InfiniteLine3D, IntersectionResult, LineSegment3D, Point3D,
    Ray3D, Vector3D,
};
use analysis::{minimize_unimodal, polynomial_real_roots_in_interval};
use geo_contracts::{
    ConicalSolid3DDistance, ConicalSolid3DProperties, ConicalSurface3DDistance,
    InfiniteLine3DProperties, LineSegment3DProperties, Ray3DProperties, Scalar,
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

/// 円錐（立体）と直線 `origin + direction * t`（`lower <= t <= upper`）の交差
///
/// パラメータの範囲を、円錐を囲む球（中心は軸の中点）と交わる部分に絞る。円錐は凸なので内部にある部分は
/// 1 つの区間になる。円錐面の 2 次式の根と、底面・頂点を通り軸に垂直な平面との交点で範囲を区切り、
/// 区切った小区間の中点が内部にあるかで区間を求める。
fn conical_solid3d_linear_intersection<T: Scalar>(
    cone: &ConicalSolid3D<T>,
    origin: Point3D<T>,
    direction: Vector3D<T>,
    lower: T,
    upper: T,
    tolerance: T,
) -> IntersectionResult<T> {
    let two = T::ONE + T::ONE;
    let base = cone.base_center();
    let axis = cone.axis().as_vector();
    let height = cone.height();
    let radius = cone.radius();

    let bounding_center = base + axis * (height / two);
    let bounding_radius = ((height / two) * (height / two) + radius * radius).sqrt() + tolerance;
    let length_squared = direction.dot(direction);
    let center_parameter =
        Vector3D::from_points(origin, bounding_center).dot(direction) / length_squared;
    let center_distance = (origin + direction * center_parameter).distance_to(bounding_center);
    if center_distance > bounding_radius {
        return IntersectionResult::disjoint(tolerance);
    }
    let half_width = (bounding_radius * bounding_radius - center_distance * center_distance).sqrt()
        / length_squared.sqrt();
    let range_lower = lower.max(center_parameter - half_width);
    let range_upper = upper.min(center_parameter + half_width);
    if range_lower > range_upper {
        return IntersectionResult::disjoint(tolerance);
    }

    // 底面の中心からの軸方向の高さ h(t) = h0 + hd t と、頂点までの残りの高さ s(t) = s0 - hd t。
    // 内部は 0 <= h かつ 0 <= s かつ（軸からの距離）² <= (radius / height)² s²
    let offset = Vector3D::from_points(base, origin);
    let (h0, hd) = (offset.dot(axis), direction.dot(axis));
    let radial_offset = offset - axis * h0;
    let radial_direction = direction - axis * hd;
    let s0 = height - h0;
    let slope_squared = (radius / height) * (radius / height);
    let cone_quadric = [
        radial_offset.dot(radial_offset) - slope_squared * s0 * s0,
        two * (radial_offset.dot(radial_direction) + slope_squared * s0 * hd),
        radial_direction.dot(radial_direction) - slope_squared * hd * hd,
    ];
    let inside = |parameter: T| {
        let quadric = cone_quadric[0] + (cone_quadric[1] + cone_quadric[2] * parameter) * parameter;
        h0 + hd * parameter >= T::ZERO && s0 - hd * parameter >= T::ZERO && quadric <= T::ZERO
    };

    let mut breakpoints = vec![range_lower, range_upper];
    let linear_heights = [[h0, hd], [s0, -hd]];
    for coefficients in
        std::iter::once(&cone_quadric[..]).chain(linear_heights.iter().map(|c| &c[..]))
    {
        breakpoints.extend(polynomial_real_roots_in_interval(
            coefficients,
            range_lower,
            range_upper,
        ));
    }
    breakpoints.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let overlap = breakpoints
        .windows(2)
        .filter(|window| inside((window[0] + window[1]) / two))
        .fold(None, |overlap: Option<(T, T)>, window| match overlap {
            None => Some((window[0], window[1])),
            Some((enter, exit)) => Some((enter.min(window[0]), exit.max(window[1]))),
        });

    let closest = || {
        let distance_at = |parameter: T| cone.distance_to_point(origin + direction * parameter);
        let parameter = minimize_unimodal(distance_at, range_lower, range_upper);
        (parameter, distance_at(parameter))
    };
    convex_solid_linear_intersection(origin, direction, overlap, closest, tolerance)
}

pub fn conical_solid3d_infinite_line3d_intersection<T: Scalar>(
    cone: &ConicalSolid3D<T>,
    line: &InfiniteLine3D<T>,
    tolerance: T,
) -> IntersectionResult<T> {
    conical_solid3d_linear_intersection(
        cone,
        line.point(),
        line.direction().as_vector(),
        -T::INFINITY,
        T::INFINITY,
        tolerance,
    )
}

pub fn conical_solid3d_ray3d_intersection<T: Scalar>(
    cone: &ConicalSolid3D<T>,
    ray: &Ray3D<T>,
    tolerance: T,
) -> IntersectionResult<T> {
    conical_solid3d_linear_intersection(
        cone,
        ray.origin(),
        ray.direction().as_vector(),
        T::ZERO,
        T::INFINITY,
        tolerance,
    )
}

pub fn conical_solid3d_line_segment3d_intersection<T: Scalar>(
    cone: &ConicalSolid3D<T>,
    segment: &LineSegment3D<T>,
    tolerance: T,
) -> IntersectionResult<T> {
    let start = segment.start_point();
    conical_solid3d_linear_intersection(
        cone,
        start,
        Vector3D::from_points(start, segment.end_point()),
        T::ZERO,
        T::ONE,
        tolerance,
    )
}
