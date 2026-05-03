use crate::{
    ConicalSolid3D, ConicalSurface3D, InfiniteLine3D, LineSegment3D, Point3D, SphericalSurface3D,
    Triangle3D, Vector3D,
};
use geo_contracts::{
    ConicalSolid3DContainment, ConicalSurface3DProperties, InfiniteLine3DProperties, Scalar,
    SphericalSurface3DProperties, Triangle3DBoundaryAccess,
};

/// 2つの無限直線の交点計算（生の計算）
///
/// 平行またはスキューの場合は None を返す。
/// 呼び出し元で追加の検証（距離チェック等）を行う想定。
pub(super) fn line_line_intersection_raw<T: Scalar>(
    line1: &InfiniteLine3D<T>,
    line2: &InfiniteLine3D<T>,
) -> Option<Point3D<T>> {
    if line1.is_parallel_to(line2) {
        return None;
    }
    if !line1.is_coplanar_with(line2) {
        return None;
    }
    let (px1, py1, pz1) = InfiniteLine3DProperties::point(line1);
    let (dx1, dy1, dz1) = InfiniteLine3DProperties::direction(line1);
    let (px2, py2, pz2) = InfiniteLine3DProperties::point(line2);
    let (dx2, dy2, dz2) = InfiniteLine3DProperties::direction(line2);
    let p1 = Point3D::new(px1, py1, pz1);
    let d1 = Vector3D::new(dx1, dy1, dz1);
    let p2 = Point3D::new(px2, py2, pz2);
    let d2 = Vector3D::new(dx2, dy2, dz2);
    let dp = Vector3D::from_points(&p1, &p2);
    let cross_d1_d2 = d1.cross(&d2);
    let cross_dp_d2 = dp.cross(&d2);
    let t = cross_dp_d2.dot(&cross_d1_d2) / cross_d1_d2.dot(&cross_d1_d2);
    Some(Point3D::new(px1 + t * dx1, py1 + t * dy1, pz1 + t * dz1))
}

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
    point.distance_to(&segment.start()) <= tolerance
        || point.distance_to(&segment.end()) <= tolerance
}

pub(crate) fn conical_solid3d_contains_point_with_tolerance<T: Scalar>(
    cone: &ConicalSolid3D<T>,
    point: &Point3D<T>,
    tolerance: T,
) -> bool {
    ConicalSolid3DContainment::contains_point_tolerance(
        cone,
        (point.x(), point.y(), point.z()),
        tolerance,
    )
}

pub(crate) fn triangle3d_vertex_points<T: Scalar>(triangle: &Triangle3D<T>) -> [Point3D<T>; 3] {
    let (ax, ay, az) = Triangle3DBoundaryAccess::vertex_a(triangle);
    let (bx, by, bz) = Triangle3DBoundaryAccess::vertex_b(triangle);
    let (cx, cy, cz) = Triangle3DBoundaryAccess::vertex_c(triangle);
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
    let center_tuple = SphericalSurface3DProperties::center(sphere);
    let center = Point3D::new(center_tuple.0, center_tuple.1, center_tuple.2);
    let radius = SphericalSurface3DProperties::radius(sphere);
    let offset = crate::Vector3D::from_points(&center, start);

    let a = direction.dot(direction);
    if a.abs() <= tolerance {
        return None;
    }

    let b = offset.dot(direction) * (T::ONE + T::ONE);
    let c = offset.dot(&offset) - radius * radius;
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
    let (cx, cy, cz) = ConicalSurface3DProperties::apex(cone);
    let h = ConicalSurface3DProperties::height(cone);
    let (dx, dy, dz) = ConicalSurface3DProperties::axis(cone);
    let r = ConicalSurface3DProperties::radius(cone);

    if h.abs() <= tolerance || r.abs() <= tolerance {
        return [None, None];
    }

    let apex_x = cx - h * dx;
    let apex_y = cy - h * dy;
    let apex_z = cz - h * dz;
    let k = (h * h) / (h * h + r * r);

    let det_x = origin.x() - apex_x;
    let det_y = origin.y() - apex_y;
    let det_z = origin.z() - apex_z;

    let ld = direction.x() * dx + direction.y() * dy + direction.z() * dz;
    let dd = det_x * dx + det_y * dy + det_z * dz;
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
        if cone.contains_point(&p, tolerance) {
            pts.push(p);
        }
    }
    if pts.len() == 2 && pts[0].distance_to(&pts[1]) <= tolerance {
        pts.truncate(1);
    }
    pts
}
