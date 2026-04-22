use super::shared::{
    conical_solid3d_contains_point_with_tolerance, conical_surface3d_filter_params,
    conical_surface3d_intersect_params, point_intersection_if, triangle3d_vertex_points,
};
use crate::{
    Circle3D, ConicalSolid3D, ConicalSurface3D, CylindricalSurface3D, InfiniteLine3D,
    IntersectionResult, LineSegment3D, Plane3D, Point3D, Ray3D, Triangle3D, Vector3D,
};
use geo_contracts::{
    Circle3DProperties, ConicalSolid3DProperties, CylindricalSurface3DProperties,
    InfiniteLine3DProperties, Scalar,
};

fn cylindrical_surface3d_point3d_intersection_raw<T: Scalar>(
    cyl: &CylindricalSurface3D<T>,
    point: &Point3D<T>,
    tolerance: T,
) -> Option<Point3D<T>> {
    point_intersection_if(
        point,
        crate::distance::cylindrical_surface3d_point3d_distance(cyl, point) <= tolerance,
    )
}

pub fn cylindrical_surface3d_point3d_intersection<T: Scalar>(
    cyl: &CylindricalSurface3D<T>,
    point: &Point3D<T>,
    tolerance: T,
) -> IntersectionResult<T> {
    IntersectionResult::from_option_point(
        cylindrical_surface3d_point3d_intersection_raw(cyl, point, tolerance),
        false,
        tolerance,
    )
}

fn cylindrical_surface3d_circle3d_intersection_raw<T: Scalar>(
    cyl: &CylindricalSurface3D<T>,
    circle: &Circle3D<T>,
    tolerance: T,
) -> Option<Point3D<T>> {
    let (cx, cy, cz) = Circle3DProperties::center(circle);
    let center = Point3D::new(cx, cy, cz);
    let dist = crate::distance::cylindrical_surface3d_point3d_distance(cyl, &center);
    if dist <= Circle3DProperties::radius(circle) + tolerance {
        Some(center)
    } else {
        None
    }
}

pub fn cylindrical_surface3d_circle3d_intersection<T: Scalar>(
    cyl: &CylindricalSurface3D<T>,
    circle: &Circle3D<T>,
    tolerance: T,
) -> IntersectionResult<T> {
    IntersectionResult::from_option_point(
        cylindrical_surface3d_circle3d_intersection_raw(cyl, circle, tolerance),
        false,
        tolerance,
    )
}

fn cylindrical_surface3d_line_segment3d_intersection_raw<T: Scalar>(
    cyl: &CylindricalSurface3D<T>,
    segment: &LineSegment3D<T>,
    tolerance: T,
) -> Option<Point3D<T>> {
    let s = segment.start();
    let e = segment.end();
    if crate::distance::cylindrical_surface3d_point3d_distance(cyl, &s) <= tolerance {
        Some(s)
    } else if crate::distance::cylindrical_surface3d_point3d_distance(cyl, &e) <= tolerance {
        Some(e)
    } else {
        None
    }
}

pub fn cylindrical_surface3d_line_segment3d_intersection<T: Scalar>(
    cyl: &CylindricalSurface3D<T>,
    segment: &LineSegment3D<T>,
    tolerance: T,
) -> IntersectionResult<T> {
    IntersectionResult::from_option_point(
        cylindrical_surface3d_line_segment3d_intersection_raw(cyl, segment, tolerance),
        false,
        tolerance,
    )
}

fn cylindrical_surface3d_triangle3d_intersection_raw<T: Scalar>(
    cyl: &CylindricalSurface3D<T>,
    triangle: &Triangle3D<T>,
    tolerance: T,
) -> Option<Point3D<T>> {
    triangle3d_vertex_points(triangle)
        .into_iter()
        .find(|point| {
            crate::distance::cylindrical_surface3d_point3d_distance(cyl, point) <= tolerance
        })
}

pub fn cylindrical_surface3d_triangle3d_intersection<T: Scalar>(
    cyl: &CylindricalSurface3D<T>,
    triangle: &Triangle3D<T>,
    tolerance: T,
) -> IntersectionResult<T> {
    IntersectionResult::from_option_point(
        cylindrical_surface3d_triangle3d_intersection_raw(cyl, triangle, tolerance),
        false,
        tolerance,
    )
}

fn cylindrical_surface3d_plane3d_intersection_raw<T: Scalar>(
    cyl: &CylindricalSurface3D<T>,
    plane: &Plane3D<T>,
    tolerance: T,
) -> Option<Point3D<T>> {
    let (cx, cy, cz) = <CylindricalSurface3D<T> as CylindricalSurface3DProperties<T>>::center(cyl);
    let center = Point3D::new(cx, cy, cz);
    let radius = <CylindricalSurface3D<T> as CylindricalSurface3DProperties<T>>::radius(cyl);
    if plane.distance_to_point(center).abs() <= radius + tolerance {
        Some(Point3D::new(cx, cy, cz))
    } else {
        None
    }
}

