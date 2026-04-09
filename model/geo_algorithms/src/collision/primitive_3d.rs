//! 3D Primitive collision algorithms
//!
//! Phase C Step 2: `geo_primitives` から 3D 衝突判定ロジックを
//! `geo_algorithms` 側へ集約するための受け皿。
//!
//! orphan rules により trait 実装ではなく形状ペア free-function を提供する。
//! 各関数は既存の `BasicCollision` 実装を呼び出す薄いラッパー。

use crate::{
    Arc3D, Circle3D, CylindricalSolid3D, CylindricalSurface3D, Ellipse3D, EllipsoidalSolid3D,
    EllipsoidalSurface3D, InfiniteLine3D, LineSegment3D, Plane3D, Point3D, Ray3D, SphericalSolid3D,
    SphericalSurface3D, TorusSolid3D, TorusSurface3D, Triangle3D, TriangleMesh3D,
};
use geo_contracts::{
    Arc3DEndpoint, Arc3DProperties, Circle3DProperties, CylindricalSolid3DProperties,
    CylindricalSurface3DProperties, EllipsoidalSolid3DProperties, InfiniteLine3DProperties, Scalar,
    SphericalSolid3DProperties, SphericalSurface3DProperties, Triangle3DBoundaryAccess,
};

// ── SphericalSolid3D ──────────────────────────────────────────────────────────

pub fn spherical_solid3d_point3d_collides<T: Scalar>(
    sphere: &SphericalSolid3D<T>,
    point: &Point3D<T>,
    tolerance: T,
) -> bool {
    sphere.distance_to_surface(Point3D::new(point.x(), point.y(), point.z())) <= tolerance
}

pub fn spherical_solid3d_circle3d_collides<T: Scalar>(
    sphere: &SphericalSolid3D<T>,
    circle: &Circle3D<T>,
    tolerance: T,
) -> bool {
    let (cx, cy, cz) = circle.center();
    sphere.distance_to_surface(Point3D::new(cx, cy, cz)) <= circle.radius() + tolerance
}

pub fn spherical_solid3d_line_segment3d_collides<T: Scalar>(
    sphere: &SphericalSolid3D<T>,
    segment: &LineSegment3D<T>,
    tolerance: T,
) -> bool {
    sphere.distance_to_line_segment(&segment.start(), &segment.end()) <= tolerance
}

pub fn spherical_solid3d_ray3d_collides<T: Scalar>(
    sphere: &SphericalSolid3D<T>,
    ray: &Ray3D<T>,
    tolerance: T,
) -> bool {
    sphere.distance_to_ray(&ray.origin(), &ray.direction_vector()) <= tolerance
}

pub fn spherical_solid3d_infinite_line3d_collides<T: Scalar>(
    sphere: &SphericalSolid3D<T>,
    line: &InfiniteLine3D<T>,
    tolerance: T,
) -> bool {
    let (px, py, pz) = line.point();
    let line_pt = Point3D::new(px, py, pz);
    let (dx, dy, dz) = line.direction();
    let line_dir = crate::Vector3D::new(dx, dy, dz);
    sphere.distance_to_infinite_line(&line_pt, &line_dir) <= tolerance
}

pub fn spherical_solid3d_triangle3d_collides<T: Scalar>(
    sphere: &SphericalSolid3D<T>,
    triangle: &Triangle3D<T>,
    tolerance: T,
) -> bool {
    let (ax, ay, az) = Triangle3DBoundaryAccess::vertex_a(triangle);
    let (bx, by, bz) = Triangle3DBoundaryAccess::vertex_b(triangle);
    let (cx, cy, cz) = Triangle3DBoundaryAccess::vertex_c(triangle);
    sphere.distance_to_surface(Point3D::new(ax, ay, az)) <= tolerance
        || sphere.distance_to_surface(Point3D::new(bx, by, bz)) <= tolerance
        || sphere.distance_to_surface(Point3D::new(cx, cy, cz)) <= tolerance
}

pub fn spherical_solid3d_plane3d_collides<T: Scalar>(
    sphere: &SphericalSolid3D<T>,
    plane: &Plane3D<T>,
    tolerance: T,
) -> bool {
    let (sc, sy, sz) = SphericalSolid3DProperties::center(sphere);
    let center = Point3D::new(sc, sy, sz);
    let radius = SphericalSolid3DProperties::radius(sphere);
    plane.distance_to_point(center).abs() <= radius + tolerance
}

pub fn spherical_solid3d_spherical_solid3d_collides<T: Scalar>(
    sphere_a: &SphericalSolid3D<T>,
    sphere_b: &SphericalSolid3D<T>,
    tolerance: T,
) -> bool {
    let (ax, ay, az) = SphericalSolid3DProperties::center(sphere_a);
    let (bx, by, bz) = SphericalSolid3DProperties::center(sphere_b);
    let center_a = Point3D::new(ax, ay, az);
    let center_b = Point3D::new(bx, by, bz);
    let dist = center_a.distance_to(&center_b);
    let r1 = SphericalSolid3DProperties::radius(sphere_a);
    let r2 = SphericalSolid3DProperties::radius(sphere_b);
    dist <= r1 + r2 + tolerance
}

// ── CylindricalSolid3D ────────────────────────────────────────────────────────

pub fn cylindrical_solid3d_point3d_collides<T: Scalar>(
    cyl: &CylindricalSolid3D<T>,
    point: &Point3D<T>,
    tolerance: T,
) -> bool {
    crate::distance::cylindrical_solid3d_point3d_distance(cyl, point) <= tolerance
}

pub fn cylindrical_solid3d_circle3d_collides<T: Scalar>(
    cyl: &CylindricalSolid3D<T>,
    circle: &Circle3D<T>,
    tolerance: T,
) -> bool {
    let (cx, cy, cz) = circle.center();
    let center = Point3D::new(cx, cy, cz);
    crate::distance::cylindrical_solid3d_point3d_distance(cyl, &center) <= tolerance
}

pub fn cylindrical_solid3d_line_segment3d_collides<T: Scalar>(
    cyl: &CylindricalSolid3D<T>,
    segment: &LineSegment3D<T>,
    tolerance: T,
) -> bool {
    let s = segment.start();
    let e = segment.end();
    crate::distance::cylindrical_solid3d_point3d_distance(cyl, &s) <= tolerance
        || crate::distance::cylindrical_solid3d_point3d_distance(cyl, &e) <= tolerance
}

pub fn cylindrical_solid3d_infinite_line3d_collides<T: Scalar>(
    cyl: &CylindricalSolid3D<T>,
    line: &InfiniteLine3D<T>,
    tolerance: T,
) -> bool {
    let (px, py, pz) = line.point();
    let point = Point3D::new(px, py, pz);
    crate::distance::cylindrical_solid3d_point3d_distance(cyl, &point) <= tolerance
}

