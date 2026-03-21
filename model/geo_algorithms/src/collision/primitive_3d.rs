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
    Arc3DMeasure, BasicCollision, CylindricalSolid3DMeasure, CylindricalSurface3DMeasure,
    Ellipse3DMeasure, Scalar, TorusSurface3DMeasure,
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
    sphere.intersects(circle, tolerance)
}

pub fn spherical_solid3d_line_segment3d_collides<T: Scalar>(
    sphere: &SphericalSolid3D<T>,
    segment: &LineSegment3D<T>,
    tolerance: T,
) -> bool {
    sphere.intersects(segment, tolerance)
}

pub fn spherical_solid3d_ray3d_collides<T: Scalar>(
    sphere: &SphericalSolid3D<T>,
    ray: &Ray3D<T>,
    tolerance: T,
) -> bool {
    sphere.intersects(ray, tolerance)
}

pub fn spherical_solid3d_infinite_line3d_collides<T: Scalar>(
    sphere: &SphericalSolid3D<T>,
    line: &InfiniteLine3D<T>,
    tolerance: T,
) -> bool {
    sphere.intersects(line, tolerance)
}

pub fn spherical_solid3d_triangle3d_collides<T: Scalar>(
    sphere: &SphericalSolid3D<T>,
    triangle: &Triangle3D<T>,
    tolerance: T,
) -> bool {
    sphere.intersects(triangle, tolerance)
}

pub fn spherical_solid3d_plane3d_collides<T: Scalar>(
    sphere: &SphericalSolid3D<T>,
    plane: &Plane3D<T>,
    tolerance: T,
) -> bool {
    sphere.intersects(plane, tolerance)
}

pub fn spherical_solid3d_spherical_solid3d_collides<T: Scalar>(
    sphere_a: &SphericalSolid3D<T>,
    sphere_b: &SphericalSolid3D<T>,
    tolerance: T,
) -> bool {
    sphere_a.intersects(sphere_b, tolerance)
}

// ── CylindricalSolid3D ────────────────────────────────────────────────────────

pub fn cylindrical_solid3d_point3d_collides<T: Scalar>(
    cyl: &CylindricalSolid3D<T>,
    point: &Point3D<T>,
    tolerance: T,
) -> bool {
    <CylindricalSolid3D<T> as CylindricalSolid3DMeasure<T>>::distance_to_point(
        cyl,
        (point.x(), point.y(), point.z()),
    ) <= tolerance
}

pub fn cylindrical_solid3d_circle3d_collides<T: Scalar>(
    cyl: &CylindricalSolid3D<T>,
    circle: &Circle3D<T>,
    tolerance: T,
) -> bool {
    cyl.intersects(circle, tolerance)
}

pub fn cylindrical_solid3d_line_segment3d_collides<T: Scalar>(
    cyl: &CylindricalSolid3D<T>,
    segment: &LineSegment3D<T>,
    tolerance: T,
) -> bool {
    cyl.intersects(segment, tolerance)
}

pub fn cylindrical_solid3d_infinite_line3d_collides<T: Scalar>(
    cyl: &CylindricalSolid3D<T>,
    line: &InfiniteLine3D<T>,
    tolerance: T,
) -> bool {
    cyl.intersects(line, tolerance)
}

pub fn cylindrical_solid3d_ray3d_collides<T: Scalar>(
    cyl: &CylindricalSolid3D<T>,
    ray: &Ray3D<T>,
    tolerance: T,
) -> bool {
    cyl.intersects(ray, tolerance)
}

pub fn cylindrical_solid3d_triangle3d_collides<T: Scalar>(
    cyl: &CylindricalSolid3D<T>,
    triangle: &Triangle3D<T>,
    tolerance: T,
) -> bool {
    cyl.intersects(triangle, tolerance)
}

pub fn cylindrical_solid3d_plane3d_collides<T: Scalar>(
    cyl: &CylindricalSolid3D<T>,
    plane: &Plane3D<T>,
    tolerance: T,
) -> bool {
    cyl.intersects(plane, tolerance)
}

pub fn cylindrical_solid3d_cylindrical_solid3d_collides<T: Scalar>(
    cyl_a: &CylindricalSolid3D<T>,
    cyl_b: &CylindricalSolid3D<T>,
    tolerance: T,
) -> bool {
    cyl_a.intersects(cyl_b, tolerance)
}