pub fn cylindrical_surface3d_plane3d_intersection<T: Scalar>(
    cyl: &CylindricalSurface3D<T>,
    plane: &Plane3D<T>,
    tolerance: T,
) -> IntersectionResult<T> {
    IntersectionResult::from_option_point(
        cylindrical_surface3d_plane3d_intersection_raw(cyl, plane, tolerance),
        false,
        tolerance,
    )
}

fn cylindrical_surface3d_cylindrical_surface3d_intersection_raw<T: Scalar>(
    lhs: &CylindricalSurface3D<T>,
    rhs: &CylindricalSurface3D<T>,
    tolerance: T,
) -> Option<Point3D<T>> {
    let lhs_center_tuple =
        <CylindricalSurface3D<T> as CylindricalSurface3DProperties<T>>::center(lhs);
    let rhs_center_tuple =
        <CylindricalSurface3D<T> as CylindricalSurface3DProperties<T>>::center(rhs);
    let lhs_center = Point3D::new(lhs_center_tuple.0, lhs_center_tuple.1, lhs_center_tuple.2);
    let rhs_center = Point3D::new(rhs_center_tuple.0, rhs_center_tuple.1, rhs_center_tuple.2);

    let lhs_dist = crate::distance::cylindrical_surface3d_point3d_distance(lhs, &rhs_center);
    let rhs_dist = crate::distance::cylindrical_surface3d_point3d_distance(rhs, &lhs_center);

    let lhs_axis_tuple = <CylindricalSurface3D<T> as CylindricalSurface3DProperties<T>>::axis(lhs);
    let rhs_axis_tuple = <CylindricalSurface3D<T> as CylindricalSurface3DProperties<T>>::axis(rhs);
    let lhs_axis = Vector3D::new(lhs_axis_tuple.0, lhs_axis_tuple.1, lhs_axis_tuple.2);
    let rhs_axis = Vector3D::new(rhs_axis_tuple.0, rhs_axis_tuple.1, rhs_axis_tuple.2);
    let center_delta = Vector3D::from_points(&lhs_center, &rhs_center);

    let angle_tolerance = if tolerance > T::from_f64(1.0e-6) {
        tolerance
    } else {
        T::from_f64(1.0e-6)
    };
    let axes_parallel = (lhs_axis.dot(&rhs_axis).abs() - T::ONE).abs() <= angle_tolerance;
    let axis_distance = center_delta.cross(&lhs_axis).magnitude();
    let radii_match = (<CylindricalSurface3D<T> as CylindricalSurface3DProperties<T>>::radius(lhs)
        - <CylindricalSurface3D<T> as CylindricalSurface3DProperties<T>>::radius(rhs))
    .abs()
        <= tolerance;
    let coincident = axes_parallel && axis_distance <= tolerance && radii_match;

    if lhs_dist <= tolerance || rhs_dist <= tolerance || coincident {
        Some(Point3D::new(
            (lhs_center.x() + rhs_center.x()) / (T::ONE + T::ONE),
            (lhs_center.y() + rhs_center.y()) / (T::ONE + T::ONE),
            (lhs_center.z() + rhs_center.z()) / (T::ONE + T::ONE),
        ))
    } else {
        None
    }
}

pub fn cylindrical_surface3d_cylindrical_surface3d_intersection<T: Scalar>(
    lhs: &CylindricalSurface3D<T>,
    rhs: &CylindricalSurface3D<T>,
    tolerance: T,
) -> IntersectionResult<T> {
    IntersectionResult::from_option_point(
        cylindrical_surface3d_cylindrical_surface3d_intersection_raw(lhs, rhs, tolerance),
        false,
        tolerance,
    )
}

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

fn conical_solid3d_line3d_intersection_raw<T: Scalar>(
    cone: &ConicalSolid3D<T>,
    line: &InfiniteLine3D<T>,
    tolerance: T,
) -> Option<Point3D<T>> {
    let (px, py, pz) = InfiniteLine3DProperties::point(line);
    let point_on_line = Point3D::new(px, py, pz);
    point_intersection_if(
        &point_on_line,
        conical_solid3d_contains_point_with_tolerance(cone, &point_on_line, tolerance),
    )
}

