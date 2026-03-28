//! 3D Primitive intersection algorithms
//!
//! Phase C Step 2: `geo_primitives` から 3D 交差判定ロジックを
//! `geo_algorithms` 側へ集約するための受け皿。
//!
//! orphan rules により trait 実装ではなく形状ペア free-function を提供する。
//! `geo_primitives` の `BasicIntersection`/`MultipleIntersection` を呼び出す薄い
//! ラッパー、または `pair_base` に委譲する。

use crate::{
    Arc3D, Circle3D, CylindricalSurface3D, Ellipse3D, EllipsoidalSolid3D, EllipsoidalSurface3D,
    InfiniteLine3D, IntersectionResult, LineSegment3D, Plane3D, Point3D, Ray3D, SphericalSurface3D,
    TorusSolid3D, TorusSurface3D, Triangle3D, TriangleMesh3D,
};
use geo_contracts::{
    Arc3DMeasure, Arc3DProperties, Circle3DProperties, CylindricalSurface3DMeasure,
    CylindricalSurface3DProperties, Ellipse3DMeasure, EllipsoidalSolid3DProperties,
    InfiniteLine3DProperties, Scalar, SphericalSurface3DProperties, TorusSurface3DMeasure,
    Triangle3DProperties,
};

fn point_intersection_if<T: Scalar>(point: &Point3D<T>, condition: bool) -> Option<Point3D<T>> {
    if condition {
        Some(Point3D::new(point.x(), point.y(), point.z()))
    } else {
        None
    }
}