pub fn cylindrical_solid3d_ray3d_collides<T: Scalar>(
    cyl: &CylindricalSolid3D<T>,
    ray: &Ray3D<T>,
    tolerance: T,
) -> bool {
    let o = ray.origin();
    crate::distance::cylindrical_solid3d_point3d_distance(cyl, &o) <= tolerance
}

pub fn cylindrical_solid3d_triangle3d_collides<T: Scalar>(
    cyl: &CylindricalSolid3D<T>,
    triangle: &Triangle3D<T>,
    tolerance: T,
) -> bool {
    let (ax, ay, az) = triangle.vertex_a();
    let (bx, by, bz) = triangle.vertex_b();
    let (cx, cy, cz) = triangle.vertex_c();
    let point_a = Point3D::new(ax, ay, az);
    let point_b = Point3D::new(bx, by, bz);
    let point_c = Point3D::new(cx, cy, cz);
    crate::distance::cylindrical_solid3d_point3d_distance(cyl, &point_a) <= tolerance
        || crate::distance::cylindrical_solid3d_point3d_distance(cyl, &point_b) <= tolerance
        || crate::distance::cylindrical_solid3d_point3d_distance(cyl, &point_c) <= tolerance
}

pub fn cylindrical_solid3d_plane3d_collides<T: Scalar>(
    cyl: &CylindricalSolid3D<T>,
    plane: &Plane3D<T>,
    tolerance: T,
) -> bool {
    let (cx, cy, cz) = <CylindricalSolid3D<T> as CylindricalSolid3DProperties<T>>::center(cyl);
    let center = Point3D::new(cx, cy, cz);
    let dist_center = plane.distance_to_point(center).abs();
    let radius = <CylindricalSolid3D<T> as CylindricalSolid3DProperties<T>>::radius(cyl);
    let height = <CylindricalSolid3D<T> as CylindricalSolid3DProperties<T>>::height(cyl);
    dist_center <= tolerance + radius + height
}

pub fn cylindrical_solid3d_cylindrical_solid3d_collides<T: Scalar>(
    cyl_a: &CylindricalSolid3D<T>,
    cyl_b: &CylindricalSolid3D<T>,
    tolerance: T,
) -> bool {
    let (ax, ay, az) = <CylindricalSolid3D<T> as CylindricalSolid3DProperties<T>>::center(cyl_a);
    let (bx, by, bz) = <CylindricalSolid3D<T> as CylindricalSolid3DProperties<T>>::center(cyl_b);
    let center_a = Point3D::new(ax, ay, az);
    let center_b = Point3D::new(bx, by, bz);
    let dist = center_a.distance_to(&center_b);
    let max_a = <CylindricalSolid3D<T> as CylindricalSolid3DProperties<T>>::radius(cyl_a)
        + <CylindricalSolid3D<T> as CylindricalSolid3DProperties<T>>::height(cyl_a);
    let max_b = <CylindricalSolid3D<T> as CylindricalSolid3DProperties<T>>::radius(cyl_b)
        + <CylindricalSolid3D<T> as CylindricalSolid3DProperties<T>>::height(cyl_b);
    dist <= max_a + max_b + tolerance
}

// ── CylindricalSurface3D ──────────────────────────────────────────────────────

pub fn cylindrical_surface3d_point3d_collides<T: Scalar>(
    cyl: &CylindricalSurface3D<T>,
    point: &Point3D<T>,
    tolerance: T,
) -> bool {
    crate::distance::cylindrical_surface3d_point3d_distance(cyl, point) <= tolerance
}

pub fn cylindrical_surface3d_circle3d_collides<T: Scalar>(
    cyl: &CylindricalSurface3D<T>,
    circle: &Circle3D<T>,
    tolerance: T,
) -> bool {
    let (cx, cy, cz) = circle.center();
    let center = Point3D::new(cx, cy, cz);
    crate::distance::cylindrical_surface3d_point3d_distance(cyl, &center) <= tolerance
}

pub fn cylindrical_surface3d_line_segment3d_collides<T: Scalar>(
    cyl: &CylindricalSurface3D<T>,
    segment: &LineSegment3D<T>,
    tolerance: T,
) -> bool {
    let s = segment.start();
    let e = segment.end();
    crate::distance::cylindrical_surface3d_point3d_distance(cyl, &s) <= tolerance
        || crate::distance::cylindrical_surface3d_point3d_distance(cyl, &e) <= tolerance
}

pub fn cylindrical_surface3d_ray3d_collides<T: Scalar>(
    cyl: &CylindricalSurface3D<T>,
    ray: &Ray3D<T>,
    tolerance: T,
) -> bool {
    let o = ray.origin();
    crate::distance::cylindrical_surface3d_point3d_distance(cyl, &o) <= tolerance
}

pub fn cylindrical_surface3d_infinite_line3d_collides<T: Scalar>(
    cyl: &CylindricalSurface3D<T>,
    line: &InfiniteLine3D<T>,
    tolerance: T,
) -> bool {
    let (px, py, pz) = line.point();
    let point = Point3D::new(px, py, pz);
    crate::distance::cylindrical_surface3d_point3d_distance(cyl, &point) <= tolerance
}

pub fn cylindrical_surface3d_triangle3d_collides<T: Scalar>(
    cyl: &CylindricalSurface3D<T>,
    triangle: &Triangle3D<T>,
    tolerance: T,
) -> bool {
    let (ax, ay, az) = triangle.vertex_a();
    let (bx, by, bz) = triangle.vertex_b();
    let (cx, cy, cz) = triangle.vertex_c();
    let point_a = Point3D::new(ax, ay, az);
    let point_b = Point3D::new(bx, by, bz);
    let point_c = Point3D::new(cx, cy, cz);
    crate::distance::cylindrical_surface3d_point3d_distance(cyl, &point_a) <= tolerance
        || crate::distance::cylindrical_surface3d_point3d_distance(cyl, &point_b) <= tolerance
        || crate::distance::cylindrical_surface3d_point3d_distance(cyl, &point_c) <= tolerance
}

pub fn cylindrical_surface3d_plane3d_collides<T: Scalar>(
    cyl: &CylindricalSurface3D<T>,
    plane: &Plane3D<T>,
    tolerance: T,
) -> bool {
    let (cx, cy, cz) = <CylindricalSurface3D<T> as CylindricalSurface3DProperties<T>>::center(cyl);
    let center = Point3D::new(cx, cy, cz);
    let dist = plane.distance_to_point(center).abs();
    let radius = <CylindricalSurface3D<T> as CylindricalSurface3DProperties<T>>::radius(cyl);
    dist <= radius + tolerance
}

