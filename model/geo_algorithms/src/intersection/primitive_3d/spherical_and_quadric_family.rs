use super::shared::point_intersection_if;
use crate::{
    EllipsoidalSolid3D, EllipsoidalSurface3D, InfiniteLine3D, IntersectionResult, LineSegment3D,
    Plane3D, Point3D, Ray3D, SphericalSolid3D, TorusSolid3D, TorusSurface3D, Vector3D,
};
use geo_contracts::{
    EllipsoidalSolid3DProperties, InfiniteLine3DProperties, Scalar, SphericalSolid3DProperties,
};

fn ellipsoidal_solid3d_point3d_intersection_raw<T: Scalar>(
    ellipsoid: &EllipsoidalSolid3D<T>,
    point: &Point3D<T>,
    tolerance: T,
) -> Option<Point3D<T>> {
    point_intersection_if(
        point,
        crate::distance::ellipsoidal_solid3d_point3d_distance(ellipsoid, point) <= tolerance,
    )
}

pub fn ellipsoidal_solid3d_point3d_intersection<T: Scalar>(
    ellipsoid: &EllipsoidalSolid3D<T>,
    point: &Point3D<T>,
    tolerance: T,
) -> IntersectionResult<T> {
    IntersectionResult::from_option_point(
        ellipsoidal_solid3d_point3d_intersection_raw(ellipsoid, point, tolerance),
        false,
        tolerance,
    )
}

fn ellipsoidal_solid3d_plane3d_intersection_raw<T: Scalar>(
    ellipsoid: &EllipsoidalSolid3D<T>,
    plane: &Plane3D<T>,
    _tolerance: T,
) -> Option<Point3D<T>> {
    let (c0, c1, c2) = EllipsoidalSolid3DProperties::center(ellipsoid);
    let center = Point3D::new(c0, c1, c2);
    let max_radius = EllipsoidalSolid3DProperties::a_radius(ellipsoid)
        .max(EllipsoidalSolid3DProperties::b_radius(ellipsoid))
        .max(EllipsoidalSolid3DProperties::c_radius(ellipsoid));
    if plane.distance_to_point(center).abs() <= max_radius {
        Some(Point3D::new(c0, c1, c2))
    } else {
        None
    }
}

pub fn ellipsoidal_solid3d_plane3d_intersection<T: Scalar>(
    ellipsoid: &EllipsoidalSolid3D<T>,
    plane: &Plane3D<T>,
    tolerance: T,
) -> IntersectionResult<T> {
    IntersectionResult::from_option_point(
        ellipsoidal_solid3d_plane3d_intersection_raw(ellipsoid, plane, tolerance),
        false,
        tolerance,
    )
}

pub fn ellipsoidal_solid3d_infinite_line3d_intersections<T: Scalar>(
    ellipsoid: &EllipsoidalSolid3D<T>,
    line: &InfiniteLine3D<T>,
    tolerance: T,
) -> IntersectionResult<T> {
    let (px, py, pz) = InfiniteLine3DProperties::point(line);
    let point_on_line = Point3D::new(px, py, pz);
    let points = if crate::distance::ellipsoidal_solid3d_point3d_distance(ellipsoid, &point_on_line)
        <= tolerance
    {
        vec![point_on_line]
    } else {
        vec![]
    };
    IntersectionResult::from_option_points(points, false, tolerance)
}

pub fn ellipsoidal_solid3d_ray3d_intersections<T: Scalar>(
    ellipsoid: &EllipsoidalSolid3D<T>,
    ray: &Ray3D<T>,
    tolerance: T,
) -> IntersectionResult<T> {
    let origin = ray.origin();
    let points =
        if crate::distance::ellipsoidal_solid3d_point3d_distance(ellipsoid, &origin) <= tolerance {
            vec![origin]
        } else {
            vec![]
        };
    IntersectionResult::from_option_points(points, false, tolerance)
}

fn ellipsoidal_surface3d_point3d_intersection_raw<T: Scalar>(
    ellipsoid: &EllipsoidalSurface3D<T>,
    point: &Point3D<T>,
    tolerance: T,
) -> Option<Point3D<T>> {
    point_intersection_if(
        point,
        crate::distance::ellipsoidal_surface3d_point3d_distance(ellipsoid, point) <= tolerance,
    )
}

pub fn ellipsoidal_surface3d_point3d_intersection<T: Scalar>(
    ellipsoid: &EllipsoidalSurface3D<T>,
    point: &Point3D<T>,
    tolerance: T,
) -> IntersectionResult<T> {
    IntersectionResult::from_option_point(
        ellipsoidal_surface3d_point3d_intersection_raw(ellipsoid, point, tolerance),
        false,
        tolerance,
    )
}

fn spherical_solid3d_point3d_intersection_raw<T: Scalar>(
    sphere: &SphericalSolid3D<T>,
    point: &Point3D<T>,
    tolerance: T,
) -> Option<Point3D<T>> {
    point_intersection_if(
        point,
        crate::distance::spherical_solid3d_point3d_distance(sphere, point) <= tolerance,
    )
}

pub fn spherical_solid3d_point3d_intersection<T: Scalar>(
    sphere: &SphericalSolid3D<T>,
    point: &Point3D<T>,
    tolerance: T,
) -> IntersectionResult<T> {
    IntersectionResult::from_option_point(
        spherical_solid3d_point3d_intersection_raw(sphere, point, tolerance),
        false,
        tolerance,
    )
}

