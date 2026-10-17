use super::shared::{convex_solid_linear_intersection, point_intersection_if};
use crate::{InfiniteLine3D, IntersectionResult, Point3D, Ray3D, SphericalSolid3D, Vector3D};
use geo_contracts::{
    InfiniteLine3DProperties, Ray3DProperties, Scalar, SphericalSolid3DDistance,
    SphericalSolid3DProperties,
};

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

/// 球（立体）と直線 `origin + direction * t`（`lower <= t <= upper`）の交差
///
/// 直線上の点が球の内部に入るパラメータの区間を 2 次方程式で求める。
fn spherical_solid3d_linear_intersection<T: Scalar>(
    sphere: &SphericalSolid3D<T>,
    origin: Point3D<T>,
    direction: Vector3D<T>,
    lower: T,
    upper: T,
    tolerance: T,
) -> IntersectionResult<T> {
    let center = sphere.center();
    let radius = sphere.radius();
    let offset = Vector3D::from_points(center, origin);
    let a = direction.dot(direction);
    let b = offset.dot(direction);
    let discriminant = b * b - a * (offset.dot(offset) - radius * radius);
    let overlap = (discriminant >= T::ZERO)
        .then(|| {
            let root = discriminant.sqrt();
            (((-b - root) / a).max(lower), ((-b + root) / a).min(upper))
        })
        .filter(|(enter, exit)| enter <= exit);
    let closest = || {
        let parameter = (-b / a).max(lower).min(upper);
        let distance = (origin + direction * parameter).distance_to(center) - radius;
        (parameter, distance)
    };
    convex_solid_linear_intersection(origin, direction, overlap, closest, tolerance)
}

pub fn spherical_solid3d_infinite_line3d_intersection<T: Scalar>(
    sphere: &SphericalSolid3D<T>,
    line: &InfiniteLine3D<T>,
    tolerance: T,
) -> IntersectionResult<T> {
    spherical_solid3d_linear_intersection(
        sphere,
        line.point(),
        line.direction().as_vector(),
        -T::INFINITY,
        T::INFINITY,
        tolerance,
    )
}

pub fn spherical_solid3d_ray3d_intersection<T: Scalar>(
    sphere: &SphericalSolid3D<T>,
    ray: &Ray3D<T>,
    tolerance: T,
) -> IntersectionResult<T> {
    spherical_solid3d_linear_intersection(
        sphere,
        ray.origin(),
        ray.direction().as_vector(),
        T::ZERO,
        T::INFINITY,
        tolerance,
    )
}