pub fn cylindrical_surface3d_cylindrical_surface3d_collides<T: Scalar>(
    cyl_a: &CylindricalSurface3D<T>,
    cyl_b: &CylindricalSurface3D<T>,
    tolerance: T,
) -> bool {
    let (ax, ay, az) =
        <CylindricalSurface3D<T> as CylindricalSurface3DProperties<T>>::center(cyl_a);
    let (bx, by, bz) =
        <CylindricalSurface3D<T> as CylindricalSurface3DProperties<T>>::center(cyl_b);
    let center_a = Point3D::new(ax, ay, az);
    let center_b = Point3D::new(bx, by, bz);
    let dist = center_a.distance_to(&center_b);
    let radius_sum = <CylindricalSurface3D<T> as CylindricalSurface3DProperties<T>>::radius(cyl_a)
        + <CylindricalSurface3D<T> as CylindricalSurface3DProperties<T>>::radius(cyl_b);
    dist <= radius_sum + tolerance
}

// ── Ellipse3D ─────────────────────────────────────────────────────────────────

pub fn ellipse3d_point3d_collides<T: Scalar + From<f64>>(
    ellipse: &Ellipse3D<T>,
    point: &Point3D<T>,
    tolerance: T,
) -> bool {
    crate::distance::ellipse3d_point3d_distance(ellipse, point) <= tolerance
}

pub fn ellipse3d_circle3d_collides<T: Scalar + From<f64>>(
    ellipse: &Ellipse3D<T>,
    circle: &Circle3D<T>,
    tolerance: T,
) -> bool {
    let (cx, cy, cz) = circle.center();
    let center = Point3D::new(cx, cy, cz);
    crate::distance::ellipse3d_point3d_distance(ellipse, &center) <= circle.radius() + tolerance
}

pub fn ellipse3d_arc3d_collides<T: Scalar + From<f64>>(
    ellipse: &Ellipse3D<T>,
    arc: &Arc3D<T>,
    tolerance: T,
) -> bool {
    let (cx, cy, cz) = Arc3DProperties::center(arc);
    let arc_radius = Arc3DProperties::radius(arc);
    let center = Point3D::new(cx, cy, cz);
    crate::distance::ellipse3d_point3d_distance(ellipse, &center) <= arc_radius + tolerance
}

pub fn ellipse3d_line_segment3d_collides<T: Scalar + From<f64>>(
    ellipse: &Ellipse3D<T>,
    segment: &LineSegment3D<T>,
    tolerance: T,
) -> bool {
    let s = segment.start();
    let e = segment.end();
    let dist_start = crate::distance::ellipse3d_point3d_distance(ellipse, &s);
    let dist_end = crate::distance::ellipse3d_point3d_distance(ellipse, &e);
    let two = T::from_f64(2.0);
    let mid = Point3D::new(
        (s.x() + e.x()) / two,
        (s.y() + e.y()) / two,
        (s.z() + e.z()) / two,
    );
    let dist_mid = crate::distance::ellipse3d_point3d_distance(ellipse, &mid);
    dist_start <= tolerance || dist_end <= tolerance || dist_mid <= tolerance
}

pub fn ellipse3d_infinite_line3d_collides<T: Scalar + From<f64>>(
    ellipse: &Ellipse3D<T>,
    line: &InfiniteLine3D<T>,
    tolerance: T,
) -> bool {
    let (px, py, pz) = line.point();
    let point = Point3D::new(px, py, pz);
    crate::distance::ellipse3d_point3d_distance(ellipse, &point) <= tolerance
}

pub fn ellipse3d_ray3d_collides<T: Scalar + From<f64>>(
    ellipse: &Ellipse3D<T>,
    ray: &Ray3D<T>,
    tolerance: T,
) -> bool {
    let o = ray.origin();
    crate::distance::ellipse3d_point3d_distance(ellipse, &o) <= tolerance
}

pub fn ellipse3d_plane3d_collides<T: Scalar>(
    ellipse: &Ellipse3D<T>,
    plane: &Plane3D<T>,
    tolerance: T,
) -> bool {
    let center = ellipse.center();
    plane.distance_to_point(center).abs() <= tolerance
}

pub fn ellipse3d_triangle3d_collides<T: Scalar + From<f64>>(
    ellipse: &Ellipse3D<T>,
    triangle: &Triangle3D<T>,
    tolerance: T,
) -> bool {
    let (ax, ay, az) = triangle.vertex_a();
    let (bx, by, bz) = triangle.vertex_b();
    let (cx, cy, cz) = triangle.vertex_c();
    let point_a = Point3D::new(ax, ay, az);
    let point_b = Point3D::new(bx, by, bz);
    let point_c = Point3D::new(cx, cy, cz);
    let dist_a = crate::distance::ellipse3d_point3d_distance(ellipse, &point_a);
    let dist_b = crate::distance::ellipse3d_point3d_distance(ellipse, &point_b);
    let dist_c = crate::distance::ellipse3d_point3d_distance(ellipse, &point_c);
    let three = T::from_f64(3.0);
    let centroid = Point3D::new(
        (ax + bx + cx) / three,
        (ay + by + cy) / three,
        (az + bz + cz) / three,
    );
    let dist_centroid = crate::distance::ellipse3d_point3d_distance(ellipse, &centroid);
    dist_a <= tolerance || dist_b <= tolerance || dist_c <= tolerance || dist_centroid <= tolerance
}

pub fn ellipse3d_ellipse3d_collides<T: Scalar>(
    ellipse_a: &Ellipse3D<T>,
    ellipse_b: &Ellipse3D<T>,
    tolerance: T,
) -> bool {
    let c1 = ellipse_a.center();
    let c2 = ellipse_b.center();
    let center_dist = c1.distance_to(&c2);
    let sum_semi_major = ellipse_a.semi_major_axis() + ellipse_b.semi_major_axis();
    center_dist <= sum_semi_major + tolerance
}

// ── EllipsoidalSolid3D ────────────────────────────────────────────────────────

pub fn ellipsoidal_solid3d_point3d_collides<T: Scalar>(
    ellipsoid: &EllipsoidalSolid3D<T>,
    point: &Point3D<T>,
    _tolerance: T,
) -> bool {
    ellipsoid.contains_point(point)
}

pub fn ellipsoidal_solid3d_line_segment3d_collides<T: Scalar>(
    ellipsoid: &EllipsoidalSolid3D<T>,
    segment: &LineSegment3D<T>,
    tolerance: T,
) -> bool {
    let start = segment.start();
    let end = segment.end();
    let d1 = if ellipsoid.contains_point(&start) {
        T::ZERO
    } else {
        ellipsoid.distance_to_surface(&start)
    };
    let d2 = if ellipsoid.contains_point(&end) {
        T::ZERO
    } else {
        ellipsoid.distance_to_surface(&end)
    };
    d1 <= tolerance || d2 <= tolerance
}