fn spherical_solid3d_line3d_intersection_raw<T: Scalar>(
    sphere: &SphericalSolid3D<T>,
    line: &InfiniteLine3D<T>,
    tolerance: T,
) -> Option<Point3D<T>> {
    let (cx, cy, cz) = SphericalSolid3DProperties::center(sphere);
    let center = Point3D::new(cx, cy, cz);
    let line_point_tuple = InfiniteLine3DProperties::point(line);
    let line_dir_tuple = InfiniteLine3DProperties::direction(line);
    let line_point = Point3D::new(line_point_tuple.0, line_point_tuple.1, line_point_tuple.2);
    let line_dir = Vector3D::new(line_dir_tuple.0, line_dir_tuple.1, line_dir_tuple.2);
    if sphere.distance_to_infinite_line(&line_point, &line_dir) <= tolerance {
        Some(line.project_point(&center))
    } else {
        None
    }
}

pub fn spherical_solid3d_line3d_intersection<T: Scalar>(
    sphere: &SphericalSolid3D<T>,
    line: &InfiniteLine3D<T>,
    tolerance: T,
) -> IntersectionResult<T> {
    IntersectionResult::from_option_point(
        spherical_solid3d_line3d_intersection_raw(sphere, line, tolerance),
        false,
        tolerance,
    )
}

fn spherical_solid3d_ray3d_intersection_raw<T: Scalar>(
    sphere: &SphericalSolid3D<T>,
    ray: &Ray3D<T>,
    tolerance: T,
) -> Option<Point3D<T>> {
    let (cx, cy, cz) = SphericalSolid3DProperties::center(sphere);
    let center = Point3D::new(cx, cy, cz);
    let ray_direction = ray.direction_vector();
    if sphere.distance_to_ray(&ray.origin(), &ray_direction) <= tolerance {
        let parameter = ray.parameter_for_point(&center);
        let clamped = if parameter < T::ZERO {
            T::ZERO
        } else {
            parameter
        };
        Some(ray.point_at_parameter(clamped))
    } else {
        None
    }
}

pub fn spherical_solid3d_ray3d_intersection<T: Scalar>(
    sphere: &SphericalSolid3D<T>,
    ray: &Ray3D<T>,
    tolerance: T,
) -> IntersectionResult<T> {
    IntersectionResult::from_option_point(
        spherical_solid3d_ray3d_intersection_raw(sphere, ray, tolerance),
        false,
        tolerance,
    )
}

fn spherical_solid3d_line_segment3d_intersection_raw<T: Scalar>(
    sphere: &SphericalSolid3D<T>,
    segment: &LineSegment3D<T>,
    tolerance: T,
) -> Option<Point3D<T>> {
    let (cx, cy, cz) = SphericalSolid3DProperties::center(sphere);
    let center = Point3D::new(cx, cy, cz);
    if sphere.distance_to_line_segment(&segment.start(), &segment.end()) <= tolerance {
        let parameter = segment.line().parameter_for_point(&center);
        let clamped = if parameter < segment.start_param() {
            segment.start_param()
        } else if parameter > segment.end_param() {
            segment.end_param()
        } else {
            parameter
        };
        Some(segment.line().point_at_parameter(clamped))
    } else {
        None
    }
}

pub fn spherical_solid3d_line_segment3d_intersection<T: Scalar>(
    sphere: &SphericalSolid3D<T>,
    segment: &LineSegment3D<T>,
    tolerance: T,
) -> IntersectionResult<T> {
    IntersectionResult::from_option_point(
        spherical_solid3d_line_segment3d_intersection_raw(sphere, segment, tolerance),
        false,
        tolerance,
    )
}

fn spherical_solid3d_plane3d_intersection_raw<T: Scalar>(
    sphere: &SphericalSolid3D<T>,
    plane: &Plane3D<T>,
    tolerance: T,
) -> Option<Point3D<T>> {
    let (cx, cy, cz) = SphericalSolid3DProperties::center(sphere);
    let center = Point3D::new(cx, cy, cz);
    let radius = SphericalSolid3DProperties::radius(sphere);
    if plane.distance_to_point(center).abs() <= radius + tolerance {
        Some(plane.project_point(center))
    } else {
        None
    }
}

pub fn spherical_solid3d_plane3d_intersection<T: Scalar>(
    sphere: &SphericalSolid3D<T>,
    plane: &Plane3D<T>,
    tolerance: T,
) -> IntersectionResult<T> {
    IntersectionResult::from_option_point(
        spherical_solid3d_plane3d_intersection_raw(sphere, plane, tolerance),
        false,
        tolerance,
    )
}

fn torus_solid3d_point3d_intersection_raw<T: Scalar>(
    torus: &TorusSolid3D<T>,
    point: &Point3D<T>,
    tolerance: T,
) -> Option<Point3D<T>> {
    point_intersection_if(
        point,
        crate::distance::torus_solid3d_point3d_distance(torus, point) <= tolerance,
    )
}

pub fn torus_solid3d_point3d_intersection<T: Scalar>(
    torus: &TorusSolid3D<T>,
    point: &Point3D<T>,
    tolerance: T,
) -> IntersectionResult<T> {
    IntersectionResult::from_option_point(
        torus_solid3d_point3d_intersection_raw(torus, point, tolerance),
        false,
        tolerance,
    )
}

fn torus_surface3d_point3d_intersection_raw<T: Scalar>(
    torus: &TorusSurface3D<T>,
    point: &Point3D<T>,
    tolerance: T,
) -> Option<Point3D<T>> {
    point_intersection_if(
        point,
        crate::distance::torus_surface3d_point3d_distance(torus, point) <= tolerance,
    )
}

pub fn torus_surface3d_point3d_intersection<T: Scalar>(
    torus: &TorusSurface3D<T>,
    point: &Point3D<T>,
    tolerance: T,
) -> IntersectionResult<T> {
    IntersectionResult::from_option_point(
        torus_surface3d_point3d_intersection_raw(torus, point, tolerance),
        false,
        tolerance,
    )
}