pub fn conical_solid3d_line3d_intersection<T: Scalar>(
    cone: &ConicalSolid3D<T>,
    line: &InfiniteLine3D<T>,
    tolerance: T,
) -> IntersectionResult<T> {
    IntersectionResult::from_option_point(
        conical_solid3d_line3d_intersection_raw(cone, line, tolerance),
        false,
        tolerance,
    )
}

fn conical_solid3d_ray3d_intersection_raw<T: Scalar>(
    cone: &ConicalSolid3D<T>,
    ray: &Ray3D<T>,
    tolerance: T,
) -> Option<Point3D<T>> {
    let origin = ray.origin();
    point_intersection_if(
        &origin,
        conical_solid3d_contains_point_with_tolerance(cone, &origin, tolerance),
    )
}

pub fn conical_solid3d_ray3d_intersection<T: Scalar>(
    cone: &ConicalSolid3D<T>,
    ray: &Ray3D<T>,
    tolerance: T,
) -> IntersectionResult<T> {
    IntersectionResult::from_option_point(
        conical_solid3d_ray3d_intersection_raw(cone, ray, tolerance),
        false,
        tolerance,
    )
}

pub fn ray3d_conical_solid3d_intersection<T: Scalar>(
    ray: &Ray3D<T>,
    cone: &ConicalSolid3D<T>,
    tolerance: T,
) -> IntersectionResult<T> {
    conical_solid3d_ray3d_intersection(cone, ray, tolerance)
}

fn conical_solid3d_line_segment3d_intersection_raw<T: Scalar>(
    cone: &ConicalSolid3D<T>,
    segment: &LineSegment3D<T>,
    tolerance: T,
) -> Option<Point3D<T>> {
    let start = segment.start();
    point_intersection_if(
        &start,
        conical_solid3d_contains_point_with_tolerance(cone, &start, tolerance),
    )
}

pub fn conical_solid3d_line_segment3d_intersection<T: Scalar>(
    cone: &ConicalSolid3D<T>,
    segment: &LineSegment3D<T>,
    tolerance: T,
) -> IntersectionResult<T> {
    IntersectionResult::from_option_point(
        conical_solid3d_line_segment3d_intersection_raw(cone, segment, tolerance),
        false,
        tolerance,
    )
}

fn conical_solid3d_plane3d_intersection_raw<T: Scalar>(
    cone: &ConicalSolid3D<T>,
    plane: &Plane3D<T>,
    _tolerance: T,
) -> Option<Point3D<T>> {
    let (ax, ay, az) = ConicalSolid3DProperties::apex(cone);
    let (bx, by, bz) = ConicalSolid3DProperties::base_center(cone);
    let apex = Point3D::new(ax, ay, az);
    let base = Point3D::new(bx, by, bz);
    let radius = ConicalSolid3DProperties::radius(cone);
    let dist_apex = plane.distance_to_point(apex);
    let dist_base = plane.distance_to_point(base);
    if dist_apex * dist_base <= T::ZERO || dist_base.abs() <= radius {
        Some(base)
    } else {
        None
    }
}

pub fn conical_solid3d_plane3d_intersection<T: Scalar>(
    cone: &ConicalSolid3D<T>,
    plane: &Plane3D<T>,
    tolerance: T,
) -> IntersectionResult<T> {
    IntersectionResult::from_option_point(
        conical_solid3d_plane3d_intersection_raw(cone, plane, tolerance),
        false,
        tolerance,
    )
}

fn conical_surface3d_point3d_intersection_raw<T: Scalar>(
    cone: &ConicalSurface3D<T>,
    point: &Point3D<T>,
    tolerance: T,
) -> Option<Point3D<T>> {
    point_intersection_if(
        point,
        crate::distance::conical_surface3d_point3d_distance(cone, point) <= tolerance,
    )
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
    let (px, py, pz) = InfiniteLine3DProperties::point(line);
    let (lx, ly, lz) = InfiniteLine3DProperties::direction(line);
    let origin = Point3D::new(px, py, pz);
    let direction = Vector3D::new(lx, ly, lz);
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
    let direction = ray.direction_vector();
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

pub fn ray3d_conical_surface3d_intersections<T: Scalar>(
    ray: &Ray3D<T>,
    cone: &ConicalSurface3D<T>,
    tolerance: T,
) -> IntersectionResult<T> {
    conical_surface3d_ray3d_intersections(cone, ray, tolerance)
}

pub fn conical_surface3d_line_segment3d_intersections<T: Scalar>(
    cone: &ConicalSurface3D<T>,
    segment: &LineSegment3D<T>,
    tolerance: T,
) -> IntersectionResult<T> {
    let start = segment.start();
    let end = segment.end();
    let direction = Vector3D::from_points(&start, &end);
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