// ── CylindricalSurface3D ──────────────────────────────────────────────────────

pub fn cylindrical_surface3d_point3d_collides<T: Scalar>(
    cyl: &CylindricalSurface3D<T>,
    point: &Point3D<T>,
    tolerance: T,
) -> bool {
    <CylindricalSurface3D<T> as CylindricalSurface3DMeasure<T>>::distance_to_point(
        cyl,
        (point.x(), point.y(), point.z()),
    ) <= tolerance
}

pub fn cylindrical_surface3d_circle3d_collides<T: Scalar>(
    cyl: &CylindricalSurface3D<T>,
    circle: &Circle3D<T>,
    tolerance: T,
) -> bool {
    cyl.intersects(circle, tolerance)
}

pub fn cylindrical_surface3d_line_segment3d_collides<T: Scalar>(
    cyl: &CylindricalSurface3D<T>,
    segment: &LineSegment3D<T>,
    tolerance: T,
) -> bool {
    cyl.intersects(segment, tolerance)
}

pub fn cylindrical_surface3d_ray3d_collides<T: Scalar>(
    cyl: &CylindricalSurface3D<T>,
    ray: &Ray3D<T>,
    tolerance: T,
) -> bool {
    cyl.intersects(ray, tolerance)
}

pub fn cylindrical_surface3d_infinite_line3d_collides<T: Scalar>(
    cyl: &CylindricalSurface3D<T>,
    line: &InfiniteLine3D<T>,
    tolerance: T,
) -> bool {
    cyl.intersects(line, tolerance)
}

pub fn cylindrical_surface3d_triangle3d_collides<T: Scalar>(
    cyl: &CylindricalSurface3D<T>,
    triangle: &Triangle3D<T>,
    tolerance: T,
) -> bool {
    cyl.intersects(triangle, tolerance)
}

pub fn cylindrical_surface3d_plane3d_collides<T: Scalar>(
    cyl: &CylindricalSurface3D<T>,
    plane: &Plane3D<T>,
    tolerance: T,
) -> bool {
    cyl.intersects(plane, tolerance)
}

pub fn cylindrical_surface3d_cylindrical_surface3d_collides<T: Scalar>(
    cyl_a: &CylindricalSurface3D<T>,
    cyl_b: &CylindricalSurface3D<T>,
    tolerance: T,
) -> bool {
    cyl_a.intersects(cyl_b, tolerance)
}

// ── Ellipse3D ─────────────────────────────────────────────────────────────────

pub fn ellipse3d_point3d_collides<T: Scalar + From<f64>>(
    ellipse: &Ellipse3D<T>,
    point: &Point3D<T>,
    tolerance: T,
) -> bool {
    <Ellipse3D<T> as Ellipse3DMeasure<T>>::distance_to_point_3d(
        ellipse,
        (point.x(), point.y(), point.z()),
    ) <= tolerance
}

pub fn ellipse3d_circle3d_collides<T: Scalar>(
    ellipse: &Ellipse3D<T>,
    circle: &Circle3D<T>,
    tolerance: T,
) -> bool {
    ellipse.intersects(circle, tolerance)
}

pub fn ellipse3d_arc3d_collides<T: Scalar>(
    ellipse: &Ellipse3D<T>,
    arc: &Arc3D<T>,
    tolerance: T,
) -> bool {
    ellipse.intersects(arc, tolerance)
}

pub fn ellipse3d_line_segment3d_collides<T: Scalar>(
    ellipse: &Ellipse3D<T>,
    segment: &LineSegment3D<T>,
    tolerance: T,
) -> bool {
    ellipse.intersects(segment, tolerance)
}

pub fn ellipse3d_infinite_line3d_collides<T: Scalar>(
    ellipse: &Ellipse3D<T>,
    line: &InfiniteLine3D<T>,
    tolerance: T,
) -> bool {
    ellipse.intersects(line, tolerance)
}

pub fn ellipse3d_ray3d_collides<T: Scalar>(
    ellipse: &Ellipse3D<T>,
    ray: &Ray3D<T>,
    tolerance: T,
) -> bool {
    ellipse.intersects(ray, tolerance)
}

pub fn ellipse3d_plane3d_collides<T: Scalar>(
    ellipse: &Ellipse3D<T>,
    plane: &Plane3D<T>,
    tolerance: T,
) -> bool {
    ellipse.intersects(plane, tolerance)
}

