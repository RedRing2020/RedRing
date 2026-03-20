//! 3D Primitive intersection algorithms
//!
//! Phase C Step 2: `geo_primitives` から 3D 交差判定ロジックを
//! `geo_algorithms` 側へ集約するための受け皿。
//!
//! orphan rules により trait 実装ではなく形状ペア free-function を提供する。
//! `geo_primitives` の `BasicIntersection`/`MultipleIntersection` を呼び出す薄い
//! ラッパー、または `pair_base` に委譲する。

use crate::intersection::pair_base;
use crate::{
    Arc3D, Circle3D, CylindricalSurface3D, Ellipse3D, EllipsoidalSolid3D, EllipsoidalSurface3D,
    InfiniteLine3D, LineSegment3D, Plane3D, Point3D, Ray3D, SphericalSurface3D, TorusSolid3D,
    TorusSurface3D, Triangle3D, TriangleMesh3D,
};
use geo_contracts::{
    Arc3DMeasure, BasicIntersection, CylindricalSurface3DMeasure, Ellipse3DMeasure,
    MultipleIntersection, Scalar, TorusSurface3DMeasure,
};

fn point_intersection_if<T: Scalar>(point: &Point3D<T>, condition: bool) -> Option<Point3D<T>> {
    if condition {
        Some(Point3D::new(point.x(), point.y(), point.z()))
    } else {
        None
    }
}

// ── Arc3D ─────────────────────────────────────────────────────────────────────