pub fn ellipsoidal_solid3d_ray3d_collides<T: Scalar>(
    ellipsoid: &EllipsoidalSolid3D<T>,
    ray: &Ray3D<T>,
    tolerance: T,
) -> bool {
    let origin = ray.origin();
    let d = if ellipsoid.contains_point(&origin) {
        T::ZERO
    } else {
        ellipsoid.distance_to_surface(&origin)
    };
    d <= tolerance
}

pub fn ellipsoidal_solid3d_infinite_line3d_collides<T: Scalar>(
    ellipsoid: &EllipsoidalSolid3D<T>,
    line: &InfiniteLine3D<T>,
    tolerance: T,
) -> bool {
    let (px, py, pz) = line.point();
    let pt = Point3D::new(px, py, pz);
    let d = if ellipsoid.contains_point(&pt) {
        T::ZERO
    } else {
        ellipsoid.distance_to_surface(&pt)
    };
    d <= tolerance
}

pub fn ellipsoidal_solid3d_plane3d_collides<T: Scalar>(
    ellipsoid: &EllipsoidalSolid3D<T>,
    plane: &Plane3D<T>,
    tolerance: T,
) -> bool {
    let (cx, cy, cz) =
        <EllipsoidalSolid3D<T> as EllipsoidalSolid3DProperties<T>>::center(ellipsoid);
    let center = Point3D::new(cx, cy, cz);
    let dist = plane.distance_to_point(center).abs();
    let a = <EllipsoidalSolid3D<T> as EllipsoidalSolid3DProperties<T>>::a_radius(ellipsoid);
    let b = <EllipsoidalSolid3D<T> as EllipsoidalSolid3DProperties<T>>::b_radius(ellipsoid);
    let c = <EllipsoidalSolid3D<T> as EllipsoidalSolid3DProperties<T>>::c_radius(ellipsoid);
    let max_radius = a.max(b).max(c);
    dist <= max_radius + tolerance
}

pub fn ellipsoidal_solid3d_ellipsoidal_solid3d_collides<T: Scalar>(
    ell_a: &EllipsoidalSolid3D<T>,
    ell_b: &EllipsoidalSolid3D<T>,
    tolerance: T,
) -> bool {
    let (ax, ay, az) = <EllipsoidalSolid3D<T> as EllipsoidalSolid3DProperties<T>>::center(ell_a);
    let (bx, by, bz) = <EllipsoidalSolid3D<T> as EllipsoidalSolid3DProperties<T>>::center(ell_b);
    let center_a = Point3D::new(ax, ay, az);
    let center_b = Point3D::new(bx, by, bz);
    let center_dist = center_a.distance_to(&center_b);
    let max_1 = {
        let a = <EllipsoidalSolid3D<T> as EllipsoidalSolid3DProperties<T>>::a_radius(ell_a);
        let b = <EllipsoidalSolid3D<T> as EllipsoidalSolid3DProperties<T>>::b_radius(ell_a);
        let c = <EllipsoidalSolid3D<T> as EllipsoidalSolid3DProperties<T>>::c_radius(ell_a);
        a.max(b).max(c)
    };
    let max_2 = {
        let a = <EllipsoidalSolid3D<T> as EllipsoidalSolid3DProperties<T>>::a_radius(ell_b);
        let b = <EllipsoidalSolid3D<T> as EllipsoidalSolid3DProperties<T>>::b_radius(ell_b);
        let c = <EllipsoidalSolid3D<T> as EllipsoidalSolid3DProperties<T>>::c_radius(ell_b);
        a.max(b).max(c)
    };
    center_dist <= max_1 + max_2 + tolerance
}

// ── EllipsoidalSurface3D ──────────────────────────────────────────────────────

pub fn ellipsoidal_surface3d_point3d_collides<T: Scalar>(
    ellipsoid: &EllipsoidalSurface3D<T>,
    point: &Point3D<T>,
    tolerance: T,
) -> bool {
    ellipsoid.contains_point(point, tolerance)
}

// ── TorusSolid3D ──────────────────────────────────────────────────────────────

pub fn torus_solid3d_point3d_collides<T: Scalar>(
    torus: &TorusSolid3D<T>,
    point: &Point3D<T>,
    _tolerance: T,
) -> bool {
    torus.contains_point(point)
}

// ── TorusSurface3D ────────────────────────────────────────────────────────────

pub fn torus_surface3d_point3d_collides<T: Scalar>(
    torus: &TorusSurface3D<T>,
    point: &Point3D<T>,
    tolerance: T,
) -> bool {
    crate::distance::torus_surface3d_point3d_distance(torus, point) <= tolerance
}

// ── Triangle3D ────────────────────────────────────────────────────────────────

pub fn triangle3d_point3d_collides<T: Scalar>(
    triangle: &Triangle3D<T>,
    point: &Point3D<T>,
    tolerance: T,
) -> bool {
    triangle.distance_to_point(point) <= tolerance
}

pub fn triangle3d_line_segment3d_collides<T: Scalar>(
    triangle: &Triangle3D<T>,
    segment: &LineSegment3D<T>,
    tolerance: T,
) -> bool {
    triangle.distance_to_point(&segment.start()) <= tolerance
        || triangle.distance_to_point(&segment.end()) <= tolerance
}

pub fn line_segment3d_triangle3d_collides<T: Scalar>(
    segment: &LineSegment3D<T>,
    triangle: &Triangle3D<T>,
    tolerance: T,
) -> bool {
    triangle3d_line_segment3d_collides(triangle, segment, tolerance)
}

pub fn triangle3d_ray3d_collides<T: Scalar>(
    triangle: &Triangle3D<T>,
    ray: &Ray3D<T>,
    tolerance: T,
) -> bool {
    triangle.distance_to_point(&ray.origin()) <= tolerance
}

pub fn ray3d_triangle3d_collides<T: Scalar>(
    ray: &Ray3D<T>,
    triangle: &Triangle3D<T>,
    tolerance: T,
) -> bool {
    triangle3d_ray3d_collides(triangle, ray, tolerance)
}