pub fn ellipse3d_triangle3d_collides<T: Scalar>(
    ellipse: &Ellipse3D<T>,
    triangle: &Triangle3D<T>,
    tolerance: T,
) -> bool {
    ellipse.intersects(triangle, tolerance)
}

pub fn ellipse3d_ellipse3d_collides<T: Scalar>(
    ellipse_a: &Ellipse3D<T>,
    ellipse_b: &Ellipse3D<T>,
    tolerance: T,
) -> bool {
    ellipse_a.intersects(ellipse_b, tolerance)
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
    ellipsoid.intersects(segment, tolerance)
}

pub fn ellipsoidal_solid3d_ray3d_collides<T: Scalar>(
    ellipsoid: &EllipsoidalSolid3D<T>,
    ray: &Ray3D<T>,
    tolerance: T,
) -> bool {
    ellipsoid.intersects(ray, tolerance)
}

pub fn ellipsoidal_solid3d_infinite_line3d_collides<T: Scalar>(
    ellipsoid: &EllipsoidalSolid3D<T>,
    line: &InfiniteLine3D<T>,
    tolerance: T,
) -> bool {
    ellipsoid.intersects(line, tolerance)
}

pub fn ellipsoidal_solid3d_plane3d_collides<T: Scalar>(
    ellipsoid: &EllipsoidalSolid3D<T>,
    plane: &Plane3D<T>,
    tolerance: T,
) -> bool {
    ellipsoid.intersects(plane, tolerance)
}

pub fn ellipsoidal_solid3d_ellipsoidal_solid3d_collides<T: Scalar>(
    ell_a: &EllipsoidalSolid3D<T>,
    ell_b: &EllipsoidalSolid3D<T>,
    tolerance: T,
) -> bool {
    ell_a.intersects(ell_b, tolerance)
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
    <TorusSurface3D<T> as TorusSurface3DMeasure<T>>::distance_to_point(
        torus,
        (point.x(), point.y(), point.z()),
    ) <= tolerance
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
    triangle.intersects(segment, tolerance)
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
    triangle.intersects(ray, tolerance)
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
    triangle_a.intersects(triangle_b, tolerance)
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
    <Arc3D<T> as Arc3DMeasure<T>>::distance_to_point(arc, (point.x(), point.y(), point.z()))
        <= tolerance
        && arc.contains_point_angle(Point3D::new(point.x(), point.y(), point.z()))
}

pub fn arc3d_line_segment3d_collides<T: Scalar>(
    arc: &Arc3D<T>,
    segment: &LineSegment3D<T>,
    tolerance: T,
) -> bool {
    arc.intersects(segment, tolerance)
}

pub fn arc3d_ray3d_collides<T: Scalar>(arc: &Arc3D<T>, ray: &Ray3D<T>, tolerance: T) -> bool {
    arc.intersects(ray, tolerance)
}

pub fn arc3d_infinite_line3d_collides<T: Scalar>(
    arc: &Arc3D<T>,
    line: &InfiniteLine3D<T>,
    tolerance: T,
) -> bool {
    arc.intersects(line, tolerance)
}

pub fn arc3d_arc3d_collides<T: Scalar>(arc_a: &Arc3D<T>, arc_b: &Arc3D<T>, tolerance: T) -> bool {
    arc_a.intersects(arc_b, tolerance)
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
    circle.intersects(segment, tolerance)
}

pub fn circle3d_ray3d_collides<T: Scalar>(
    circle: &Circle3D<T>,
    ray: &Ray3D<T>,
    tolerance: T,
) -> bool {
    circle.intersects(ray, tolerance)
}

pub fn circle3d_infinite_line3d_collides<T: Scalar>(
    circle: &Circle3D<T>,
    line: &InfiniteLine3D<T>,
    tolerance: T,
) -> bool {
    circle.intersects(line, tolerance)
}

pub fn circle3d_circle3d_collides<T: Scalar>(
    circle_a: &Circle3D<T>,
    circle_b: &Circle3D<T>,
    tolerance: T,
) -> bool {
    circle_a.intersects(circle_b, tolerance)
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
    plane.intersects(segment, tolerance)
}