fn spherical_surface_intersection_parameters<T: Scalar>(
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

// ── Arc3D ─────────────────────────────────────────────────────────────────────

fn arc3d_point3d_intersection_raw<T: Scalar>(
    arc: &Arc3D<T>,
    point: &Point3D<T>,
    tolerance: T,
) -> Option<Point3D<T>> {
    let point_tuple = (point.x(), point.y(), point.z());
    point_intersection_if(
        point,
        <Arc3D<T> as Arc3DMeasure<T>>::distance_to_point(arc, point_tuple) <= tolerance
            && arc.contains_point_angle(Point3D::new(point.x(), point.y(), point.z())),
    )
}

pub fn arc3d_point3d_intersection<T: Scalar>(
    arc: &Arc3D<T>,
    point: &Point3D<T>,
    tolerance: T,
) -> IntersectionResult<T> {
    IntersectionResult::from_option_point(
        arc3d_point3d_intersection_raw(arc, point, tolerance),
        false,
        tolerance,
    )
}

fn arc3d_line_segment3d_intersection_raw<T: Scalar>(
    arc: &Arc3D<T>,
    segment: &LineSegment3D<T>,
    tolerance: T,
) -> Option<Point3D<T>> {
    let (sx, sy, sz) = <Arc3D<T> as Arc3DMeasure<T>>::start_point(arc);
    let (ex, ey, ez) = <Arc3D<T> as Arc3DMeasure<T>>::end_point(arc);
    let arc_start = Point3D::new(sx, sy, sz);
    let arc_end = Point3D::new(ex, ey, ez);
    let d_seg_s = <Arc3D<T> as Arc3DMeasure<T>>::distance_to_point(
        arc,
        (
            segment.start().x(),
            segment.start().y(),
            segment.start().z(),
        ),
    );
    let d_seg_e = <Arc3D<T> as Arc3DMeasure<T>>::distance_to_point(
        arc,
        (segment.end().x(), segment.end().y(), segment.end().z()),
    );
    if d_seg_s <= tolerance {
        return Some(segment.start());
    }
    if d_seg_e <= tolerance {
        return Some(segment.end());
    }
    let d1 = crate::Vector3D::from_points(&arc_start, &segment.start()).magnitude();
    let d2 = crate::Vector3D::from_points(&arc_end, &segment.start()).magnitude();
    if d1 <= tolerance {
        Some(arc_start)
    } else if d2 <= tolerance {
        Some(arc_end)
    } else {
        None
    }
}

pub fn arc3d_line_segment3d_intersection<T: Scalar>(
    arc: &Arc3D<T>,
    segment: &LineSegment3D<T>,
    tolerance: T,
) -> IntersectionResult<T> {
    IntersectionResult::from_option_point(
        arc3d_line_segment3d_intersection_raw(arc, segment, tolerance),
        false,
        tolerance,
    )
}

fn arc3d_ray3d_intersection_raw<T: Scalar>(
    arc: &Arc3D<T>,
    ray: &Ray3D<T>,
    tolerance: T,
) -> Option<Point3D<T>> {
    let origin = ray.origin();
    let d =
        <Arc3D<T> as Arc3DMeasure<T>>::distance_to_point(arc, (origin.x(), origin.y(), origin.z()));
    if d <= tolerance {
        Some(origin)
    } else {
        None
    }
}

pub fn arc3d_ray3d_intersection<T: Scalar>(
    arc: &Arc3D<T>,
    ray: &Ray3D<T>,
    tolerance: T,
) -> IntersectionResult<T> {
    IntersectionResult::from_option_point(
        arc3d_ray3d_intersection_raw(arc, ray, tolerance),
        false,
        tolerance,
    )
}

fn arc3d_infinite_line3d_intersection_raw<T: Scalar>(
    arc: &Arc3D<T>,
    line: &InfiniteLine3D<T>,
    tolerance: T,
) -> Option<Point3D<T>> {
    let (px, py, pz) = InfiniteLine3DProperties::point(line);
    let pt = Point3D::new(px, py, pz);
    let d = <Arc3D<T> as Arc3DMeasure<T>>::distance_to_point(arc, (px, py, pz));
    if d <= tolerance {
        Some(pt)
    } else {
        None
    }
}

pub fn arc3d_infinite_line3d_intersection<T: Scalar>(
    arc: &Arc3D<T>,
    line: &InfiniteLine3D<T>,
    tolerance: T,
) -> IntersectionResult<T> {
    IntersectionResult::from_option_point(
        arc3d_infinite_line3d_intersection_raw(arc, line, tolerance),
        false,
        tolerance,
    )
}

fn arc3d_arc3d_intersection_raw<T: Scalar>(
    arc_a: &Arc3D<T>,
    arc_b: &Arc3D<T>,
    tolerance: T,
) -> Option<Point3D<T>> {
    let (s1x, s1y, s1z) = <Arc3D<T> as Arc3DMeasure<T>>::start_point(arc_a);
    let (e1x, e1y, e1z) = <Arc3D<T> as Arc3DMeasure<T>>::end_point(arc_a);
    let (s2x, s2y, s2z) = <Arc3D<T> as Arc3DMeasure<T>>::start_point(arc_b);
    let (e2x, e2y, e2z) = <Arc3D<T> as Arc3DMeasure<T>>::end_point(arc_b);
    let pa = Point3D::new(s1x, s1y, s1z);
    let pb = Point3D::new(e1x, e1y, e1z);
    let pc = Point3D::new(s2x, s2y, s2z);
    let pd = Point3D::new(e2x, e2y, e2z);
    if crate::Vector3D::from_points(&pa, &pc).magnitude() <= tolerance {
        return Some(pa);
    }
    if crate::Vector3D::from_points(&pa, &pd).magnitude() <= tolerance {
        return Some(pa);
    }
    if crate::Vector3D::from_points(&pb, &pc).magnitude() <= tolerance {
        return Some(pb);
    }
    if crate::Vector3D::from_points(&pb, &pd).magnitude() <= tolerance {
        return Some(pb);
    }
    None
}

pub fn arc3d_arc3d_intersection<T: Scalar>(
    arc_a: &Arc3D<T>,
    arc_b: &Arc3D<T>,
    tolerance: T,
) -> IntersectionResult<T> {
    IntersectionResult::from_option_point(
        arc3d_arc3d_intersection_raw(arc_a, arc_b, tolerance),
        false,
        tolerance,
    )
}

// ── Circle3D ──────────────────────────────────────────────────────────────────

fn circle3d_point3d_intersection_raw<T: Scalar>(
    circle: &Circle3D<T>,
    point: &Point3D<T>,
    tolerance: T,
) -> Option<Point3D<T>> {
    point_intersection_if(
        point,
        circle.distance_to_point_3d(Point3D::new(point.x(), point.y(), point.z())) <= tolerance,
    )
}

pub fn circle3d_point3d_intersection<T: Scalar>(
    circle: &Circle3D<T>,
    point: &Point3D<T>,
    tolerance: T,
) -> IntersectionResult<T> {
    IntersectionResult::from_option_point(
        circle3d_point3d_intersection_raw(circle, point, tolerance),
        false,
        tolerance,
    )
}

fn circle3d_line_segment3d_intersection_raw<T: Scalar>(
    circle: &Circle3D<T>,
    segment: &LineSegment3D<T>,
    _tolerance: T,
) -> Option<Point3D<T>> {
    if circle.contains_point_3d(segment.start()) {
        return Some(segment.start());
    }
    if circle.contains_point_3d(segment.end()) {
        return Some(segment.end());
    }
    None
}

pub fn circle3d_line_segment3d_intersection<T: Scalar>(
    circle: &Circle3D<T>,
    segment: &LineSegment3D<T>,
    tolerance: T,
) -> IntersectionResult<T> {
    IntersectionResult::from_option_point(
        circle3d_line_segment3d_intersection_raw(circle, segment, tolerance),
        false,
        tolerance,
    )
}

fn circle3d_ray3d_intersection_raw<T: Scalar>(
    circle: &Circle3D<T>,
    ray: &Ray3D<T>,
    _tolerance: T,
) -> Option<Point3D<T>> {
    let origin = ray.origin();
    if circle.contains_point_3d(origin) {
        Some(origin)
    } else {
        None
    }
}

pub fn circle3d_ray3d_intersection<T: Scalar>(
    circle: &Circle3D<T>,
    ray: &Ray3D<T>,
    tolerance: T,
) -> IntersectionResult<T> {
    IntersectionResult::from_option_point(
        circle3d_ray3d_intersection_raw(circle, ray, tolerance),
        false,
        tolerance,
    )
}

fn circle3d_infinite_line3d_intersection_raw<T: Scalar>(
    circle: &Circle3D<T>,
    line: &InfiniteLine3D<T>,
    _tolerance: T,
) -> Option<Point3D<T>> {
    let (px, py, pz) = InfiniteLine3DProperties::point(line);
    let pt = Point3D::new(px, py, pz);
    if circle.contains_point_3d(pt) {
        Some(pt)
    } else {
        None
    }
}

pub fn circle3d_infinite_line3d_intersection<T: Scalar>(
    circle: &Circle3D<T>,
    line: &InfiniteLine3D<T>,
    tolerance: T,
) -> IntersectionResult<T> {
    IntersectionResult::from_option_point(
        circle3d_infinite_line3d_intersection_raw(circle, line, tolerance),
        false,
        tolerance,
    )
}

pub fn circle3d_circle3d_intersection<T: Scalar>(
    _circle_a: &Circle3D<T>,
    _circle_b: &Circle3D<T>,
    tolerance: T,
) -> IntersectionResult<T> {
    IntersectionResult::from_option_point(None, false, tolerance)
}

// ── CylindricalSurface3D ──────────────────────────────────────────────────────

fn cylindrical_surface3d_point3d_intersection_raw<T: Scalar>(
    cyl: &CylindricalSurface3D<T>,
    point: &Point3D<T>,
    tolerance: T,
) -> Option<Point3D<T>> {
    point_intersection_if(
        point,
        <CylindricalSurface3D<T> as CylindricalSurface3DMeasure<T>>::distance_to_point(
            cyl,
            (point.x(), point.y(), point.z()),
        ) <= tolerance,
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
    let dist = <CylindricalSurface3D<T> as CylindricalSurface3DMeasure<T>>::distance_to_point(
        cyl,
        (cx, cy, cz),
    );
    if dist <= Circle3DProperties::radius(circle) + tolerance {
        Some(Point3D::new(cx, cy, cz))
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
    if <CylindricalSurface3D<T> as CylindricalSurface3DMeasure<T>>::distance_to_point(
        cyl,
        (s.x(), s.y(), s.z()),
    ) <= tolerance
    {
        Some(s)
    } else if <CylindricalSurface3D<T> as CylindricalSurface3DMeasure<T>>::distance_to_point(
        cyl,
        (e.x(), e.y(), e.z()),
    ) <= tolerance
    {
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
    let (ax, ay, az) = Triangle3DProperties::vertex_a(triangle);
    let (bx, by, bz) = Triangle3DProperties::vertex_b(triangle);
    let (cx, cy, cz) = Triangle3DProperties::vertex_c(triangle);
    if <CylindricalSurface3D<T> as CylindricalSurface3DMeasure<T>>::distance_to_point(
        cyl,
        (ax, ay, az),
    ) <= tolerance
    {
        Some(Point3D::new(ax, ay, az))
    } else if <CylindricalSurface3D<T> as CylindricalSurface3DMeasure<T>>::distance_to_point(
        cyl,
        (bx, by, bz),
    ) <= tolerance
    {
        Some(Point3D::new(bx, by, bz))
    } else if <CylindricalSurface3D<T> as CylindricalSurface3DMeasure<T>>::distance_to_point(
        cyl,
        (cx, cy, cz),
    ) <= tolerance
    {
        Some(Point3D::new(cx, cy, cz))
    } else {
        None
    }
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

// cylindrical_surface3d_cylindrical_surface3d_intersection: BasicIntersection<T, CylindricalSurface3D<T>> は未実装のため対象外

// ── Ellipse3D ─────────────────────────────────────────────────────────────────

fn ellipse3d_point3d_intersection_raw<T: Scalar + From<f64>>(
    ellipse: &Ellipse3D<T>,
    point: &Point3D<T>,
    tolerance: T,
) -> Option<Point3D<T>> {
    point_intersection_if(
        point,
        <Ellipse3D<T> as Ellipse3DMeasure<T>>::distance_to_point_3d(
            ellipse,
            (point.x(), point.y(), point.z()),
        ) <= tolerance,
    )
}

pub fn ellipse3d_point3d_intersection<T: Scalar + From<f64>>(
    ellipse: &Ellipse3D<T>,
    point: &Point3D<T>,
    tolerance: T,
) -> IntersectionResult<T> {
    IntersectionResult::from_option_point(
        ellipse3d_point3d_intersection_raw(ellipse, point, tolerance),
        false,
        tolerance,
    )
}

pub fn ellipse3d_circle3d_intersections<T: Scalar + From<f64>>(
    ellipse: &Ellipse3D<T>,
    circle: &Circle3D<T>,
    tolerance: T,
) -> Vec<Point3D<T>> {
    let (cx, cy, cz) = Circle3DProperties::center(circle);
    let dist = <Ellipse3D<T> as Ellipse3DMeasure<T>>::distance_to_point_3d(ellipse, (cx, cy, cz));
    if dist <= Circle3DProperties::radius(circle) + tolerance {
        vec![Point3D::new(cx, cy, cz)]
    } else {
        Vec::new()
    }
}

pub fn ellipse3d_arc3d_intersections<T: Scalar + From<f64>>(
    ellipse: &Ellipse3D<T>,
    arc: &Arc3D<T>,
    tolerance: T,
) -> Vec<Point3D<T>> {
    let (cx, cy, cz) = Arc3DProperties::center(arc);
    let dist = <Ellipse3D<T> as Ellipse3DMeasure<T>>::distance_to_point_3d(ellipse, (cx, cy, cz));
    if dist <= Arc3DProperties::radius(arc) + tolerance {
        vec![Point3D::new(cx, cy, cz)]
    } else {
        Vec::new()
    }
}

pub fn ellipse3d_line_segment3d_intersections<T: Scalar + From<f64>>(
    ellipse: &Ellipse3D<T>,
    segment: &LineSegment3D<T>,
    tolerance: T,
) -> Vec<Point3D<T>> {
    let start = segment.start();
    let end = segment.end();
    let mut intersections = Vec::new();
    if <Ellipse3D<T> as Ellipse3DMeasure<T>>::distance_to_point_3d(
        ellipse,
        (start.x(), start.y(), start.z()),
    ) <= tolerance
    {
        intersections.push(start);
    }
    if <Ellipse3D<T> as Ellipse3DMeasure<T>>::distance_to_point_3d(
        ellipse,
        (end.x(), end.y(), end.z()),
    ) <= tolerance
    {
        intersections.push(end);
    }
    intersections
}

pub fn ellipse3d_infinite_line3d_intersections<T: Scalar + From<f64>>(
    ellipse: &Ellipse3D<T>,
    line: &InfiniteLine3D<T>,
    tolerance: T,
) -> Vec<Point3D<T>> {
    let (px, py, pz) = InfiniteLine3DProperties::point(line);
    let dist = <Ellipse3D<T> as Ellipse3DMeasure<T>>::distance_to_point_3d(ellipse, (px, py, pz));
    if dist <= tolerance {
        vec![Point3D::new(px, py, pz)]
    } else {
        Vec::new()
    }
}

pub fn ellipse3d_ray3d_intersections<T: Scalar + From<f64>>(
    ellipse: &Ellipse3D<T>,
    ray: &Ray3D<T>,
    tolerance: T,
) -> Vec<Point3D<T>> {
    let o = ray.origin();
    let dist =
        <Ellipse3D<T> as Ellipse3DMeasure<T>>::distance_to_point_3d(ellipse, (o.x(), o.y(), o.z()));
    if dist <= tolerance {
        vec![o]
    } else {
        Vec::new()
    }
}

fn ellipse3d_plane3d_intersection_raw<T: Scalar>(
    ellipse: &Ellipse3D<T>,
    plane: &Plane3D<T>,
    tolerance: T,
) -> Option<Point3D<T>> {
    let center = ellipse.center();
    if plane.distance_to_point(center).abs() <= tolerance {
        Some(ellipse.center())
    } else {
        None
    }
}

pub fn ellipse3d_plane3d_intersection<T: Scalar>(
    ellipse: &Ellipse3D<T>,
    plane: &Plane3D<T>,
    tolerance: T,
) -> IntersectionResult<T> {
    IntersectionResult::from_option_point(
        ellipse3d_plane3d_intersection_raw(ellipse, plane, tolerance),
        false,
        tolerance,
    )
}

pub fn ellipse3d_triangle3d_intersections<T: Scalar + From<f64>>(
    ellipse: &Ellipse3D<T>,
    triangle: &Triangle3D<T>,
    tolerance: T,
) -> Vec<Point3D<T>> {
    let (ax, ay, az) = Triangle3DProperties::vertex_a(triangle);
    let (bx, by, bz) = Triangle3DProperties::vertex_b(triangle);
    let (cx, cy, cz) = Triangle3DProperties::vertex_c(triangle);
    let mut intersections = Vec::new();
    if <Ellipse3D<T> as Ellipse3DMeasure<T>>::distance_to_point_3d(ellipse, (ax, ay, az))
        <= tolerance
    {
        intersections.push(Point3D::new(ax, ay, az));
    }
    if <Ellipse3D<T> as Ellipse3DMeasure<T>>::distance_to_point_3d(ellipse, (bx, by, bz))
        <= tolerance
    {
        intersections.push(Point3D::new(bx, by, bz));
    }
    if <Ellipse3D<T> as Ellipse3DMeasure<T>>::distance_to_point_3d(ellipse, (cx, cy, cz))
        <= tolerance
    {
        intersections.push(Point3D::new(cx, cy, cz));
    }
    intersections
}

pub fn ellipse3d_ellipse3d_intersections<T: Scalar>(
    _ellipse_a: &Ellipse3D<T>,
    _ellipse_b: &Ellipse3D<T>,
    _tolerance: T,
) -> Vec<Point3D<T>> {
    Vec::new()
}

// ── EllipsoidalSolid3D ────────────────────────────────────────────────────────

fn ellipsoidal_solid3d_point3d_intersection_raw<T: Scalar>(
    ellipsoid: &EllipsoidalSolid3D<T>,
    point: &Point3D<T>,
    _tolerance: T,
) -> Option<Point3D<T>> {
    point_intersection_if(point, ellipsoid.contains_point(point))
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
    _tolerance: T,
) -> Vec<Point3D<T>> {
    let (px, py, pz) = InfiniteLine3DProperties::point(line);
    let point_on_line = Point3D::new(px, py, pz);
    if ellipsoid.contains_point(&point_on_line) {
        vec![point_on_line]
    } else {
        vec![]
    }
}

pub fn ellipsoidal_solid3d_ray3d_intersections<T: Scalar>(
    ellipsoid: &EllipsoidalSolid3D<T>,
    ray: &Ray3D<T>,
    _tolerance: T,
) -> Vec<Point3D<T>> {
    let origin = ray.origin();
    if ellipsoid.contains_point(&origin) {
        vec![origin]
    } else {
        vec![]
    }
}

// ── EllipsoidalSurface3D ──────────────────────────────────────────────────────

fn ellipsoidal_surface3d_point3d_intersection_raw<T: Scalar>(
    ellipsoid: &EllipsoidalSurface3D<T>,
    point: &Point3D<T>,
    tolerance: T,
) -> Option<Point3D<T>> {
    point_intersection_if(point, ellipsoid.contains_point(point, tolerance))
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

// SphericalSolid3D の intersection: spherical_solid_3d_intersection モジュールが
// lib.rs でコメントアウトされているため BasicIntersection/MultipleIntersection は未実装。
// collision 側（primitive_3d.rs）の spherical_solid3d_*_collides を使用すること。

// ── TorusSolid3D ──────────────────────────────────────────────────────────────

fn torus_solid3d_point3d_intersection_raw<T: Scalar>(
    torus: &TorusSolid3D<T>,
    point: &Point3D<T>,
    _tolerance: T,
) -> Option<Point3D<T>> {
    point_intersection_if(point, torus.contains_point(point))
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

// ── TorusSurface3D ────────────────────────────────────────────────────────────

fn torus_surface3d_point3d_intersection_raw<T: Scalar>(
    torus: &TorusSurface3D<T>,
    point: &Point3D<T>,
    tolerance: T,
) -> Option<Point3D<T>> {
    point_intersection_if(
        point,
        <TorusSurface3D<T> as TorusSurface3DMeasure<T>>::distance_to_point(
            torus,
            (point.x(), point.y(), point.z()),
        ) <= tolerance,
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

// ── Triangle3D ────────────────────────────────────────────────────────────────

fn triangle3d_point3d_intersection_raw<T: Scalar>(
    triangle: &Triangle3D<T>,
    point: &Point3D<T>,
    tolerance: T,
) -> Option<Point3D<T>> {
    point_intersection_if(point, triangle.distance_to_point(point) <= tolerance)
}

pub fn triangle3d_point3d_intersection<T: Scalar>(
    triangle: &Triangle3D<T>,
    point: &Point3D<T>,
    tolerance: T,
) -> IntersectionResult<T> {
    IntersectionResult::from_option_point(
        triangle3d_point3d_intersection_raw(triangle, point, tolerance),
        false,
        tolerance,
    )
}

fn triangle3d_line_segment3d_intersection_raw<T: Scalar>(
    triangle: &Triangle3D<T>,
    segment: &LineSegment3D<T>,
    tolerance: T,
) -> Option<Point3D<T>> {
    let start = segment.start();
    let end = segment.end();
    let dir = end - start;
    let length = dir.length();
    if length <= tolerance {
        return None;
    }
    let ray = Ray3D::new(start, dir)?;
    let point = triangle3d_ray3d_intersection_raw(triangle, &ray, tolerance)?;

    let t = ray.parameter_for_point(&point);
    if t >= T::ZERO && t <= length {
        Some(point)
    } else {
        None
    }
}

pub fn triangle3d_line_segment3d_intersection<T: Scalar>(
    triangle: &Triangle3D<T>,
    segment: &LineSegment3D<T>,
    tolerance: T,
) -> IntersectionResult<T> {
    IntersectionResult::from_option_point(
        triangle3d_line_segment3d_intersection_raw(triangle, segment, tolerance),
        false,
        tolerance,
    )
}

pub fn line_segment3d_triangle3d_intersection<T: Scalar>(
    segment: &LineSegment3D<T>,
    triangle: &Triangle3D<T>,
    tolerance: T,
) -> IntersectionResult<T> {
    triangle3d_line_segment3d_intersection(triangle, segment, tolerance)
}

fn triangle3d_ray3d_intersection_raw<T: Scalar>(
    triangle: &Triangle3D<T>,
    ray: &Ray3D<T>,
    tolerance: T,
) -> Option<Point3D<T>> {
    let va = triangle.vertex_a();
    let vb = triangle.vertex_b();
    let vc = triangle.vertex_c();

    let v0 = Point3D::new(va.0, va.1, va.2);
    let v1 = Point3D::new(vb.0, vb.1, vb.2);
    let v2 = Point3D::new(vc.0, vc.1, vc.2);

    let edge1 = crate::Vector3D::from_points(&v0, &v1);
    let edge2 = crate::Vector3D::from_points(&v0, &v2);

    let ray_dir = ray.direction_vector();
    let h = ray_dir.cross(&edge2);
    let a = edge1.dot(&h);

    if a.abs() <= tolerance {
        return None;
    }

    let f = T::ONE / a;
    let s = crate::Vector3D::from_points(&v0, &ray.origin());
    let u = f * s.dot(&h);

    if u < T::ZERO || u > T::ONE {
        return None;
    }

    let q = s.cross(&edge1);
    let v = f * ray_dir.dot(&q);

    if v < T::ZERO || u + v > T::ONE {
        return None;
    }

    let t = f * edge2.dot(&q);
    if t < T::ZERO {
        return None;
    }

    Some(Point3D::new(
        ray.origin().x() + t * ray_dir.x(),
        ray.origin().y() + t * ray_dir.y(),
        ray.origin().z() + t * ray_dir.z(),
    ))
}

pub fn triangle3d_ray3d_intersection<T: Scalar>(
    triangle: &Triangle3D<T>,
    ray: &Ray3D<T>,
    tolerance: T,
) -> IntersectionResult<T> {
    IntersectionResult::from_option_point(
        triangle3d_ray3d_intersection_raw(triangle, ray, tolerance),
        false,
        tolerance,
    )
}

pub fn ray3d_triangle3d_intersection<T: Scalar>(
    ray: &Ray3D<T>,
    triangle: &Triangle3D<T>,
    tolerance: T,
) -> IntersectionResult<T> {
    triangle3d_ray3d_intersection(triangle, ray, tolerance)
}

// ── TriangleMesh3D ────────────────────────────────────────────────────────────

fn triangle_mesh3d_point3d_intersection_raw<T: Scalar>(
    mesh: &TriangleMesh3D<T>,
    point: &Point3D<T>,
    tolerance: T,
) -> Option<Point3D<T>> {
    point_intersection_if(
        point,
        (0..mesh.triangle_count()).any(|i| {
            mesh.triangle(i)
                .map(|tri| tri.distance_to_point(point) <= tolerance)
                .unwrap_or(false)
        }),
    )
}

pub fn triangle_mesh3d_point3d_intersection<T: Scalar>(
    mesh: &TriangleMesh3D<T>,
    point: &Point3D<T>,
    tolerance: T,
) -> IntersectionResult<T> {
    IntersectionResult::from_option_point(
        triangle_mesh3d_point3d_intersection_raw(mesh, point, tolerance),
        false,
        tolerance,
    )
}

// ── Plane3D ───────────────────────────────────────────────────────────────────

fn plane3d_point3d_intersection_raw<T: Scalar>(
    plane: &Plane3D<T>,
    point: &Point3D<T>,
    tolerance: T,
) -> Option<Point3D<T>> {
    point_intersection_if(
        point,
        plane.contains_point(Point3D::new(point.x(), point.y(), point.z()), tolerance),
    )
}

pub fn plane3d_point3d_intersection<T: Scalar>(
    plane: &Plane3D<T>,
    point: &Point3D<T>,
    tolerance: T,
) -> IntersectionResult<T> {
    IntersectionResult::from_option_point(
        plane3d_point3d_intersection_raw(plane, point, tolerance),
        false,
        tolerance,
    )
}

fn plane3d_line_segment3d_intersection_raw<T: Scalar>(
    plane: &Plane3D<T>,
    segment: &LineSegment3D<T>,
    tolerance: T,
) -> Option<Point3D<T>> {
    let start = segment.start();
    let end = segment.end();
    let direction = crate::Vector3D::from_points(&start, &end);

    if direction.is_zero() {
        return if plane.contains_point(start, tolerance) {
            Some(start)
        } else {
            None
        };
    }

    let normal = plane.normal().as_vector();
    let denom = direction.dot(&normal);

    if denom.abs() <= tolerance {
        return if plane.contains_point(start, tolerance) {
            Some(start)
        } else {
            None
        };
    }

    let to_plane = crate::Vector3D::from_points(&start, &plane.origin());
    let t = to_plane.dot(&normal) / denom;

    if t >= T::ZERO && t <= T::ONE {
        Some(Point3D::new(
            start.x() + t * direction.x(),
            start.y() + t * direction.y(),
            start.z() + t * direction.z(),
        ))
    } else {
        None
    }
}

pub fn plane3d_line_segment3d_intersection<T: Scalar>(
    plane: &Plane3D<T>,
    segment: &LineSegment3D<T>,
    tolerance: T,
) -> IntersectionResult<T> {
    IntersectionResult::from_option_point(
        plane3d_line_segment3d_intersection_raw(plane, segment, tolerance),
        false,
        tolerance,
    )
}

fn plane3d_ray3d_intersection_raw<T: Scalar>(
    plane: &Plane3D<T>,
    ray: &Ray3D<T>,
    tolerance: T,
) -> Option<Point3D<T>> {
    let origin = ray.origin();
    let direction = ray.direction_vector();
    let normal = plane.normal().as_vector();
    let denom = direction.dot(&normal);

    if denom.abs() <= tolerance {
        return if plane.contains_point(origin, tolerance) {
            Some(origin)
        } else {
            None
        };
    }

    let to_plane = crate::Vector3D::from_points(&origin, &plane.origin());
    let t = to_plane.dot(&normal) / denom;

    if t >= T::ZERO {
        Some(Point3D::new(
            origin.x() + t * direction.x(),
            origin.y() + t * direction.y(),
            origin.z() + t * direction.z(),
        ))
    } else {
        None
    }
}

pub fn plane3d_ray3d_intersection<T: Scalar>(
    plane: &Plane3D<T>,
    ray: &Ray3D<T>,
    tolerance: T,
) -> IntersectionResult<T> {
    IntersectionResult::from_option_point(
        plane3d_ray3d_intersection_raw(plane, ray, tolerance),
        false,
        tolerance,
    )
}

fn plane3d_infinite_line3d_intersection_raw<T: Scalar>(
    plane: &Plane3D<T>,
    line: &InfiniteLine3D<T>,
    tolerance: T,
) -> Option<Point3D<T>> {
    let lp = line.point();
    let ld = line.direction();
    let line_point = Point3D::new(lp.0, lp.1, lp.2);
    let line_dir_vec = crate::Vector3D::new(ld.0, ld.1, ld.2);
    let normal = plane.normal().as_vector();
    let denom = line_dir_vec.dot(&normal);

    if denom.abs() <= tolerance {
        return if plane.distance_to_point(line_point).abs() <= tolerance {
            Some(line_point)
        } else {
            None
        };
    }

    let to_plane = crate::Vector3D::from_points(&line_point, &plane.origin());
    let t = to_plane.dot(&normal) / denom;
    Some(Point3D::new(
        line_point.x() + t * line_dir_vec.x(),
        line_point.y() + t * line_dir_vec.y(),
        line_point.z() + t * line_dir_vec.z(),
    ))
}

pub fn plane3d_infinite_line3d_intersection<T: Scalar>(
    plane: &Plane3D<T>,
    line: &InfiniteLine3D<T>,
    tolerance: T,
) -> IntersectionResult<T> {
    IntersectionResult::from_option_point(
        plane3d_infinite_line3d_intersection_raw(plane, line, tolerance),
        false,
        tolerance,
    )
}

// ── Ray3D ─────────────────────────────────────────────────────────────────────

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
) -> Vec<Point3D<T>> {
    let mut intersections = Vec::new();
    let start = ray.origin();
    let direction = ray.direction_vector();

    let Some((t1, t2)) =
        spherical_surface_intersection_parameters(&start, &direction, sphere, tolerance)
    else {
        return intersections;
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

    intersections
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
    let origin_offset = crate::Vector3D::from_points(&origin_b, &origin_a);

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

// ── LineSegment3D ─────────────────────────────────────────────────────────────

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
) -> Vec<Point3D<T>> {
    let mut intersections = Vec::new();
    let start = segment.start();
    let end = segment.end();
    let direction = crate::Vector3D::from_points(&start, &end);

    let Some((t1, t2)) =
        spherical_surface_intersection_parameters(&start, &direction, sphere, tolerance)
    else {
        return intersections;
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

    intersections
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

    let d1 = crate::Vector3D::from_points(&p1, &p2);
    let d2 = crate::Vector3D::from_points(&p3, &p4);
    let r = crate::Vector3D::from_points(&p3, &p1);

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

// ── InfiniteLine3D ────────────────────────────────────────────────────────────

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
) -> Vec<Point3D<T>> {
    let mut intersections = Vec::new();
    let point = line.point();
    let direction = line.direction();
    let start = Point3D::new(point.0, point.1, point.2);
    let direction = crate::Vector3D::new(direction.0, direction.1, direction.2);

    let Some((t1, t2)) =
        spherical_surface_intersection_parameters(&start, &direction, sphere, tolerance)
    else {
        return intersections;
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

    intersections
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

#[cfg(test)]
mod tests {
    use super::{
        arc3d_infinite_line3d_intersection, arc3d_line_segment3d_intersection,
        arc3d_point3d_intersection, arc3d_ray3d_intersection,
        circle3d_infinite_line3d_intersection, circle3d_line_segment3d_intersection,
        circle3d_point3d_intersection, circle3d_ray3d_intersection,
        cylindrical_surface3d_point3d_intersection, ellipse3d_point3d_intersection,
        infinite_line3d_line_segment3d_intersection, infinite_line3d_point3d_intersection,
        infinite_line3d_spherical_surface3d_intersections,
        line_segment3d_infinite_line3d_intersection, line_segment3d_plane3d_intersection,
        line_segment3d_point3d_intersection, line_segment3d_ray3d_intersection,
        line_segment3d_spherical_surface3d_intersections, line_segment3d_triangle3d_intersection,
        plane3d_line_segment3d_intersection, plane3d_point3d_intersection,
        plane3d_ray3d_intersection, ray3d_line_segment3d_intersection, ray3d_plane3d_intersection,
        ray3d_point3d_intersection, ray3d_ray3d_intersection,
        ray3d_spherical_surface3d_intersections, ray3d_triangle3d_intersection,
        torus_surface3d_point3d_intersection, triangle3d_line_segment3d_intersection,
        triangle3d_point3d_intersection, triangle3d_ray3d_intersection,
        triangle_mesh3d_point3d_intersection,
    };
    use crate::{
        Angle, Arc3D, Circle3D, CylindricalSurface3D, Direction3D, Ellipse3D, InfiniteLine3D,
        IntersectionGeometry, IntersectionTopology, LineSegment3D, Plane3D, Point3D, Ray3D,
        SphericalSurface3D, TorusSurface3D, Triangle3D, TriangleMesh3D, Vector3D,
    };
    use geo_contracts::ToleranceSettings;

    fn standard_distance_tol() -> f64 {
        ToleranceSettings::<f64>::standard().distance_tolerance
    }

    #[test]
    fn plane_point_intersection_returns_same_point() {
        let plane = Plane3D::xy_plane(0.0_f64);
        let point = Point3D::new(1.0, -2.0, 0.0);

        let result = plane3d_point3d_intersection(&plane, &point, standard_distance_tol());

        assert_eq!(result.topology, IntersectionTopology::Crossing);
        assert!(matches!(result.geometry, IntersectionGeometry::Point(p) if p == point));
    }

    #[test]
    fn ray_point_intersection_respects_ray_direction() {
        let ray = Ray3D::new(Point3D::new(0.0, 0.0, 0.0), Vector3D::new(1.0, 0.0, 0.0)).unwrap();
        let on_ray = Point3D::new(2.0, 0.0, 0.0);
        let behind_ray = Point3D::new(-1.0, 0.0, 0.0);

        let on = ray3d_point3d_intersection(&ray, &on_ray, standard_distance_tol());
        let behind = ray3d_point3d_intersection(&ray, &behind_ray, standard_distance_tol());
        assert_eq!(on.topology, IntersectionTopology::Crossing);
        assert!(matches!(on.geometry, IntersectionGeometry::Point(p) if p == on_ray));
        assert_eq!(behind.topology, IntersectionTopology::Disjoint);
    }

    #[test]
    fn line_segment_point_intersection_checks_segment_bounds() {
        let segment =
            LineSegment3D::new(Point3D::new(0.0, 0.0, 0.0), Point3D::new(2.0, 0.0, 0.0)).unwrap();
        let on_segment = Point3D::new(1.0, 0.0, 0.0);
        let outside_segment = Point3D::new(3.0, 0.0, 0.0);

        let on =
            line_segment3d_point3d_intersection(&segment, &on_segment, standard_distance_tol());
        let outside = line_segment3d_point3d_intersection(
            &segment,
            &outside_segment,
            standard_distance_tol(),
        );
        assert_eq!(on.topology, IntersectionTopology::Crossing);
        assert!(matches!(on.geometry, IntersectionGeometry::Point(p) if p == on_segment));
        assert_eq!(outside.topology, IntersectionTopology::Disjoint);
    }

    #[test]
    fn infinite_line_point_intersection_checks_collinearity() {
        let line = InfiniteLine3D::from_two_points(
            Point3D::new(0.0, 0.0, 0.0),
            Point3D::new(1.0, 0.0, 0.0),
        )
        .unwrap();
        let on_line = Point3D::new(5.0, 0.0, 0.0);
        let off_line = Point3D::new(0.0, 1.0, 0.0);

        let on = infinite_line3d_point3d_intersection(&line, &on_line, standard_distance_tol());
        let off = infinite_line3d_point3d_intersection(&line, &off_line, standard_distance_tol());
        assert_eq!(on.topology, IntersectionTopology::Crossing);
        assert!(matches!(on.geometry, IntersectionGeometry::Point(p) if p == on_line));
        assert_eq!(off.topology, IntersectionTopology::Disjoint);
    }

    #[test]
    fn triangle_point_intersection_uses_distance_based_test() {
        let triangle = Triangle3D::new(
            Point3D::new(0.0, 0.0, 0.0),
            Point3D::new(1.0, 0.0, 0.0),
            Point3D::new(0.0, 1.0, 0.0),
        )
        .unwrap();
        let on_triangle = Point3D::new(0.2, 0.2, 0.0);
        let off_triangle = Point3D::new(0.2, 0.2, 0.5);

        let on = triangle3d_point3d_intersection(&triangle, &on_triangle, standard_distance_tol());
        let off =
            triangle3d_point3d_intersection(&triangle, &off_triangle, standard_distance_tol());
        assert_eq!(on.topology, IntersectionTopology::Crossing);
        assert!(matches!(on.geometry, IntersectionGeometry::Point(p) if p == on_triangle));
        assert_eq!(off.topology, IntersectionTopology::Disjoint);
    }

    #[test]
    fn circle_point_intersection_requires_point_on_circumference() {
        let circle = Circle3D::new(
            Point3D::new(0.0, 0.0, 0.0),
            Direction3D::new(0.0, 0.0, 1.0).unwrap(),
            2.0,
        )
        .unwrap();
        let on_circle = Point3D::new(2.0, 0.0, 0.0);
        let inside_disk = Point3D::new(1.0, 0.0, 0.0);

        assert_eq!(
            circle3d_point3d_intersection(&circle, &on_circle, standard_distance_tol()).topology,
            IntersectionTopology::Crossing
        );
        assert!(matches!(
            circle3d_point3d_intersection(&circle, &on_circle, standard_distance_tol()).geometry,
            IntersectionGeometry::Point(point) if point == on_circle
        ));
        assert_eq!(
            circle3d_point3d_intersection(&circle, &inside_disk, standard_distance_tol()).topology,
            IntersectionTopology::Disjoint
        );
        assert!(matches!(
            circle3d_point3d_intersection(&circle, &inside_disk, standard_distance_tol()).geometry,
            IntersectionGeometry::None
        ));
    }

    #[test]
    fn circle_point_intersection_result_converts_to_topology() {
        let circle = Circle3D::new(
            Point3D::new(0.0, 0.0, 0.0),
            Direction3D::new(0.0, 0.0, 1.0).unwrap(),
            2.0,
        )
        .unwrap();
        let on_circle = Point3D::new(2.0, 0.0, 0.0);
        let off_circle = Point3D::new(1.0, 0.0, 0.0);

        let hit = circle3d_point3d_intersection(&circle, &on_circle, standard_distance_tol());
        let miss = circle3d_point3d_intersection(&circle, &off_circle, standard_distance_tol());

        assert_eq!(hit.topology, IntersectionTopology::Crossing);
        assert!(matches!(hit.geometry, IntersectionGeometry::Point(_)));
        assert_eq!(miss.topology, IntersectionTopology::Disjoint);
        assert!(matches!(miss.geometry, IntersectionGeometry::None));
    }

    #[test]
    fn arc_point_intersection_checks_angle_range() {
        let arc = Arc3D::new(
            Point3D::new(0.0, 0.0, 0.0),
            2.0,
            Direction3D::new(0.0, 0.0, 1.0).unwrap(),
            Direction3D::new(1.0, 0.0, 0.0).unwrap(),
            Angle::from_radians(0.0),
            Angle::from_radians(std::f64::consts::FRAC_PI_2),
        )
        .unwrap();

        let on_arc = Point3D::new(2.0, 0.0, 0.0);
        let out_of_angle = Point3D::new(-2.0, 0.0, 0.0);

        assert_eq!(
            arc3d_point3d_intersection(&arc, &on_arc, standard_distance_tol()).topology,
            IntersectionTopology::Crossing
        );
        assert!(matches!(
            arc3d_point3d_intersection(&arc, &on_arc, standard_distance_tol()).geometry,
            IntersectionGeometry::Point(point) if point == on_arc
        ));
        assert_eq!(
            arc3d_point3d_intersection(&arc, &out_of_angle, standard_distance_tol()).topology,
            IntersectionTopology::Disjoint
        );
        assert!(matches!(
            arc3d_point3d_intersection(&arc, &out_of_angle, standard_distance_tol()).geometry,
            IntersectionGeometry::None
        ));
    }

    #[test]
    fn arc_point_intersection_result_converts_to_topology() {
        let arc = Arc3D::new(
            Point3D::new(0.0, 0.0, 0.0),
            2.0,
            Direction3D::new(0.0, 0.0, 1.0).unwrap(),
            Direction3D::new(1.0, 0.0, 0.0).unwrap(),
            Angle::from_radians(0.0),
            Angle::from_radians(std::f64::consts::FRAC_PI_2),
        )
        .unwrap();

        let on_arc = Point3D::new(2.0, 0.0, 0.0);
        let off_arc = Point3D::new(-2.0, 0.0, 0.0);
        let hit = arc3d_point3d_intersection(&arc, &on_arc, standard_distance_tol());
        let miss = arc3d_point3d_intersection(&arc, &off_arc, standard_distance_tol());

        assert_eq!(hit.topology, IntersectionTopology::Crossing);
        assert!(matches!(hit.geometry, IntersectionGeometry::Point(_)));
        assert_eq!(miss.topology, IntersectionTopology::Disjoint);
        assert!(matches!(miss.geometry, IntersectionGeometry::None));
    }

    #[test]
    fn arc_linear_input_result_variants_convert_to_topology() {
        let arc = Arc3D::new(
            Point3D::new(0.0, 0.0, 0.0),
            2.0,
            Direction3D::new(0.0, 0.0, 1.0).unwrap(),
            Direction3D::new(1.0, 0.0, 0.0).unwrap(),
            Angle::from_radians(0.0),
            Angle::from_radians(std::f64::consts::FRAC_PI_2),
        )
        .unwrap();

        let segment_hit =
            LineSegment3D::new(Point3D::new(2.0, 0.0, 0.0), Point3D::new(3.0, 0.0, 0.0)).unwrap();
        let segment_miss =
            LineSegment3D::new(Point3D::new(5.0, 0.0, 0.0), Point3D::new(6.0, 0.0, 0.0)).unwrap();
        let ray_hit =
            Ray3D::new(Point3D::new(2.0, 0.0, 0.0), Vector3D::new(1.0, 0.0, 0.0)).unwrap();
        let ray_miss =
            Ray3D::new(Point3D::new(5.0, 0.0, 0.0), Vector3D::new(1.0, 0.0, 0.0)).unwrap();
        let line_hit = InfiniteLine3D::from_two_points(
            Point3D::new(2.0, 0.0, 0.0),
            Point3D::new(2.0, 1.0, 0.0),
        )
        .unwrap();
        let line_miss = InfiniteLine3D::from_two_points(
            Point3D::new(5.0, 0.0, 0.0),
            Point3D::new(5.0, 1.0, 0.0),
        )
        .unwrap();

        let hit_segment =
            arc3d_line_segment3d_intersection(&arc, &segment_hit, standard_distance_tol());
        let miss_segment =
            arc3d_line_segment3d_intersection(&arc, &segment_miss, standard_distance_tol());
        let hit_ray = arc3d_ray3d_intersection(&arc, &ray_hit, standard_distance_tol());
        let miss_ray = arc3d_ray3d_intersection(&arc, &ray_miss, standard_distance_tol());
        let hit_line = arc3d_infinite_line3d_intersection(&arc, &line_hit, standard_distance_tol());
        let miss_line =
            arc3d_infinite_line3d_intersection(&arc, &line_miss, standard_distance_tol());

        assert_eq!(hit_segment.topology, IntersectionTopology::Crossing);
        assert!(matches!(
            hit_segment.geometry,
            IntersectionGeometry::Point(_)
        ));
        assert_eq!(miss_segment.topology, IntersectionTopology::Disjoint);
        assert_eq!(hit_ray.topology, IntersectionTopology::Crossing);
        assert_eq!(miss_ray.topology, IntersectionTopology::Disjoint);
        assert_eq!(hit_line.topology, IntersectionTopology::Crossing);
        assert_eq!(miss_line.topology, IntersectionTopology::Disjoint);
    }

    #[test]
    fn ellipse_point_intersection_uses_distance() {
        let ellipse = Ellipse3D::new(
            Point3D::new(0.0, 0.0, 0.0),
            3.0,
            2.0,
            Vector3D::new(0.0, 0.0, 1.0),
            Vector3D::new(1.0, 0.0, 0.0),
        )
        .unwrap();

        let on_ellipse = Point3D::new(3.0, 0.0, 0.0);
        let inside_ellipse = Point3D::new(1.0, 0.0, 0.0);
        let outside_plane = Point3D::new(0.0, 0.0, 0.5);

        let on = ellipse3d_point3d_intersection(&ellipse, &on_ellipse, standard_distance_tol());
        let inside =
            ellipse3d_point3d_intersection(&ellipse, &inside_ellipse, standard_distance_tol());
        let outside =
            ellipse3d_point3d_intersection(&ellipse, &outside_plane, standard_distance_tol());
        assert_eq!(on.topology, IntersectionTopology::Crossing);
        assert!(matches!(on.geometry, IntersectionGeometry::Point(p) if p == on_ellipse));
        assert_eq!(inside.topology, IntersectionTopology::Crossing);
        assert!(matches!(inside.geometry, IntersectionGeometry::Point(p) if p == inside_ellipse));
        assert_eq!(outside.topology, IntersectionTopology::Disjoint);
    }

    #[test]
    fn circle_linear_input_result_variants_convert_to_topology() {
        let circle = Circle3D::new(
            Point3D::new(0.0, 0.0, 0.0),
            Direction3D::new(0.0, 0.0, 1.0).unwrap(),
            2.0,
        )
        .unwrap();

        let segment_hit =
            LineSegment3D::new(Point3D::new(2.0, 0.0, 0.0), Point3D::new(3.0, 0.0, 0.0)).unwrap();
        let segment_miss =
            LineSegment3D::new(Point3D::new(5.0, 0.0, 0.0), Point3D::new(6.0, 0.0, 0.0)).unwrap();
        let ray_hit =
            Ray3D::new(Point3D::new(2.0, 0.0, 0.0), Vector3D::new(1.0, 0.0, 0.0)).unwrap();
        let ray_miss =
            Ray3D::new(Point3D::new(5.0, 0.0, 0.0), Vector3D::new(1.0, 0.0, 0.0)).unwrap();
        let line_hit = InfiniteLine3D::from_two_points(
            Point3D::new(2.0, 0.0, 0.0),
            Point3D::new(2.0, 1.0, 0.0),
        )
        .unwrap();
        let line_miss = InfiniteLine3D::from_two_points(
            Point3D::new(5.0, 0.0, 0.0),
            Point3D::new(5.0, 1.0, 0.0),
        )
        .unwrap();

        let hit_segment =
            circle3d_line_segment3d_intersection(&circle, &segment_hit, standard_distance_tol());
        let miss_segment =
            circle3d_line_segment3d_intersection(&circle, &segment_miss, standard_distance_tol());
        let hit_ray = circle3d_ray3d_intersection(&circle, &ray_hit, standard_distance_tol());
        let miss_ray = circle3d_ray3d_intersection(&circle, &ray_miss, standard_distance_tol());
        let hit_line =
            circle3d_infinite_line3d_intersection(&circle, &line_hit, standard_distance_tol());
        let miss_line =
            circle3d_infinite_line3d_intersection(&circle, &line_miss, standard_distance_tol());

        assert_eq!(hit_segment.topology, IntersectionTopology::Crossing);
        assert!(matches!(
            hit_segment.geometry,
            IntersectionGeometry::Point(_)
        ));
        assert_eq!(miss_segment.topology, IntersectionTopology::Disjoint);
        assert_eq!(hit_ray.topology, IntersectionTopology::Crossing);
        assert_eq!(miss_ray.topology, IntersectionTopology::Disjoint);
        assert_eq!(hit_line.topology, IntersectionTopology::Crossing);
        assert_eq!(miss_line.topology, IntersectionTopology::Disjoint);
    }

    #[test]
    fn torus_surface_point_intersection_uses_surface_distance() {
        let torus = TorusSurface3D::new(
            Point3D::new(0.0, 0.0, 0.0),
            Direction3D::new(0.0, 0.0, 1.0).unwrap(),
            Direction3D::new(1.0, 0.0, 0.0).unwrap(),
            3.0,
            1.0,
        )
        .unwrap();

        let on_surface = Point3D::new(4.0, 0.0, 0.0);
        let inside_tube = Point3D::new(3.0, 0.0, 0.0);

        let on = torus_surface3d_point3d_intersection(&torus, &on_surface, standard_distance_tol());
        let inside =
            torus_surface3d_point3d_intersection(&torus, &inside_tube, standard_distance_tol());
        assert_eq!(on.topology, IntersectionTopology::Crossing);
        assert!(matches!(on.geometry, IntersectionGeometry::Point(p) if p == on_surface));
        assert_eq!(inside.topology, IntersectionTopology::Disjoint);
    }

    #[test]
    fn cylindrical_surface_point_intersection_uses_surface_distance() {
        let cyl = CylindricalSurface3D::new_z_axis(Point3D::new(0.0, 0.0, 0.0), 1.0).unwrap();

        let on_surface = Point3D::new(1.0, 0.0, 0.5);
        let off_surface = Point3D::new(2.0, 0.0, 0.5);

        let on =
            cylindrical_surface3d_point3d_intersection(&cyl, &on_surface, standard_distance_tol());
        let off =
            cylindrical_surface3d_point3d_intersection(&cyl, &off_surface, standard_distance_tol());
        assert_eq!(on.topology, IntersectionTopology::Crossing);
        assert!(matches!(on.geometry, IntersectionGeometry::Point(p) if p == on_surface));
        assert_eq!(off.topology, IntersectionTopology::Disjoint);
    }

    #[test]
    fn triangle_mesh_point_intersection_checks_member_triangles() {
        let mesh = TriangleMesh3D::new(
            vec![
                Point3D::new(0.0, 0.0, 0.0),
                Point3D::new(1.0, 0.0, 0.0),
                Point3D::new(0.0, 1.0, 0.0),
            ],
            vec![[0, 1, 2]],
        )
        .unwrap();

        let on_triangle = Point3D::new(0.2, 0.2, 0.0);
        let off_triangle = Point3D::new(0.2, 0.2, 0.3);

        let on = triangle_mesh3d_point3d_intersection(&mesh, &on_triangle, standard_distance_tol());
        let off =
            triangle_mesh3d_point3d_intersection(&mesh, &off_triangle, standard_distance_tol());
        assert_eq!(on.topology, IntersectionTopology::Crossing);
        assert!(matches!(on.geometry, IntersectionGeometry::Point(p) if p == on_triangle));
        assert_eq!(off.topology, IntersectionTopology::Disjoint);
    }

    #[test]
    fn symmetric_triangle_intersection_wrappers_match_base_functions() {
        let tri = Triangle3D::new(
            Point3D::new(0.0, 0.0, 0.0),
            Point3D::new(1.0, 0.0, 0.0),
            Point3D::new(0.0, 1.0, 0.0),
        )
        .unwrap();
        let seg =
            LineSegment3D::new(Point3D::new(0.2, 0.2, -1.0), Point3D::new(0.2, 0.2, 1.0)).unwrap();
        let ray = Ray3D::new(Point3D::new(0.2, 0.2, 1.0), Vector3D::new(0.0, 0.0, -1.0)).unwrap();

        let tol = standard_distance_tol();
        assert_eq!(
            line_segment3d_triangle3d_intersection(&seg, &tri, tol).topology,
            triangle3d_line_segment3d_intersection(&tri, &seg, tol).topology
        );
        assert_eq!(
            ray3d_triangle3d_intersection(&ray, &tri, tol).topology,
            triangle3d_ray3d_intersection(&tri, &ray, tol).topology
        );
    }

    #[test]
    fn symmetric_plane_intersection_wrappers_match_base_functions() {
        let tol = standard_distance_tol();
        let plane = Plane3D::xy_plane(0.0_f64);
        let ray = Ray3D::new(Point3D::new(0.0, 0.0, 1.0), Vector3D::new(0.0, 0.0, -1.0)).unwrap();
        let seg =
            LineSegment3D::new(Point3D::new(0.0, 0.0, -1.0), Point3D::new(0.0, 0.0, 1.0)).unwrap();

        assert_eq!(
            ray3d_plane3d_intersection(&ray, &plane, tol).topology,
            plane3d_ray3d_intersection(&plane, &ray, tol).topology
        );
        assert_eq!(
            line_segment3d_plane3d_intersection(&seg, &plane, tol).topology,
            plane3d_line_segment3d_intersection(&plane, &seg, tol).topology
        );
    }

    #[test]
    fn symmetric_line_segment_intersection_wrappers_match_base_functions() {
        let tol = standard_distance_tol();
        let seg =
            LineSegment3D::new(Point3D::new(0.0, 0.0, 0.0), Point3D::new(1.0, 0.0, 0.0)).unwrap();
        let ray = Ray3D::new(Point3D::new(0.0, 0.0, 0.0), Vector3D::new(1.0, 0.0, 0.0)).unwrap();
        let line = InfiniteLine3D::from_two_points(
            Point3D::new(0.0, 0.0, 0.0),
            Point3D::new(1.0, 0.0, 0.0),
        )
        .unwrap();

        assert_eq!(
            line_segment3d_ray3d_intersection(&seg, &ray, tol).topology,
            ray3d_line_segment3d_intersection(&ray, &seg, tol).topology
        );
        assert_eq!(
            line_segment3d_infinite_line3d_intersection(&seg, &line, tol).topology,
            infinite_line3d_line_segment3d_intersection(&line, &seg, tol).topology
        );
    }

    #[test]
    fn spherical_surface_line_like_intersections_return_expected_points() {
        let tolerance = 1e-9;
        let sphere = SphericalSurface3D::new(
            Point3D::new(0.0, 0.0, 0.0),
            Vector3D::new(0.0, 0.0, 1.0),
            Vector3D::new(1.0, 0.0, 0.0),
            1.0,
        )
        .unwrap();

        let segment =
            LineSegment3D::new(Point3D::new(-2.0, 0.0, 0.0), Point3D::new(2.0, 0.0, 0.0)).unwrap();
        let ray = Ray3D::new(Point3D::new(-2.0, 0.0, 0.0), Vector3D::new(1.0, 0.0, 0.0)).unwrap();
        let line = InfiniteLine3D::from_two_points(
            Point3D::new(-2.0, 0.0, 0.0),
            Point3D::new(2.0, 0.0, 0.0),
        )
        .unwrap();

        assert_eq!(
            line_segment3d_spherical_surface3d_intersections(&segment, &sphere, tolerance),
            vec![Point3D::new(-1.0, 0.0, 0.0), Point3D::new(1.0, 0.0, 0.0)]
        );
        assert_eq!(
            ray3d_spherical_surface3d_intersections(&ray, &sphere, tolerance),
            vec![Point3D::new(-1.0, 0.0, 0.0), Point3D::new(1.0, 0.0, 0.0)]
        );
        assert_eq!(
            infinite_line3d_spherical_surface3d_intersections(&line, &sphere, tolerance),
            vec![Point3D::new(-1.0, 0.0, 0.0), Point3D::new(1.0, 0.0, 0.0)]
        );
    }

    #[test]
    fn ray_ray_intersection_returns_shared_point_only_for_forward_rays() {
        let tolerance = 1e-9;
        let ray_a = Ray3D::new(Point3D::new(-1.0, 0.0, 0.0), Vector3D::new(1.0, 0.0, 0.0)).unwrap();
        let ray_b = Ray3D::new(Point3D::new(0.0, -1.0, 0.0), Vector3D::new(0.0, 1.0, 0.0)).unwrap();
        let opposite =
            Ray3D::new(Point3D::new(0.0, -1.0, 0.0), Vector3D::new(0.0, -1.0, 0.0)).unwrap();

        let crossing = ray3d_ray3d_intersection(&ray_a, &ray_b, tolerance);
        let disjoint = ray3d_ray3d_intersection(&ray_a, &opposite, tolerance);
        assert_eq!(crossing.topology, IntersectionTopology::Crossing);
        let origin = Point3D::new(0.0_f64, 0.0, 0.0);
        if let IntersectionGeometry::Point(ref p) = crossing.geometry {
            assert!(p.distance_to(&origin) < 1e-9);
        } else {
            panic!("Expected IntersectionGeometry::Point for crossing");
        }
        assert_eq!(disjoint.topology, IntersectionTopology::Disjoint);
    }
}