pub fn triangle3d_triangle3d_collides<T: Scalar>(
    triangle_a: &Triangle3D<T>,
    triangle_b: &Triangle3D<T>,
    tolerance: T,
) -> bool {
    let (ax, ay, az) = Triangle3DBoundaryAccess::vertex_a(triangle_a);
    let (bx, by, bz) = Triangle3DBoundaryAccess::vertex_b(triangle_a);
    let (cx, cy, cz) = Triangle3DBoundaryAccess::vertex_c(triangle_a);
    let (ax2, ay2, az2) = Triangle3DBoundaryAccess::vertex_a(triangle_b);
    let (bx2, by2, bz2) = Triangle3DBoundaryAccess::vertex_b(triangle_b);
    let (cx2, cy2, cz2) = Triangle3DBoundaryAccess::vertex_c(triangle_b);
    triangle_b.distance_to_point(&Point3D::new(ax, ay, az)) <= tolerance
        || triangle_b.distance_to_point(&Point3D::new(bx, by, bz)) <= tolerance
        || triangle_b.distance_to_point(&Point3D::new(cx, cy, cz)) <= tolerance
        || triangle_a.distance_to_point(&Point3D::new(ax2, ay2, az2)) <= tolerance
        || triangle_a.distance_to_point(&Point3D::new(bx2, by2, bz2)) <= tolerance
        || triangle_a.distance_to_point(&Point3D::new(cx2, cy2, cz2)) <= tolerance
}

// ── TriangleMesh3D ────────────────────────────────────────────────────────────

pub fn triangle_mesh3d_point3d_collides<T: Scalar>(
    mesh: &TriangleMesh3D<T>,
    point: &Point3D<T>,
    tolerance: T,
) -> bool {
    (0..mesh.triangle_count()).any(|i| {
        mesh.triangle(i)
            .map(|tri| tri.distance_to_point(point) <= tolerance)
            .unwrap_or(false)
    })
}

// ── Arc3D ─────────────────────────────────────────────────────────────────────

pub fn arc3d_point3d_collides<T: Scalar>(arc: &Arc3D<T>, point: &Point3D<T>, tolerance: T) -> bool {
    crate::distance::arc3d_point3d_distance(arc, point) <= tolerance
        && arc.contains_point_angle(Point3D::new(point.x(), point.y(), point.z()))
}

pub fn arc3d_line_segment3d_collides<T: Scalar>(
    arc: &Arc3D<T>,
    segment: &LineSegment3D<T>,
    tolerance: T,
) -> bool {
    let (sx, sy, sz) = <Arc3D<T> as Arc3DEndpoint<T>>::start_point(arc);
    let (ex, ey, ez) = <Arc3D<T> as Arc3DEndpoint<T>>::end_point(arc);
    let arc_start = Point3D::new(sx, sy, sz);
    let arc_end = Point3D::new(ex, ey, ez);
    crate::distance::arc3d_point3d_distance(arc, &segment.start()) <= tolerance
        || crate::distance::arc3d_point3d_distance(arc, &segment.end()) <= tolerance
        || {
            let d1 = crate::Vector3D::from_points(&arc_start, &segment.start()).magnitude();
            let d2 = crate::Vector3D::from_points(&arc_end, &segment.start()).magnitude();
            d1 <= tolerance || d2 <= tolerance
        }
}

pub fn arc3d_ray3d_collides<T: Scalar>(arc: &Arc3D<T>, ray: &Ray3D<T>, tolerance: T) -> bool {
    crate::distance::arc3d_point3d_distance(arc, &ray.origin()) <= tolerance
}

pub fn arc3d_infinite_line3d_collides<T: Scalar>(
    arc: &Arc3D<T>,
    line: &InfiniteLine3D<T>,
    tolerance: T,
) -> bool {
    let (px, py, pz) = line.point();
    crate::distance::arc3d_point3d_distance(arc, &Point3D::new(px, py, pz)) <= tolerance
}

pub fn arc3d_arc3d_collides<T: Scalar>(arc_a: &Arc3D<T>, arc_b: &Arc3D<T>, tolerance: T) -> bool {
    let (s1x, s1y, s1z) = <Arc3D<T> as Arc3DEndpoint<T>>::start_point(arc_a);
    let (e1x, e1y, e1z) = <Arc3D<T> as Arc3DEndpoint<T>>::end_point(arc_a);
    let (s2x, s2y, s2z) = <Arc3D<T> as Arc3DEndpoint<T>>::start_point(arc_b);
    let (e2x, e2y, e2z) = <Arc3D<T> as Arc3DEndpoint<T>>::end_point(arc_b);
    let pa = Point3D::new(s1x, s1y, s1z);
    let pb = Point3D::new(e1x, e1y, e1z);
    let pc = Point3D::new(s2x, s2y, s2z);
    let pd = Point3D::new(e2x, e2y, e2z);
    crate::Vector3D::from_points(&pa, &pc).magnitude() <= tolerance
        || crate::Vector3D::from_points(&pa, &pd).magnitude() <= tolerance
        || crate::Vector3D::from_points(&pb, &pc).magnitude() <= tolerance
        || crate::Vector3D::from_points(&pb, &pd).magnitude() <= tolerance
}

// ── Circle3D ──────────────────────────────────────────────────────────────────

pub fn circle3d_point3d_collides<T: Scalar>(
    circle: &Circle3D<T>,
    point: &Point3D<T>,
    tolerance: T,
) -> bool {
    circle.distance_to_point_3d(Point3D::new(point.x(), point.y(), point.z())) <= tolerance
}

pub fn circle3d_line_segment3d_collides<T: Scalar>(
    circle: &Circle3D<T>,
    segment: &LineSegment3D<T>,
    tolerance: T,
) -> bool {
    let dist_start = circle.distance_to_point_3d(segment.start());
    let dist_end = circle.distance_to_point_3d(segment.end());
    let mid_x = (segment.start().x() + segment.end().x()) / T::from_f64(2.0);
    let mid_y = (segment.start().y() + segment.end().y()) / T::from_f64(2.0);
    let mid_z = (segment.start().z() + segment.end().z()) / T::from_f64(2.0);
    dist_start <= tolerance
        || dist_end <= tolerance
        || circle.distance_to_point_3d(Point3D::new(mid_x, mid_y, mid_z)) <= tolerance
}

pub fn circle3d_ray3d_collides<T: Scalar>(
    circle: &Circle3D<T>,
    ray: &Ray3D<T>,
    tolerance: T,
) -> bool {
    circle.distance_to_point_3d(ray.origin()) <= tolerance
}

pub fn circle3d_infinite_line3d_collides<T: Scalar>(
    circle: &Circle3D<T>,
    line: &InfiniteLine3D<T>,
    tolerance: T,
) -> bool {
    let (px, py, pz) = line.point();
    circle.distance_to_point_3d(Point3D::new(px, py, pz)) <= tolerance
}

pub fn circle3d_circle3d_collides<T: Scalar>(
    circle_a: &Circle3D<T>,
    circle_b: &Circle3D<T>,
    tolerance: T,
) -> bool {
    let (ax, ay, az) = circle_a.center();
    let (bx, by, bz) = circle_b.center();
    let center_a = Point3D::new(ax, ay, az);
    let center_b = Point3D::new(bx, by, bz);
    let dist = center_a.distance_to(&center_b);
    dist <= circle_a.radius() + circle_b.radius() + tolerance
}

