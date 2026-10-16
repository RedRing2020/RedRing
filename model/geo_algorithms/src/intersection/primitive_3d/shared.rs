use crate::{
    ConicalSolid3D, ConicalSurface3D, LineSegment3D, Point3D, SphericalSurface3D, Triangle3D,
};
use geo_contracts::{
    ConicalSolid3DDistance, ConicalSurface3DDistance, ConicalSurface3DProperties,
    LineSegment3DProperties, Scalar, SphericalSurface3DProperties, Triangle3DBoundaryAccess,
};

pub(crate) fn point_intersection_if<T: Scalar>(
    point: &Point3D<T>,
    condition: bool,
) -> Option<Point3D<T>> {
    if condition {
        Some(Point3D::new(point.x(), point.y(), point.z()))
    } else {
        None
    }
}

pub(crate) fn point_matches_either_segment_endpoint<T: Scalar>(
    point: Point3D<T>,
    segment: &LineSegment3D<T>,
    tolerance: T,
) -> bool {
    point.distance_to(segment.start_point()) <= tolerance
        || point.distance_to(segment.end_point()) <= tolerance
}

pub(crate) fn conical_solid3d_contains_point_with_tolerance<T: Scalar>(
    cone: &ConicalSolid3D<T>,
    point: &Point3D<T>,
    tolerance: T,
) -> bool {
    cone.distance_to_point(*point) <= tolerance
}

pub(crate) fn triangle3d_vertex_points<T: Scalar>(triangle: &Triangle3D<T>) -> [Point3D<T>; 3] {
    let triangle_vertex_a = Triangle3DBoundaryAccess::vertex_a(triangle);
    let (ax, ay, az) = (
        triangle_vertex_a.x(),
        triangle_vertex_a.y(),
        triangle_vertex_a.z(),
    );
    let triangle_vertex_b = Triangle3DBoundaryAccess::vertex_b(triangle);
    let (bx, by, bz) = (
        triangle_vertex_b.x(),
        triangle_vertex_b.y(),
        triangle_vertex_b.z(),
    );
    let triangle_vertex_c = Triangle3DBoundaryAccess::vertex_c(triangle);
    let (cx, cy, cz) = (
        triangle_vertex_c.x(),
        triangle_vertex_c.y(),
        triangle_vertex_c.z(),
    );
    [
        Point3D::new(ax, ay, az),
        Point3D::new(bx, by, bz),
        Point3D::new(cx, cy, cz),
    ]
}

pub(crate) fn spherical_surface_intersection_parameters<T: Scalar>(
    start: &Point3D<T>,
    direction: &crate::Vector3D<T>,
    sphere: &SphericalSurface3D<T>,
    tolerance: T,
) -> Option<(T, T)> {
    let center = SphericalSurface3DProperties::center(sphere);
    let radius = SphericalSurface3DProperties::radius(sphere);
    let offset = crate::Vector3D::from_points(center, *start);

    let a = direction.dot(*direction);
    if a.abs() <= tolerance {
        return None;
    }

    let b = offset.dot(*direction) * (T::ONE + T::ONE);
    let c = offset.dot(offset) - radius * radius;
    let discriminant = b * b - (T::ONE + T::ONE + T::ONE + T::ONE) * a * c;
    if discriminant < T::ZERO {
        return None;
    }

    let sqrt_discriminant = discriminant.sqrt();
    let two_a = (T::ONE + T::ONE) * a;
    Some((
        (-b - sqrt_discriminant) / two_a,
        (-b + sqrt_discriminant) / two_a,
    ))
}

pub(crate) fn conical_surface3d_intersect_params<T: Scalar>(
    cone: &ConicalSurface3D<T>,
    origin: &Point3D<T>,
    direction: &crate::Vector3D<T>,
    tolerance: T,
) -> [Option<T>; 2] {
    // 頂点を原点とする二重円錐 ((p - apex)·axis)² = cos²(半頂角) |p - apex|² と直線の交点
    let apex = ConicalSurface3DProperties::apex(cone);
    let d = ConicalSurface3DProperties::axis(cone);
    let cos = ConicalSurface3DProperties::semi_angle(cone).cos();
    let apex_x = apex.x();
    let apex_y = apex.y();
    let apex_z = apex.z();
    let k = cos * cos;

    let det_x = origin.x() - apex_x;
    let det_y = origin.y() - apex_y;
    let det_z = origin.z() - apex_z;

    let ld = direction.x() * d.x() + direction.y() * d.y() + direction.z() * d.z();
    let dd = det_x * d.x() + det_y * d.y() + det_z * d.z();
    let ll = direction.x() * direction.x()
        + direction.y() * direction.y()
        + direction.z() * direction.z();
    let dl = det_x * direction.x() + det_y * direction.y() + det_z * direction.z();
    let dd_sq = det_x * det_x + det_y * det_y + det_z * det_z;

    let two = T::ONE + T::ONE;
    let four = two + two;
    let qa = ld * ld - k * ll;
    let qb = two * (dd * ld - k * dl);
    let qc = dd * dd - k * dd_sq;

    if qa.abs() <= tolerance {
        if qb.abs() > tolerance {
            return [Some(-qc / qb), None];
        }
        return [None, None];
    }

    let discriminant = qb * qb - four * qa * qc;
    if discriminant < T::ZERO {
        return [None, None];
    }

    let sqrt_d = discriminant.sqrt();
    let two_a = two * qa;
    [Some((-qb - sqrt_d) / two_a), Some((-qb + sqrt_d) / two_a)]
}

pub(crate) fn conical_surface3d_filter_params<T: Scalar>(
    cone: &ConicalSurface3D<T>,
    origin: &Point3D<T>,
    direction: &crate::Vector3D<T>,
    params: [Option<T>; 2],
    t_min: T,
    t_max: T,
    tolerance: T,
) -> Vec<Point3D<T>> {
    let mut pts = Vec::new();
    for maybe_t in params {
        let Some(t) = maybe_t else { continue };
        if t < t_min - tolerance || t > t_max + tolerance {
            continue;
        }
        let p = Point3D::new(
            origin.x() + t * direction.x(),
            origin.y() + t * direction.y(),
            origin.z() + t * direction.z(),
        );
        if cone.distance_to_point(p) <= tolerance {
            pts.push(p);
        }
    }
    if pts.len() == 2 && pts[0].distance_to(pts[1]) <= tolerance {
        pts.truncate(1);
    }
    pts
}
