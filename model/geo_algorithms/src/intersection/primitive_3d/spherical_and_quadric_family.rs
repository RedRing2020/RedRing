use super::shared::point_intersection_if;
use crate::{IntersectionResult, Point3D, SphericalSolid3D};
use geo_contracts::{Scalar, SphericalSolid3DDistance};

fn spherical_solid3d_point3d_intersection_raw<T: Scalar>(
    sphere: &SphericalSolid3D<T>,
    point: &Point3D<T>,
    tolerance: T,
) -> Option<Point3D<T>> {
    point_intersection_if(point, sphere.distance_to_point(*point) <= tolerance)
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