// ── Plane3D ───────────────────────────────────────────────────────────────────

pub fn plane3d_point3d_collides<T: Scalar>(
    plane: &Plane3D<T>,
    point: &Point3D<T>,
    tolerance: T,
) -> bool {
    plane.contains_point(Point3D::new(point.x(), point.y(), point.z()), tolerance)
}

pub fn plane3d_line_segment3d_collides<T: Scalar>(
    plane: &Plane3D<T>,
    segment: &LineSegment3D<T>,
    tolerance: T,
) -> bool {
    let start_dist = plane.distance_to_point(segment.start());
    let end_dist = plane.distance_to_point(segment.end());
    (start_dist * end_dist <= T::ZERO)
        || (start_dist.abs() <= tolerance)
        || (end_dist.abs() <= tolerance)
}

pub fn plane3d_ray3d_collides<T: Scalar>(plane: &Plane3D<T>, ray: &Ray3D<T>, tolerance: T) -> bool {
    let ray_dir = ray.direction_vector();
    let normal_vec = plane.normal().as_vector();
    let denom = ray_dir.dot(&normal_vec);
    if denom.abs() > tolerance {
        let origin_dist = plane.distance_to_point(ray.origin());
        origin_dist.abs() <= tolerance || (origin_dist * denom <= T::ZERO)
    } else {
        plane.contains_point(ray.origin(), tolerance)
    }
}

pub fn plane3d_infinite_line3d_collides<T: Scalar>(
    plane: &Plane3D<T>,
    line: &InfiniteLine3D<T>,
    tolerance: T,
) -> bool {
    let (dx, dy, dz) = InfiniteLine3DProperties::direction(line);
    let line_dir_vec = crate::Vector3D::new(dx, dy, dz);
    let normal_vec = plane.normal().as_vector();
    let denom = line_dir_vec.dot(&normal_vec);
    if denom.abs() > tolerance {
        true
    } else {
        let (px, py, pz) = InfiniteLine3DProperties::point(line);
        plane.contains_point(crate::Point3D::new(px, py, pz), tolerance)
    }
}

pub fn plane3d_plane3d_collides<T: Scalar>(
    plane_a: &Plane3D<T>,
    plane_b: &Plane3D<T>,
    tolerance: T,
) -> bool {
    let normal1 = plane_a.normal().as_vector();
    let normal2 = plane_b.normal().as_vector();
    let cross = normal1.cross(&normal2);
    if cross.length() > tolerance {
        true
    } else {
        let dist = plane_a.distance_to_point(plane_b.origin());
        dist.abs() <= tolerance
    }
}

// ── Ray3D ─────────────────────────────────────────────────────────────────────

pub fn ray3d_point3d_collides<T: Scalar>(ray: &Ray3D<T>, point: &Point3D<T>, tolerance: T) -> bool {
    ray.contains_point(point, tolerance)
}

pub fn ray3d_spherical_surface3d_collides<T: Scalar>(
    ray: &Ray3D<T>,
    sphere: &SphericalSurface3D<T>,
    tolerance: T,
) -> bool {
    let center_tuple = SphericalSurface3DProperties::center(sphere);
    let center = Point3D::new(center_tuple.0, center_tuple.1, center_tuple.2);
    ray.distance_to_point(&center) <= SphericalSurface3DProperties::radius(sphere) + tolerance
}

pub fn ray3d_ray3d_collides<T: Scalar>(ray_a: &Ray3D<T>, ray_b: &Ray3D<T>, tolerance: T) -> bool {
    ray_a.origin().distance_to(&ray_b.origin()) <= tolerance
}

pub fn ray3d_line_segment3d_collides<T: Scalar>(
    ray: &Ray3D<T>,
    segment: &LineSegment3D<T>,
    tolerance: T,
) -> bool {
    let d1 = segment.distance_to_point(&ray.origin());
    let d2 = ray.distance_to_point(&segment.start());
    let d3 = ray.distance_to_point(&segment.end());
    d1.min(d2).min(d3) <= tolerance
}

pub fn ray3d_infinite_line3d_collides<T: Scalar>(
    ray: &Ray3D<T>,
    line: &InfiniteLine3D<T>,
    tolerance: T,
) -> bool {
    line.distance_to_point(&ray.origin()) <= tolerance
}

pub fn ray3d_plane3d_collides<T: Scalar>(ray: &Ray3D<T>, plane: &Plane3D<T>, tolerance: T) -> bool {
    plane3d_ray3d_collides(plane, ray, tolerance)
}

// ── LineSegment3D ─────────────────────────────────────────────────────────────

pub fn line_segment3d_point3d_collides<T: Scalar>(
    segment: &LineSegment3D<T>,
    point: &Point3D<T>,
    tolerance: T,
) -> bool {
    segment.contains_point(point, tolerance)
}

pub fn line_segment3d_spherical_surface3d_collides<T: Scalar>(
    segment: &LineSegment3D<T>,
    sphere: &SphericalSurface3D<T>,
    tolerance: T,
) -> bool {
    let center_tuple = SphericalSurface3DProperties::center(sphere);
    let center = Point3D::new(center_tuple.0, center_tuple.1, center_tuple.2);
    let dist = segment.distance_to_point(&center);
    (dist - SphericalSurface3DProperties::radius(sphere)).max(T::ZERO) <= tolerance
}

pub fn line_segment3d_line_segment3d_collides<T: Scalar>(
    seg_a: &LineSegment3D<T>,
    seg_b: &LineSegment3D<T>,
    tolerance: T,
) -> bool {
    let d1 = seg_a.distance_to_point(&seg_b.start());
    let d2 = seg_a.distance_to_point(&seg_b.end());
    let d3 = seg_b.distance_to_point(&seg_a.start());
    let d4 = seg_b.distance_to_point(&seg_a.end());
    d1.min(d2).min(d3).min(d4) <= tolerance
}

pub fn line_segment3d_ray3d_collides<T: Scalar>(
    segment: &LineSegment3D<T>,
    ray: &Ray3D<T>,
    tolerance: T,
) -> bool {
    let d1 = ray.distance_to_point(&segment.start());
    let d2 = ray.distance_to_point(&segment.end());
    let d3 = segment.distance_to_point(&ray.origin());
    d1.min(d2).min(d3) <= tolerance
}

pub fn line_segment3d_infinite_line3d_collides<T: Scalar>(
    segment: &LineSegment3D<T>,
    line: &InfiniteLine3D<T>,
    tolerance: T,
) -> bool {
    let d1 = line.distance_to_point(&segment.start());
    let d2 = line.distance_to_point(&segment.end());
    d1.min(d2) <= tolerance
}