pub fn plane3d_ray3d_collides<T: Scalar>(plane: &Plane3D<T>, ray: &Ray3D<T>, tolerance: T) -> bool {
    plane.intersects(ray, tolerance)
}

pub fn plane3d_infinite_line3d_collides<T: Scalar>(
    plane: &Plane3D<T>,
    line: &InfiniteLine3D<T>,
    tolerance: T,
) -> bool {
    plane.intersects(line, tolerance)
}

pub fn plane3d_plane3d_collides<T: Scalar>(
    plane_a: &Plane3D<T>,
    plane_b: &Plane3D<T>,
    tolerance: T,
) -> bool {
    plane_a.intersects(plane_b, tolerance)
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
    ray.intersects(sphere, tolerance)
}

pub fn ray3d_ray3d_collides<T: Scalar>(ray_a: &Ray3D<T>, ray_b: &Ray3D<T>, tolerance: T) -> bool {
    ray_a.intersects(ray_b, tolerance)
}

pub fn ray3d_line_segment3d_collides<T: Scalar>(
    ray: &Ray3D<T>,
    segment: &LineSegment3D<T>,
    tolerance: T,
) -> bool {
    ray.intersects(segment, tolerance)
}

pub fn ray3d_infinite_line3d_collides<T: Scalar>(
    ray: &Ray3D<T>,
    line: &InfiniteLine3D<T>,
    tolerance: T,
) -> bool {
    ray.intersects(line, tolerance)
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
    segment.intersects(sphere, tolerance)
}

pub fn line_segment3d_line_segment3d_collides<T: Scalar>(
    seg_a: &LineSegment3D<T>,
    seg_b: &LineSegment3D<T>,
    tolerance: T,
) -> bool {
    seg_a.intersects(seg_b, tolerance)
}

pub fn line_segment3d_ray3d_collides<T: Scalar>(
    segment: &LineSegment3D<T>,
    ray: &Ray3D<T>,
    tolerance: T,
) -> bool {
    segment.intersects(ray, tolerance)
}

pub fn line_segment3d_infinite_line3d_collides<T: Scalar>(
    segment: &LineSegment3D<T>,
    line: &InfiniteLine3D<T>,
    tolerance: T,
) -> bool {
    segment.intersects(line, tolerance)
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
    line.intersects(sphere, tolerance)
}

pub fn infinite_line3d_infinite_line3d_collides<T: Scalar>(
    line_a: &InfiniteLine3D<T>,
    line_b: &InfiniteLine3D<T>,
    tolerance: T,
) -> bool {
    line_a.intersects(line_b, tolerance)
}

pub fn infinite_line3d_line_segment3d_collides<T: Scalar>(
    line: &InfiniteLine3D<T>,
    segment: &LineSegment3D<T>,
    tolerance: T,
) -> bool {
    line.intersects(segment, tolerance)
}

pub fn infinite_line3d_ray3d_collides<T: Scalar>(
    line: &InfiniteLine3D<T>,
    ray: &Ray3D<T>,
    tolerance: T,
) -> bool {
    line.intersects(ray, tolerance)
}

#[cfg(test)]
mod tests {
    use super::{
        arc3d_point3d_collides, circle3d_point3d_collides, cylindrical_solid3d_point3d_collides,
        cylindrical_surface3d_point3d_collides, ellipse3d_point3d_collides,
        infinite_line3d_point3d_collides, line_segment3d_point3d_collides,
        line_segment3d_triangle3d_collides, plane3d_point3d_collides, ray3d_point3d_collides,
        ray3d_triangle3d_collides, spherical_solid3d_point3d_collides,
        torus_solid3d_point3d_collides, torus_surface3d_point3d_collides,
        triangle3d_line_segment3d_collides, triangle3d_point3d_collides, triangle3d_ray3d_collides,
        triangle_mesh3d_point3d_collides,
    };
    use crate::{
        Angle, Arc3D, Circle3D, CylindricalSolid3D, CylindricalSurface3D, Direction3D, Ellipse3D,
        InfiniteLine3D, LineSegment3D, Plane3D, Point3D, Ray3D, SphericalSolid3D, TorusSolid3D,
        TorusSurface3D, Triangle3D, TriangleMesh3D, Vector3D,
    };
    use geo_contracts::ToleranceSettings;

    fn standard_distance_tol() -> f64 {
        ToleranceSettings::<f64>::standard().distance_tolerance
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
}