pub fn arc3d_point3d_intersection<T: Scalar>(
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

pub fn arc3d_line_segment3d_intersection<T: Scalar>(
    arc: &Arc3D<T>,
    segment: &LineSegment3D<T>,
    tolerance: T,
) -> Option<Point3D<T>> {
    arc.intersection_with(segment, tolerance)
}

pub fn arc3d_ray3d_intersection<T: Scalar>(
    arc: &Arc3D<T>,
    ray: &Ray3D<T>,
    tolerance: T,
) -> Option<Point3D<T>> {
    arc.intersection_with(ray, tolerance)
}

pub fn arc3d_infinite_line3d_intersection<T: Scalar>(
    arc: &Arc3D<T>,
    line: &InfiniteLine3D<T>,
    tolerance: T,
) -> Option<Point3D<T>> {
    arc.intersection_with(line, tolerance)
}

pub fn arc3d_arc3d_intersection<T: Scalar>(
    arc_a: &Arc3D<T>,
    arc_b: &Arc3D<T>,
    tolerance: T,
) -> Option<Point3D<T>> {
    arc_a.intersection_with(arc_b, tolerance)
}

// ── Circle3D ──────────────────────────────────────────────────────────────────

pub fn circle3d_point3d_intersection<T: Scalar>(
    circle: &Circle3D<T>,
    point: &Point3D<T>,
    tolerance: T,
) -> Option<Point3D<T>> {
    point_intersection_if(
        point,
        circle.distance_to_point_3d(Point3D::new(point.x(), point.y(), point.z())) <= tolerance,
    )
}

pub fn circle3d_line_segment3d_intersection<T: Scalar>(
    circle: &Circle3D<T>,
    segment: &LineSegment3D<T>,
    tolerance: T,
) -> Option<Point3D<T>> {
    circle.intersection_with(segment, tolerance)
}

pub fn circle3d_ray3d_intersection<T: Scalar>(
    circle: &Circle3D<T>,
    ray: &Ray3D<T>,
    tolerance: T,
) -> Option<Point3D<T>> {
    circle.intersection_with(ray, tolerance)
}

pub fn circle3d_infinite_line3d_intersection<T: Scalar>(
    circle: &Circle3D<T>,
    line: &InfiniteLine3D<T>,
    tolerance: T,
) -> Option<Point3D<T>> {
    circle.intersection_with(line, tolerance)
}

pub fn circle3d_circle3d_intersection<T: Scalar>(
    circle_a: &Circle3D<T>,
    circle_b: &Circle3D<T>,
    tolerance: T,
) -> Option<Point3D<T>> {
    circle_a.intersection_with(circle_b, tolerance)
}

// ── CylindricalSurface3D ──────────────────────────────────────────────────────

pub fn cylindrical_surface3d_point3d_intersection<T: Scalar>(
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

pub fn cylindrical_surface3d_circle3d_intersection<T: Scalar>(
    cyl: &CylindricalSurface3D<T>,
    circle: &Circle3D<T>,
    tolerance: T,
) -> Option<Point3D<T>> {
    cyl.intersection_with(circle, tolerance)
}

pub fn cylindrical_surface3d_line_segment3d_intersection<T: Scalar>(
    cyl: &CylindricalSurface3D<T>,
    segment: &LineSegment3D<T>,
    tolerance: T,
) -> Option<Point3D<T>> {
    cyl.intersection_with(segment, tolerance)
}

pub fn cylindrical_surface3d_triangle3d_intersection<T: Scalar>(
    cyl: &CylindricalSurface3D<T>,
    triangle: &Triangle3D<T>,
    tolerance: T,
) -> Option<Point3D<T>> {
    cyl.intersection_with(triangle, tolerance)
}

pub fn cylindrical_surface3d_plane3d_intersection<T: Scalar>(
    cyl: &CylindricalSurface3D<T>,
    plane: &Plane3D<T>,
    tolerance: T,
) -> Option<Point3D<T>> {
    cyl.intersection_with(plane, tolerance)
}

// cylindrical_surface3d_cylindrical_surface3d_intersection: BasicIntersection<T, CylindricalSurface3D<T>> は未実装のため対象外

// ── Ellipse3D ─────────────────────────────────────────────────────────────────

pub fn ellipse3d_point3d_intersection<T: Scalar + From<f64>>(
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

pub fn ellipse3d_circle3d_intersections<T: Scalar>(
    ellipse: &Ellipse3D<T>,
    circle: &Circle3D<T>,
    tolerance: T,
) -> Vec<Point3D<T>> {
    ellipse.intersections_with(circle, tolerance)
}

pub fn ellipse3d_arc3d_intersections<T: Scalar>(
    ellipse: &Ellipse3D<T>,
    arc: &Arc3D<T>,
    tolerance: T,
) -> Vec<Point3D<T>> {
    ellipse.intersections_with(arc, tolerance)
}

pub fn ellipse3d_line_segment3d_intersections<T: Scalar>(
    ellipse: &Ellipse3D<T>,
    segment: &LineSegment3D<T>,
    tolerance: T,
) -> Vec<Point3D<T>> {
    ellipse.intersections_with(segment, tolerance)
}

pub fn ellipse3d_infinite_line3d_intersections<T: Scalar>(
    ellipse: &Ellipse3D<T>,
    line: &InfiniteLine3D<T>,
    tolerance: T,
) -> Vec<Point3D<T>> {
    ellipse.intersections_with(line, tolerance)
}

pub fn ellipse3d_ray3d_intersections<T: Scalar>(
    ellipse: &Ellipse3D<T>,
    ray: &Ray3D<T>,
    tolerance: T,
) -> Vec<Point3D<T>> {
    ellipse.intersections_with(ray, tolerance)
}

pub fn ellipse3d_plane3d_intersection<T: Scalar>(
    ellipse: &Ellipse3D<T>,
    plane: &Plane3D<T>,
    tolerance: T,
) -> Option<Point3D<T>> {
    ellipse.intersection_with(plane, tolerance)
}

pub fn ellipse3d_triangle3d_intersections<T: Scalar>(
    ellipse: &Ellipse3D<T>,
    triangle: &Triangle3D<T>,
    tolerance: T,
) -> Vec<Point3D<T>> {
    ellipse.intersections_with(triangle, tolerance)
}

pub fn ellipse3d_ellipse3d_intersections<T: Scalar>(
    ellipse_a: &Ellipse3D<T>,
    ellipse_b: &Ellipse3D<T>,
    tolerance: T,
) -> Vec<Point3D<T>> {
    ellipse_a.intersections_with(ellipse_b, tolerance)
}

// ── EllipsoidalSolid3D ────────────────────────────────────────────────────────

pub fn ellipsoidal_solid3d_point3d_intersection<T: Scalar>(
    ellipsoid: &EllipsoidalSolid3D<T>,
    point: &Point3D<T>,
    _tolerance: T,
) -> Option<Point3D<T>> {
    point_intersection_if(point, ellipsoid.contains_point(point))
}

pub fn ellipsoidal_solid3d_plane3d_intersection<T: Scalar>(
    ellipsoid: &EllipsoidalSolid3D<T>,
    plane: &Plane3D<T>,
    tolerance: T,
) -> Option<Point3D<T>> {
    ellipsoid.intersection_with(plane, tolerance)
}

pub fn ellipsoidal_solid3d_infinite_line3d_intersections<T: Scalar>(
    ellipsoid: &EllipsoidalSolid3D<T>,
    line: &InfiniteLine3D<T>,
    tolerance: T,
) -> Vec<Point3D<T>> {
    ellipsoid.intersections_with(line, tolerance)
}

pub fn ellipsoidal_solid3d_ray3d_intersections<T: Scalar>(
    ellipsoid: &EllipsoidalSolid3D<T>,
    ray: &Ray3D<T>,
    tolerance: T,
) -> Vec<Point3D<T>> {
    ellipsoid.intersections_with(ray, tolerance)
}

// ── EllipsoidalSurface3D ──────────────────────────────────────────────────────

pub fn ellipsoidal_surface3d_point3d_intersection<T: Scalar>(
    ellipsoid: &EllipsoidalSurface3D<T>,
    point: &Point3D<T>,
    tolerance: T,
) -> Option<Point3D<T>> {
    point_intersection_if(point, ellipsoid.contains_point(point, tolerance))
}

// SphericalSolid3D の intersection: spherical_solid_3d_intersection モジュールが
// lib.rs でコメントアウトされているため BasicIntersection/MultipleIntersection は未実装。
// collision 側（primitive_3d.rs）の spherical_solid3d_*_collides を使用すること。

// ── TorusSolid3D ──────────────────────────────────────────────────────────────

pub fn torus_solid3d_point3d_intersection<T: Scalar>(
    torus: &TorusSolid3D<T>,
    point: &Point3D<T>,
    _tolerance: T,
) -> Option<Point3D<T>> {
    point_intersection_if(point, torus.contains_point(point))
}

// ── TorusSurface3D ────────────────────────────────────────────────────────────

pub fn torus_surface3d_point3d_intersection<T: Scalar>(
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

// ── Triangle3D ────────────────────────────────────────────────────────────────

pub fn triangle3d_point3d_intersection<T: Scalar>(
    triangle: &Triangle3D<T>,
    point: &Point3D<T>,
    tolerance: T,
) -> Option<Point3D<T>> {
    point_intersection_if(point, triangle.distance_to_point(point) <= tolerance)
}

pub fn triangle3d_line_segment3d_intersection<T: Scalar>(
    triangle: &Triangle3D<T>,
    segment: &LineSegment3D<T>,
    _tolerance: T,
) -> Option<Point3D<T>> {
    pair_base::triangle3d_line_segment3d_intersection(triangle, segment)
}

pub fn line_segment3d_triangle3d_intersection<T: Scalar>(
    segment: &LineSegment3D<T>,
    triangle: &Triangle3D<T>,
    tolerance: T,
) -> Option<Point3D<T>> {
    triangle3d_line_segment3d_intersection(triangle, segment, tolerance)
}

pub fn triangle3d_ray3d_intersection<T: Scalar>(
    triangle: &Triangle3D<T>,
    ray: &Ray3D<T>,
    _tolerance: T,
) -> Option<Point3D<T>> {
    pair_base::triangle3d_ray3d_intersection(triangle, ray)
}

pub fn ray3d_triangle3d_intersection<T: Scalar>(
    ray: &Ray3D<T>,
    triangle: &Triangle3D<T>,
    tolerance: T,
) -> Option<Point3D<T>> {
    triangle3d_ray3d_intersection(triangle, ray, tolerance)
}

// ── TriangleMesh3D ────────────────────────────────────────────────────────────

pub fn triangle_mesh3d_point3d_intersection<T: Scalar>(
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

// ── Plane3D ───────────────────────────────────────────────────────────────────

pub fn plane3d_point3d_intersection<T: Scalar>(
    plane: &Plane3D<T>,
    point: &Point3D<T>,
    tolerance: T,
) -> Option<Point3D<T>> {
    point_intersection_if(
        point,
        plane.contains_point(Point3D::new(point.x(), point.y(), point.z()), tolerance),
    )
}

pub fn plane3d_line_segment3d_intersection<T: Scalar>(
    plane: &Plane3D<T>,
    segment: &LineSegment3D<T>,
    tolerance: T,
) -> Option<Point3D<T>> {
    pair_base::plane3d_line_segment3d_intersection(plane, segment, tolerance)
}

pub fn plane3d_ray3d_intersection<T: Scalar>(
    plane: &Plane3D<T>,
    ray: &Ray3D<T>,
    tolerance: T,
) -> Option<Point3D<T>> {
    pair_base::plane3d_ray3d_intersection(plane, ray, tolerance)
}

pub fn plane3d_infinite_line3d_intersection<T: Scalar>(
    plane: &Plane3D<T>,
    line: &InfiniteLine3D<T>,
    _tolerance: T,
) -> Option<Point3D<T>> {
    pair_base::plane3d_infinite_line3d_intersection(plane, line)
}

// ── Ray3D ─────────────────────────────────────────────────────────────────────

pub fn ray3d_point3d_intersection<T: Scalar>(
    ray: &Ray3D<T>,
    point: &Point3D<T>,
    tolerance: T,
) -> Option<Point3D<T>> {
    point_intersection_if(point, ray.contains_point(point, tolerance))
}

pub fn ray3d_spherical_surface3d_intersections<T: Scalar>(
    ray: &Ray3D<T>,
    sphere: &SphericalSurface3D<T>,
) -> Vec<Point3D<T>> {
    pair_base::ray3d_spherical_surface3d_intersections(ray, sphere)
}

pub fn ray3d_ray3d_intersection<T: Scalar>(
    ray_a: &Ray3D<T>,
    ray_b: &Ray3D<T>,
) -> Option<Point3D<T>> {
    pair_base::ray3d_ray3d_intersection(ray_a, ray_b)
}

pub fn ray3d_line_segment3d_intersection<T: Scalar>(
    ray: &Ray3D<T>,
    segment: &LineSegment3D<T>,
    _tolerance: T,
) -> Option<Point3D<T>> {
    pair_base::ray3d_line_segment3d_intersection(ray, segment)
}

pub fn ray3d_infinite_line3d_intersection<T: Scalar>(
    ray: &Ray3D<T>,
    line: &InfiniteLine3D<T>,
    _tolerance: T,
) -> Option<Point3D<T>> {
    pair_base::ray3d_infinite_line3d_intersection(ray, line)
}

// ── LineSegment3D ─────────────────────────────────────────────────────────────

pub fn line_segment3d_point3d_intersection<T: Scalar>(
    segment: &LineSegment3D<T>,
    point: &Point3D<T>,
    tolerance: T,
) -> Option<Point3D<T>> {
    point_intersection_if(point, segment.contains_point(point, tolerance))
}

pub fn line_segment3d_spherical_surface3d_intersections<T: Scalar>(
    segment: &LineSegment3D<T>,
    sphere: &SphericalSurface3D<T>,
) -> Vec<Point3D<T>> {
    pair_base::line_segment3d_spherical_surface3d_intersections(segment, sphere)
}

pub fn line_segment3d_line_segment3d_intersection<T: Scalar>(
    seg_a: &LineSegment3D<T>,
    seg_b: &LineSegment3D<T>,
    tolerance: T,
) -> Option<Point3D<T>> {
    pair_base::line_segment3d_line_segment3d_intersection(seg_a, seg_b, tolerance)
}

// ── InfiniteLine3D ────────────────────────────────────────────────────────────

pub fn infinite_line3d_point3d_intersection<T: Scalar>(
    line: &InfiniteLine3D<T>,
    point: &Point3D<T>,
    tolerance: T,
) -> Option<Point3D<T>> {
    point_intersection_if(point, line.contains_point(point, tolerance))
}

pub fn infinite_line3d_spherical_surface3d_intersections<T: Scalar>(
    line: &InfiniteLine3D<T>,
    sphere: &SphericalSurface3D<T>,
) -> Vec<Point3D<T>> {
    pair_base::infinite_line3d_spherical_surface3d_intersections(line, sphere)
}

pub fn infinite_line3d_infinite_line3d_intersection<T: Scalar>(
    line_a: &InfiniteLine3D<T>,
    line_b: &InfiniteLine3D<T>,
    tolerance: T,
) -> Option<Point3D<T>> {
    pair_base::infinite_line3d_infinite_line3d_intersection(line_a, line_b, tolerance)
}

pub fn infinite_line3d_line_segment3d_intersection<T: Scalar>(
    line: &InfiniteLine3D<T>,
    segment: &LineSegment3D<T>,
    tolerance: T,
) -> Option<Point3D<T>> {
    pair_base::infinite_line3d_line_segment3d_intersection(line, segment, tolerance)
}

pub fn infinite_line3d_ray3d_intersection<T: Scalar>(
    line: &InfiniteLine3D<T>,
    ray: &Ray3D<T>,
    tolerance: T,
) -> Option<Point3D<T>> {
    pair_base::infinite_line3d_ray3d_intersection(line, ray, tolerance)
}

#[cfg(test)]
mod tests {
    use super::{
        arc3d_point3d_intersection, circle3d_point3d_intersection,
        cylindrical_surface3d_point3d_intersection, ellipse3d_point3d_intersection,
        infinite_line3d_point3d_intersection, line_segment3d_point3d_intersection,
        line_segment3d_triangle3d_intersection, plane3d_point3d_intersection,
        ray3d_point3d_intersection, ray3d_triangle3d_intersection,
        torus_surface3d_point3d_intersection, triangle3d_line_segment3d_intersection,
        triangle3d_point3d_intersection, triangle3d_ray3d_intersection,
        triangle_mesh3d_point3d_intersection,
    };
    use crate::{
        Angle, Arc3D, Circle3D, CylindricalSurface3D, Direction3D, Ellipse3D, InfiniteLine3D,
        LineSegment3D, Plane3D, Point3D, Ray3D, TorusSurface3D, Triangle3D, TriangleMesh3D,
        Vector3D,
    };

    #[test]
    fn plane_point_intersection_returns_same_point() {
        let plane = Plane3D::xy_plane(0.0_f64);
        let point = Point3D::new(1.0, -2.0, 0.0);

        let result = plane3d_point3d_intersection(&plane, &point, 1e-6);

        assert_eq!(result, Some(point));
    }

    #[test]
    fn ray_point_intersection_respects_ray_direction() {
        let ray = Ray3D::new(Point3D::new(0.0, 0.0, 0.0), Vector3D::new(1.0, 0.0, 0.0)).unwrap();
        let on_ray = Point3D::new(2.0, 0.0, 0.0);
        let behind_ray = Point3D::new(-1.0, 0.0, 0.0);

        assert_eq!(
            ray3d_point3d_intersection(&ray, &on_ray, 1e-6),
            Some(on_ray)
        );
        assert_eq!(ray3d_point3d_intersection(&ray, &behind_ray, 1e-6), None);
    }

    #[test]
    fn line_segment_point_intersection_checks_segment_bounds() {
        let segment =
            LineSegment3D::new(Point3D::new(0.0, 0.0, 0.0), Point3D::new(2.0, 0.0, 0.0)).unwrap();
        let on_segment = Point3D::new(1.0, 0.0, 0.0);
        let outside_segment = Point3D::new(3.0, 0.0, 0.0);

        assert_eq!(
            line_segment3d_point3d_intersection(&segment, &on_segment, 1e-6),
            Some(on_segment)
        );
        assert_eq!(
            line_segment3d_point3d_intersection(&segment, &outside_segment, 1e-6),
            None
        );
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

        assert_eq!(
            infinite_line3d_point3d_intersection(&line, &on_line, 1e-6),
            Some(on_line)
        );
        assert_eq!(
            infinite_line3d_point3d_intersection(&line, &off_line, 1e-6),
            None
        );
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

        assert_eq!(
            triangle3d_point3d_intersection(&triangle, &on_triangle, 1e-6),
            Some(on_triangle)
        );
        assert_eq!(
            triangle3d_point3d_intersection(&triangle, &off_triangle, 1e-6),
            None
        );
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
            circle3d_point3d_intersection(&circle, &on_circle, 1e-6),
            Some(on_circle)
        );
        assert_eq!(
            circle3d_point3d_intersection(&circle, &inside_disk, 1e-6),
            None
        );
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
            arc3d_point3d_intersection(&arc, &on_arc, 1e-6),
            Some(on_arc)
        );
        assert_eq!(arc3d_point3d_intersection(&arc, &out_of_angle, 1e-6), None);
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

        assert_eq!(
            ellipse3d_point3d_intersection(&ellipse, &on_ellipse, 1e-6),
            Some(on_ellipse)
        );
        assert_eq!(
            ellipse3d_point3d_intersection(&ellipse, &inside_ellipse, 1e-6),
            Some(inside_ellipse)
        );
        assert_eq!(
            ellipse3d_point3d_intersection(&ellipse, &outside_plane, 1e-6),
            None
        );
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

        assert_eq!(
            torus_surface3d_point3d_intersection(&torus, &on_surface, 1e-6),
            Some(on_surface)
        );
        assert_eq!(
            torus_surface3d_point3d_intersection(&torus, &inside_tube, 1e-6),
            None
        );
    }

    #[test]
    fn cylindrical_surface_point_intersection_uses_surface_distance() {
        let cyl = CylindricalSurface3D::new_z_axis(Point3D::new(0.0, 0.0, 0.0), 1.0).unwrap();

        let on_surface = Point3D::new(1.0, 0.0, 0.5);
        let off_surface = Point3D::new(2.0, 0.0, 0.5);

        assert_eq!(
            cylindrical_surface3d_point3d_intersection(&cyl, &on_surface, 1e-6),
            Some(on_surface)
        );
        assert_eq!(
            cylindrical_surface3d_point3d_intersection(&cyl, &off_surface, 1e-6),
            None
        );
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

        assert_eq!(
            triangle_mesh3d_point3d_intersection(&mesh, &on_triangle, 1e-6),
            Some(on_triangle)
        );
        assert_eq!(
            triangle_mesh3d_point3d_intersection(&mesh, &off_triangle, 1e-6),
            None
        );
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

        let tol = 1e-6;
        assert_eq!(
            line_segment3d_triangle3d_intersection(&seg, &tri, tol),
            triangle3d_line_segment3d_intersection(&tri, &seg, tol)
        );
        assert_eq!(
            ray3d_triangle3d_intersection(&ray, &tri, tol),
            triangle3d_ray3d_intersection(&tri, &ray, tol)
        );
    }
}