pub fn line_segment3d_plane3d_collides<T: Scalar>(
    segment: &LineSegment3D<T>,
    plane: &Plane3D<T>,
    tolerance: T,
) -> bool {
    plane3d_line_segment3d_collides(plane, segment, tolerance)
}

// ── InfiniteLine3D ────────────────────────────────────────────────────────────

pub fn infinite_line3d_point3d_collides<T: Scalar>(
    line: &InfiniteLine3D<T>,
    point: &Point3D<T>,
    tolerance: T,
) -> bool {
    line.contains_point(point, tolerance)
}

pub fn infinite_line3d_spherical_surface3d_collides<T: Scalar>(
    line: &InfiniteLine3D<T>,
    sphere: &SphericalSurface3D<T>,
    tolerance: T,
) -> bool {
    let center_tuple = SphericalSurface3DProperties::center(sphere);
    let center = Point3D::new(center_tuple.0, center_tuple.1, center_tuple.2);
    line.distance_to_point(&center) <= SphericalSurface3DProperties::radius(sphere) + tolerance
}

pub fn infinite_line3d_infinite_line3d_collides<T: Scalar>(
    line_a: &InfiniteLine3D<T>,
    line_b: &InfiniteLine3D<T>,
    _tolerance: T,
) -> bool {
    !line_a.is_parallel_to(line_b) && line_a.is_coplanar_with(line_b)
}

pub fn infinite_line3d_line_segment3d_collides<T: Scalar>(
    line: &InfiniteLine3D<T>,
    segment: &LineSegment3D<T>,
    tolerance: T,
) -> bool {
    let d1 = line.distance_to_point(&segment.start());
    let d2 = line.distance_to_point(&segment.end());
    d1.min(d2) <= tolerance
}

pub fn infinite_line3d_ray3d_collides<T: Scalar>(
    line: &InfiniteLine3D<T>,
    ray: &Ray3D<T>,
    tolerance: T,
) -> bool {
    line.distance_to_point(&ray.origin()) <= tolerance
}

pub fn infinite_line3d_plane3d_collides<T: Scalar>(
    line: &InfiniteLine3D<T>,
    plane: &Plane3D<T>,
    tolerance: T,
) -> bool {
    plane3d_infinite_line3d_collides(plane, line, tolerance)
}

#[cfg(test)]
mod tests {
    use super::{
        arc3d_point3d_collides, circle3d_point3d_collides, cylindrical_solid3d_point3d_collides,
        cylindrical_surface3d_point3d_collides, ellipse3d_point3d_collides,
        infinite_line3d_plane3d_collides, infinite_line3d_point3d_collides,
        line_segment3d_plane3d_collides, line_segment3d_point3d_collides,
        line_segment3d_triangle3d_collides, plane3d_infinite_line3d_collides,
        plane3d_line_segment3d_collides, plane3d_point3d_collides, plane3d_ray3d_collides,
        ray3d_plane3d_collides, ray3d_point3d_collides, ray3d_triangle3d_collides,
        spherical_solid3d_point3d_collides, torus_solid3d_point3d_collides,
        torus_surface3d_point3d_collides, triangle3d_line_segment3d_collides,
        triangle3d_point3d_collides, triangle3d_ray3d_collides, triangle_mesh3d_point3d_collides,
    };
    use crate::{
        Angle, Arc3D, Circle3D, CylindricalSolid3D, CylindricalSurface3D, Direction3D, Ellipse3D,
        InfiniteLine3D, LineSegment3D, Plane3D, Point3D, Ray3D, SphericalSolid3D, TorusSolid3D,
        TorusSurface3D, Triangle3D, TriangleMesh3D, Vector3D,
    };
    use analysis::test_constants;

    fn standard_distance_tol() -> f64 {
        test_constants::DISTANCE_TOLERANCE_F64
    }

    #[test]
    fn arc_point_collision_checks_angle_range() {
        let tol = standard_distance_tol();
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

        assert!(arc3d_point3d_collides(&arc, &on_arc, tol));
        assert!(!arc3d_point3d_collides(&arc, &out_of_angle, tol));
    }

    #[test]
    fn ellipse_point_collision_uses_distance() {
        let tol = standard_distance_tol();
        let ellipse = Ellipse3D::new(
            Point3D::new(0.0, 0.0, 0.0),
            3.0,
            2.0,
            Vector3D::new(0.0, 0.0, 1.0),
            Vector3D::new(1.0, 0.0, 0.0),
        )
        .unwrap();

        let on_ellipse = Point3D::new(3.0, 0.0, 0.0);
        let outside_plane = Point3D::new(0.0, 0.0, 0.5);

        assert!(ellipse3d_point3d_collides(&ellipse, &on_ellipse, tol));
        assert!(!ellipse3d_point3d_collides(&ellipse, &outside_plane, tol));
    }

    #[test]
    fn torus_surface_point_collision_uses_surface_distance() {
        let tol = standard_distance_tol();
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

        assert!(torus_surface3d_point3d_collides(&torus, &on_surface, tol));
        assert!(!torus_surface3d_point3d_collides(&torus, &inside_tube, tol));
    }

    #[test]
    fn circle_plane_ray_line_point_collisions_use_geometric_checks() {
        let tol = standard_distance_tol();
        let circle = Circle3D::new(
            Point3D::new(0.0, 0.0, 0.0),
            Direction3D::new(0.0, 0.0, 1.0).unwrap(),
            2.0,
        )
        .unwrap();
        assert!(circle3d_point3d_collides(
            &circle,
            &Point3D::new(2.0, 0.0, 0.0),
            tol
        ));
        assert!(!circle3d_point3d_collides(
            &circle,
            &Point3D::new(1.0, 0.0, 0.0),
            tol
        ));

        let plane = Plane3D::xy_plane(0.0);
        assert!(plane3d_point3d_collides(
            &plane,
            &Point3D::new(0.0, 0.0, 0.0),
            tol
        ));
        assert!(!plane3d_point3d_collides(
            &plane,
            &Point3D::new(0.0, 0.0, 1.0),
            tol
        ));

        let ray = Ray3D::new(Point3D::new(0.0, 0.0, 0.0), Vector3D::new(1.0, 0.0, 0.0)).unwrap();
        assert!(ray3d_point3d_collides(
            &ray,
            &Point3D::new(2.0, 0.0, 0.0),
            tol
        ));
        assert!(!ray3d_point3d_collides(
            &ray,
            &Point3D::new(-1.0, 0.0, 0.0),
            tol
        ));

        let segment =
            LineSegment3D::new(Point3D::new(0.0, 0.0, 0.0), Point3D::new(1.0, 0.0, 0.0)).unwrap();
        assert!(line_segment3d_point3d_collides(
            &segment,
            &Point3D::new(0.5, 0.0, 0.0),
            tol
        ));
        assert!(!line_segment3d_point3d_collides(
            &segment,
            &Point3D::new(2.0, 0.0, 0.0),
            tol
        ));

        let line = InfiniteLine3D::from_two_points(
            Point3D::new(0.0, 0.0, 0.0),
            Point3D::new(1.0, 0.0, 0.0),
        )
        .unwrap();
        assert!(infinite_line3d_point3d_collides(
            &line,
            &Point3D::new(10.0, 0.0, 0.0),
            tol
        ));
        assert!(!infinite_line3d_point3d_collides(
            &line,
            &Point3D::new(0.0, 1.0, 0.0),
            tol
        ));
    }

    #[test]
    fn triangle_ellipsoid_and_torus_solid_point_collisions() {
        let tol = standard_distance_tol();
        let tri = Triangle3D::new(
            Point3D::new(0.0, 0.0, 0.0),
            Point3D::new(1.0, 0.0, 0.0),
            Point3D::new(0.0, 1.0, 0.0),
        )
        .unwrap();
        assert!(triangle3d_point3d_collides(
            &tri,
            &Point3D::new(0.2, 0.2, 0.0),
            tol
        ));
        assert!(!triangle3d_point3d_collides(
            &tri,
            &Point3D::new(0.2, 0.2, 0.4),
            tol
        ));

        let torus_solid = TorusSolid3D::new(
            Point3D::new(0.0, 0.0, 0.0),
            Direction3D::new(0.0, 0.0, 1.0).unwrap(),
            Direction3D::new(1.0, 0.0, 0.0).unwrap(),
            3.0,
            1.0,
        )
        .unwrap();
        assert!(torus_solid3d_point3d_collides(
            &torus_solid,
            &Point3D::new(3.0, 0.0, 0.0),
            tol
        ));
        assert!(!torus_solid3d_point3d_collides(
            &torus_solid,
            &Point3D::new(0.0, 0.0, 0.0),
            tol
        ));
    }

    #[test]
    fn spherical_and_cylindrical_point_collisions_use_distance_based_checks() {
        let tol = standard_distance_tol();
        let sphere = SphericalSolid3D::new(
            Point3D::new(0.0, 0.0, 0.0),
            Vector3D::new(0.0, 0.0, 1.0),
            Vector3D::new(1.0, 0.0, 0.0),
            2.0,
        )
        .unwrap();
        assert!(spherical_solid3d_point3d_collides(
            &sphere,
            &Point3D::new(1.0, 0.0, 0.0),
            tol
        ));
        assert!(!spherical_solid3d_point3d_collides(
            &sphere,
            &Point3D::new(4.0, 0.0, 0.0),
            tol
        ));

        let cyl_solid =
            CylindricalSolid3D::new_z_axis(Point3D::new(0.0, 0.0, 0.0), 1.0, 2.0).unwrap();
        assert!(cylindrical_solid3d_point3d_collides(
            &cyl_solid,
            &Point3D::new(0.5, 0.0, 1.0),
            tol
        ));
        assert!(!cylindrical_solid3d_point3d_collides(
            &cyl_solid,
            &Point3D::new(2.0, 0.0, 1.0),
            tol
        ));

        let cyl_surface =
            CylindricalSurface3D::new_z_axis(Point3D::new(0.0, 0.0, 0.0), 1.0).unwrap();
        assert!(cylindrical_surface3d_point3d_collides(
            &cyl_surface,
            &Point3D::new(1.0, 0.0, 0.5),
            tol
        ));
        assert!(!cylindrical_surface3d_point3d_collides(
            &cyl_surface,
            &Point3D::new(2.0, 0.0, 0.5),
            tol
        ));
    }

    #[test]
    fn triangle_mesh_point_collision_checks_member_triangles() {
        let tol = standard_distance_tol();
        let mesh = TriangleMesh3D::new(
            vec![
                Point3D::new(0.0, 0.0, 0.0),
                Point3D::new(1.0, 0.0, 0.0),
                Point3D::new(0.0, 1.0, 0.0),
            ],
            vec![[0, 1, 2]],
        )
        .unwrap();

        assert!(triangle_mesh3d_point3d_collides(
            &mesh,
            &Point3D::new(0.2, 0.2, 0.0),
            tol
        ));
        assert!(!triangle_mesh3d_point3d_collides(
            &mesh,
            &Point3D::new(0.2, 0.2, 0.3),
            tol
        ));
    }

    #[test]
    fn symmetric_triangle_collision_wrappers_match_base_functions() {
        let tol = standard_distance_tol();
        let tri = Triangle3D::new(
            Point3D::new(0.0, 0.0, 0.0),
            Point3D::new(1.0, 0.0, 0.0),
            Point3D::new(0.0, 1.0, 0.0),
        )
        .unwrap();
        let seg =
            LineSegment3D::new(Point3D::new(0.2, 0.2, -1.0), Point3D::new(0.2, 0.2, 1.0)).unwrap();
        let ray = Ray3D::new(Point3D::new(0.2, 0.2, 1.0), Vector3D::new(0.0, 0.0, -1.0)).unwrap();

        assert_eq!(
            line_segment3d_triangle3d_collides(&seg, &tri, tol),
            triangle3d_line_segment3d_collides(&tri, &seg, tol)
        );
        assert_eq!(
            ray3d_triangle3d_collides(&ray, &tri, tol),
            triangle3d_ray3d_collides(&tri, &ray, tol)
        );
    }

    #[test]
    fn symmetric_plane_collision_wrappers_match_base_functions() {
        let tol = standard_distance_tol();
        let plane = Plane3D::xy_plane(0.0_f64);
        let ray = Ray3D::new(Point3D::new(0.0, 0.0, 1.0), Vector3D::new(0.0, 0.0, -1.0)).unwrap();
        let seg =
            LineSegment3D::new(Point3D::new(0.0, 0.0, -1.0), Point3D::new(0.0, 0.0, 1.0)).unwrap();
        let line = InfiniteLine3D::from_two_points(
            Point3D::new(0.0, 0.0, -1.0),
            Point3D::new(0.0, 0.0, 1.0),
        )
        .unwrap();

        assert_eq!(
            ray3d_plane3d_collides(&ray, &plane, tol),
            plane3d_ray3d_collides(&plane, &ray, tol)
        );
        assert_eq!(
            line_segment3d_plane3d_collides(&seg, &plane, tol),
            plane3d_line_segment3d_collides(&plane, &seg, tol)
        );
        assert_eq!(
            infinite_line3d_plane3d_collides(&line, &plane, tol),
            plane3d_infinite_line3d_collides(&plane, &line, tol)
        );
    }
}
