//! 3D Primitive distance algorithms
//!
//! `geo_primitives` に実装済みの各 3D 形状の距離計算メソッドを
//! `geo_algorithms` 層の公開 entrypoint としてまとめた薄いラッパー群。
//!
//! 命名規則: `{shape_a}_{shape_b}_distance`

use crate::{
    Arc3D, Circle3D, ConicalSurface3D, CylindricalSolid3D, CylindricalSurface3D, Ellipse3D,
    EllipsoidalSolid3D, EllipsoidalSurface3D, InfiniteLine3D, LineSegment3D, Plane3D, Point3D,
    Ray3D, SphericalSolid3D, TorusSolid3D, TorusSurface3D, Triangle3D, TriangleMesh3D,
};
use geo_contracts::{
    Arc3DDistance, ConicalSurface3DDistance, CylindricalSolid3DDistance,
    CylindricalSurface3DDistance, Ellipse3DDistance, EllipsoidalSolid3DContainment,
    EllipsoidalSolid3DDistance, EllipsoidalSurface3DDistance, Scalar, SphericalSolid3DContainment,
    SphericalSolid3DDistance, TorusSolid3DContainment, TorusSolid3DDistance,
    TorusSurface3DDistance,
};

/// LineSegment3D-点 間の最短距離（端点クランプあり）
pub fn line_segment3d_point3d_distance<T: Scalar>(
    segment: &LineSegment3D<T>,
    point: &Point3D<T>,
) -> T {
    segment.distance_to_point(point)
}

/// 逆向きラッパー: point-segment
pub fn point3d_line_segment3d_distance<T: Scalar>(
    point: &Point3D<T>,
    segment: &LineSegment3D<T>,
) -> T {
    segment.distance_to_point(point)
}

/// 無限直線3D-点 間の最短距離（垂直距離）
pub fn infinite_line3d_point3d_distance<T: Scalar>(
    line: &InfiniteLine3D<T>,
    point: &Point3D<T>,
) -> T {
    line.distance_to_point(point)
}

/// 逆向きラッパー: point-line
pub fn point3d_infinite_line3d_distance<T: Scalar>(
    point: &Point3D<T>,
    line: &InfiniteLine3D<T>,
) -> T {
    line.distance_to_point(point)
}

/// 無限直線3D-無限直線3D 間の最短距離
pub fn infinite_line3d_infinite_line3d_distance<T: Scalar>(
    line_a: &InfiniteLine3D<T>,
    line_b: &InfiniteLine3D<T>,
) -> T {
    line_a.distance_to_line(line_b)
}

/// Plane3D-点 間の符号付き距離（法線方向の符号を保持）
pub fn plane3d_point3d_distance<T: Scalar>(plane: &Plane3D<T>, point: &Point3D<T>) -> T {
    plane.distance_to_point(*point)
}

/// 逆向きラッパー: point-plane
pub fn point3d_plane3d_distance<T: Scalar>(point: &Point3D<T>, plane: &Plane3D<T>) -> T {
    plane.distance_to_point(*point)
}

/// SphericalSolid3D-点 間の最短距離（内部点は 0）
pub fn spherical_solid3d_point3d_distance<T: Scalar>(
    sphere: &SphericalSolid3D<T>,
    point: &Point3D<T>,
) -> T {
    if <SphericalSolid3D<T> as SphericalSolid3DContainment<T>>::contains_point(
        sphere,
        (point.x(), point.y(), point.z()),
    ) {
        T::ZERO
    } else {
        <SphericalSolid3D<T> as SphericalSolid3DDistance<T>>::distance_to_point(
            sphere,
            (point.x(), point.y(), point.z()),
        )
    }
}

/// 逆向きラッパー: point-spherical_solid
pub fn point3d_spherical_solid3d_distance<T: Scalar>(
    point: &Point3D<T>,
    sphere: &SphericalSolid3D<T>,
) -> T {
    spherical_solid3d_point3d_distance(sphere, point)
}

/// Ray3D-点 間の最短距離（Ray の有効範囲を考慮）
pub fn ray3d_point3d_distance<T: Scalar>(ray: &Ray3D<T>, point: &Point3D<T>) -> T {
    ray.distance_to_point(point)
}

/// 逆向きラッパー: point-ray
pub fn point3d_ray3d_distance<T: Scalar>(point: &Point3D<T>, ray: &Ray3D<T>) -> T {
    ray.distance_to_point(point)
}

/// Arc3D-点 間の最短距離
pub fn arc3d_point3d_distance<T: Scalar>(arc: &Arc3D<T>, point: &Point3D<T>) -> T {
    <Arc3D<T> as Arc3DDistance<T>>::distance_to_point(arc, (point.x(), point.y(), point.z()))
}

/// 逆向きラッパー: point-arc
pub fn point3d_arc3d_distance<T: Scalar>(point: &Point3D<T>, arc: &Arc3D<T>) -> T {
    arc3d_point3d_distance(arc, point)
}

/// TorusSurface3D-点 間の最短距離
pub fn torus_surface3d_point3d_distance<T: Scalar>(
    torus: &TorusSurface3D<T>,
    point: &Point3D<T>,
) -> T {
    <TorusSurface3D<T> as TorusSurface3DDistance<T>>::distance_to_point(
        torus,
        (point.x(), point.y(), point.z()),
    )
}

/// 逆向きラッパー: point-torus_surface
pub fn point3d_torus_surface3d_distance<T: Scalar>(
    point: &Point3D<T>,
    torus: &TorusSurface3D<T>,
) -> T {
    torus_surface3d_point3d_distance(torus, point)
}

/// Circle3D-点 間の最短距離（円周への3D空間での距離）
pub fn circle3d_point3d_distance<T: Scalar>(circle: &Circle3D<T>, point: &Point3D<T>) -> T {
    circle.distance_to_point_3d(*point)
}

/// 逆向きラッパー: point-circle
pub fn point3d_circle3d_distance<T: Scalar>(point: &Point3D<T>, circle: &Circle3D<T>) -> T {
    circle.distance_to_point_3d(*point)
}

/// Ellipse3D-点 間の最短距離
pub fn ellipse3d_point3d_distance<T: Scalar + From<f64>>(
    ellipse: &Ellipse3D<T>,
    point: &Point3D<T>,
) -> T {
    <Ellipse3D<T> as Ellipse3DDistance<T>>::distance_to_point(
        ellipse,
        (point.x(), point.y(), point.z()),
    )
}

/// 逆向きラッパー: point-ellipse
pub fn point3d_ellipse3d_distance<T: Scalar + From<f64>>(
    point: &Point3D<T>,
    ellipse: &Ellipse3D<T>,
) -> T {
    ellipse3d_point3d_distance(ellipse, point)
}

/// CylindricalSolid3D-点 間の最短距離
pub fn cylindrical_solid3d_point3d_distance<T: Scalar>(
    cyl: &CylindricalSolid3D<T>,
    point: &Point3D<T>,
) -> T {
    <CylindricalSolid3D<T> as CylindricalSolid3DDistance<T>>::distance_to_point(
        cyl,
        (point.x(), point.y(), point.z()),
    )
}

/// 逆向きラッパー: point-cylindrical_solid
pub fn point3d_cylindrical_solid3d_distance<T: Scalar>(
    point: &Point3D<T>,
    cyl: &CylindricalSolid3D<T>,
) -> T {
    cylindrical_solid3d_point3d_distance(cyl, point)
}

/// CylindricalSurface3D-点 間の最短距離
pub fn cylindrical_surface3d_point3d_distance<T: Scalar>(
    cyl: &CylindricalSurface3D<T>,
    point: &Point3D<T>,
) -> T {
    <CylindricalSurface3D<T> as CylindricalSurface3DDistance<T>>::distance_to_point(
        cyl,
        (point.x(), point.y(), point.z()),
    )
}

/// 逆向きラッパー: point-cylindrical_surface
pub fn point3d_cylindrical_surface3d_distance<T: Scalar>(
    point: &Point3D<T>,
    cyl: &CylindricalSurface3D<T>,
) -> T {
    cylindrical_surface3d_point3d_distance(cyl, point)
}

/// EllipsoidalSolid3D-点 間の最短距離（内部点は 0）
pub fn ellipsoidal_solid3d_point3d_distance<T: Scalar>(
    ellipsoid: &EllipsoidalSolid3D<T>,
    point: &Point3D<T>,
) -> T {
    if <EllipsoidalSolid3D<T> as EllipsoidalSolid3DContainment<T>>::contains_point(
        ellipsoid,
        (point.x(), point.y(), point.z()),
    ) {
        T::ZERO
    } else {
        <EllipsoidalSolid3D<T> as EllipsoidalSolid3DDistance<T>>::distance_to_surface(
            ellipsoid,
            (point.x(), point.y(), point.z()),
        )
    }
}

/// 逆向きラッパー: point-ellipsoidal_solid
pub fn point3d_ellipsoidal_solid3d_distance<T: Scalar>(
    point: &Point3D<T>,
    ellipsoid: &EllipsoidalSolid3D<T>,
) -> T {
    ellipsoidal_solid3d_point3d_distance(ellipsoid, point)
}

/// EllipsoidalSurface3D-点 間の最短距離
pub fn ellipsoidal_surface3d_point3d_distance<T: Scalar>(
    ellipsoid: &EllipsoidalSurface3D<T>,
    point: &Point3D<T>,
) -> T {
    <EllipsoidalSurface3D<T> as EllipsoidalSurface3DDistance<T>>::distance_to_point(
        ellipsoid,
        (point.x(), point.y(), point.z()),
    )
}

/// 逆向きラッパー: point-ellipsoidal_surface
pub fn point3d_ellipsoidal_surface3d_distance<T: Scalar>(
    point: &Point3D<T>,
    ellipsoid: &EllipsoidalSurface3D<T>,
) -> T {
    ellipsoidal_surface3d_point3d_distance(ellipsoid, point)
}

/// ConicalSurface3D-点 間の最短距離
pub fn conical_surface3d_point3d_distance<T: Scalar>(
    cone: &ConicalSurface3D<T>,
    point: &Point3D<T>,
) -> T {
    <ConicalSurface3D<T> as ConicalSurface3DDistance<T>>::distance_to_point(
        cone,
        (point.x(), point.y(), point.z()),
    )
}

/// 逆向きラッパー: point-conical_surface
pub fn point3d_conical_surface3d_distance<T: Scalar>(
    point: &Point3D<T>,
    cone: &ConicalSurface3D<T>,
) -> T {
    conical_surface3d_point3d_distance(cone, point)
}

/// TorusSolid3D-点 間の最短距離（内部点は 0）
pub fn torus_solid3d_point3d_distance<T: Scalar>(torus: &TorusSolid3D<T>, point: &Point3D<T>) -> T {
    if <TorusSolid3D<T> as TorusSolid3DContainment<T>>::contains_point(
        torus,
        (point.x(), point.y(), point.z()),
    ) {
        T::ZERO
    } else {
        <TorusSolid3D<T> as TorusSolid3DDistance<T>>::distance_to_point(
            torus,
            (point.x(), point.y(), point.z()),
        )
    }
}

/// 逆向きラッパー: point-torus_solid
pub fn point3d_torus_solid3d_distance<T: Scalar>(point: &Point3D<T>, torus: &TorusSolid3D<T>) -> T {
    torus_solid3d_point3d_distance(torus, point)
}

/// Triangle3D-点 間の最短距離
pub fn triangle3d_point3d_distance<T: Scalar>(triangle: &Triangle3D<T>, point: &Point3D<T>) -> T {
    triangle.distance_to_point(point)
}

/// 逆向きラッパー: point-triangle
pub fn point3d_triangle3d_distance<T: Scalar>(point: &Point3D<T>, triangle: &Triangle3D<T>) -> T {
    triangle3d_point3d_distance(triangle, point)
}

/// TriangleMesh3D-点 間の最短距離
pub fn triangle_mesh3d_point3d_distance<T: Scalar>(
    mesh: &TriangleMesh3D<T>,
    point: &Point3D<T>,
) -> T {
    (0..mesh.triangle_count())
        .filter_map(|index| {
            mesh.triangle(index)
                .map(|triangle| triangle.distance_to_point(point))
        })
        .reduce(|best, distance| best.min(distance))
        .unwrap_or(T::INFINITY)
}

/// 逆向きラッパー: point-triangle_mesh
pub fn point3d_triangle_mesh3d_distance<T: Scalar>(
    point: &Point3D<T>,
    mesh: &TriangleMesh3D<T>,
) -> T {
    triangle_mesh3d_point3d_distance(mesh, point)
}

#[cfg(test)]
mod tests {
    use super::*;
    use analysis::test_constants;

    fn standard_distance_tol() -> f64 {
        test_constants::DISTANCE_TOLERANCE_F64
    }

    #[test]
    fn infinite_line3d_distance_parallel_lines() {
        let tol = standard_distance_tol();
        let line_a = InfiniteLine3D::from_two_points(
            Point3D::new(0.0, 0.0, 0.0),
            Point3D::new(1.0, 0.0, 0.0),
        )
        .unwrap();
        let line_b = InfiniteLine3D::from_two_points(
            Point3D::new(0.0, 2.0, 0.0),
            Point3D::new(1.0, 2.0, 0.0),
        )
        .unwrap();

        let d = infinite_line3d_infinite_line3d_distance(&line_a, &line_b);
        assert!((d - 2.0).abs() < tol);
    }

    #[test]
    fn infinite_line3d_distance_intersecting_lines() {
        let tol = standard_distance_tol();
        let line_a = InfiniteLine3D::from_two_points(
            Point3D::new(-1.0, 0.0, 0.0),
            Point3D::new(1.0, 0.0, 0.0),
        )
        .unwrap();
        let line_b = InfiniteLine3D::from_two_points(
            Point3D::new(0.0, -1.0, 0.0),
            Point3D::new(0.0, 1.0, 0.0),
        )
        .unwrap();

        let d = infinite_line3d_infinite_line3d_distance(&line_a, &line_b);
        assert!(d.abs() < tol);
    }

    #[test]
    fn cylindrical_surface_point_boundary_guard_keeps_collision_and_intersection_on_distance_entrypoint(
    ) {
        const CYLINDRICAL_SURFACE_DIRECT_UFCS: &str =
            "<CylindricalSurface3D<T> as CylindricalSurface3DDistance<T>>::distance_to_point";
        const CYLINDRICAL_SURFACE_POINT_ENTRYPOINT: &str =
            "crate::distance::cylindrical_surface3d_point3d_distance";

        fn section<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
            let start_index = source
                .find(start)
                .unwrap_or_else(|| panic!("missing start marker: {start}"));
            let tail = &source[start_index..];
            let end_index = tail
                .find(end)
                .unwrap_or_else(|| panic!("missing end marker: {end}"));
            &tail[..end_index]
        }

        let collision_source = include_str!("../collision/primitive_3d.rs");
        let intersection_source =
            include_str!("../intersection/primitive_3d/cylindrical_and_conical_family.rs");
        let collision_cylindrical_surface_point_section = section(
            collision_source,
            "pub fn cylindrical_surface3d_point3d_collides",
            "pub fn cylindrical_surface3d_plane3d_collides",
        );
        let intersection_cylindrical_surface_point_section = section(
            intersection_source,
            "fn cylindrical_surface3d_point3d_intersection_raw",
            "fn cylindrical_surface3d_plane3d_intersection_raw",
        );

        assert!(
            collision_cylindrical_surface_point_section
                .contains(CYLINDRICAL_SURFACE_POINT_ENTRYPOINT),
            "collision/primitive_3d.rs should route cylindrical surface point checks through the distance entrypoint"
        );
        assert!(
            intersection_cylindrical_surface_point_section
                .contains(CYLINDRICAL_SURFACE_POINT_ENTRYPOINT),
            "intersection/primitive_3d.rs should route cylindrical surface point checks through the distance entrypoint"
        );
        assert!(
            !collision_cylindrical_surface_point_section.contains(CYLINDRICAL_SURFACE_DIRECT_UFCS),
            "collision/primitive_3d.rs must not call CylindricalSurface3DDistance::distance_to_point directly"
        );
        assert!(
            !intersection_cylindrical_surface_point_section
                .contains(CYLINDRICAL_SURFACE_DIRECT_UFCS),
            "intersection/primitive_3d.rs must not call CylindricalSurface3DDistance::distance_to_point directly"
        );
    }

    #[test]
    fn cylindrical_surface_pair_guard_keeps_intersection_on_distance_entrypoint() {
        const CYLINDRICAL_SURFACE_DIRECT_UFCS: &str =
            "<CylindricalSurface3D<T> as CylindricalSurface3DDistance<T>>::distance_to_point";
        const CYLINDRICAL_SURFACE_POINT_ENTRYPOINT: &str =
            "crate::distance::cylindrical_surface3d_point3d_distance";

        fn section<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
            let start_index = source
                .find(start)
                .unwrap_or_else(|| panic!("missing start marker: {start}"));
            let tail = &source[start_index..];
            let end_index = tail
                .find(end)
                .unwrap_or_else(|| panic!("missing end marker: {end}"));
            &tail[..end_index]
        }

        let intersection_source =
            include_str!("../intersection/primitive_3d/cylindrical_and_conical_family.rs");
        let intersection_cylindrical_surface_pair_section = section(
            intersection_source,
            "fn cylindrical_surface3d_cylindrical_surface3d_intersection_raw",
            "fn conical_solid3d_point3d_intersection_raw",
        );

        assert!(
            intersection_cylindrical_surface_pair_section.contains(CYLINDRICAL_SURFACE_POINT_ENTRYPOINT),
            "intersection/primitive_3d.rs should route cylindrical surface pair center checks through the distance entrypoint"
        );
        assert!(
            !intersection_cylindrical_surface_pair_section.contains(CYLINDRICAL_SURFACE_DIRECT_UFCS),
            "intersection/primitive_3d.rs must not call CylindricalSurface3DDistance::distance_to_point directly in the cylindrical surface pair section"
        );
    }

    #[test]
    fn cylindrical_solid_point_boundary_guard_keeps_collision_on_distance_entrypoint() {
        const CYLINDRICAL_SOLID_DIRECT_UFCS: &str =
            "<CylindricalSolid3D<T> as CylindricalSolid3DDistance<T>>::distance_to_point";
        const CYLINDRICAL_SOLID_POINT_ENTRYPOINT: &str =
            "crate::distance::cylindrical_solid3d_point3d_distance";

        fn section<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
            let start_index = source
                .find(start)
                .unwrap_or_else(|| panic!("missing start marker: {start}"));
            let tail = &source[start_index..];
            let end_index = tail
                .find(end)
                .unwrap_or_else(|| panic!("missing end marker: {end}"));
            &tail[..end_index]
        }

        let collision_source = include_str!("../collision/primitive_3d.rs");
        let collision_cylindrical_solid_point_section = section(
            collision_source,
            "pub fn cylindrical_solid3d_point3d_collides",
            "pub fn cylindrical_solid3d_plane3d_collides",
        );

        assert!(
            collision_cylindrical_solid_point_section.contains(CYLINDRICAL_SOLID_POINT_ENTRYPOINT),
            "collision/primitive_3d.rs should route cylindrical solid point checks through the distance entrypoint"
        );
        assert!(
            !collision_cylindrical_solid_point_section.contains(CYLINDRICAL_SOLID_DIRECT_UFCS),
            "collision/primitive_3d.rs must not call CylindricalSolid3DDistance::distance_to_point directly"
        );
    }

    #[test]
    fn ellipse_point_boundary_guard_keeps_collision_and_intersection_on_distance_entrypoint() {
        const ELLIPSE_DIRECT_UFCS: &str =
            "<Ellipse3D<T> as Ellipse3DDistance<T>>::distance_to_point";
        const ELLIPSE_POINT_ENTRYPOINT: &str = "crate::distance::ellipse3d_point3d_distance";

        fn section<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
            let start_index = source
                .find(start)
                .unwrap_or_else(|| panic!("missing start marker: {start}"));
            let tail = &source[start_index..];
            let end_index = tail
                .find(end)
                .unwrap_or_else(|| panic!("missing end marker: {end}"));
            &tail[..end_index]
        }

        let collision_source = include_str!("../collision/primitive_3d.rs");
        let intersection_source = include_str!("../intersection/primitive_3d/circular_family.rs");
        let collision_ellipse_point_section = section(
            collision_source,
            "pub fn ellipse3d_point3d_collides",
            "pub fn ellipse3d_plane3d_collides",
        );
        let intersection_ellipse_point_section = section(
            intersection_source,
            "fn ellipse3d_point3d_intersection_raw",
            "fn ellipse3d_plane3d_intersection_raw",
        );

        assert!(
            collision_ellipse_point_section.contains(ELLIPSE_POINT_ENTRYPOINT),
            "collision/primitive_3d.rs should route ellipse point checks through the distance entrypoint"
        );
        assert!(
            intersection_ellipse_point_section.contains(ELLIPSE_POINT_ENTRYPOINT),
            "intersection/primitive_3d.rs should route ellipse point checks through the distance entrypoint"
        );
        assert!(
            !collision_ellipse_point_section.contains(ELLIPSE_DIRECT_UFCS),
            "collision/primitive_3d.rs must not call Ellipse3DDistance::distance_to_point directly"
        );
        assert!(
            !intersection_ellipse_point_section.contains(ELLIPSE_DIRECT_UFCS),
            "intersection/primitive_3d.rs must not call Ellipse3DDistance::distance_to_point directly"
        );
    }

    #[test]
    fn arc_point_boundary_guard_keeps_collision_and_intersection_on_distance_entrypoint() {
        const ARC_DIRECT_UFCS: &str = "<Arc3D<T> as Arc3DDistance<T>>::distance_to_point";
        const ARC_POINT_ENTRYPOINT: &str = "crate::distance::arc3d_point3d_distance";

        fn section<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
            let start_index = source
                .find(start)
                .unwrap_or_else(|| panic!("missing start marker: {start}"));
            let tail = &source[start_index..];
            let end_index = tail
                .find(end)
                .unwrap_or_else(|| panic!("missing end marker: {end}"));
            &tail[..end_index]
        }

        let collision_source = include_str!("../collision/primitive_3d.rs");
        let intersection_source = include_str!("../intersection/primitive_3d/circular_family.rs");
        let collision_arc_point_section = section(
            collision_source,
            "pub fn arc3d_point3d_collides",
            "pub fn circle3d_point3d_collides",
        );
        let intersection_arc_point_section = section(
            intersection_source,
            "fn arc3d_point3d_intersection_raw",
            "fn circle3d_point3d_intersection_raw",
        );

        assert!(
            collision_arc_point_section.contains(ARC_POINT_ENTRYPOINT),
            "collision/primitive_3d.rs should route arc point checks through the distance entrypoint"
        );
        assert!(
            intersection_arc_point_section.contains(ARC_POINT_ENTRYPOINT),
            "intersection/primitive_3d.rs should route arc point checks through the distance entrypoint"
        );
        assert!(
            !collision_arc_point_section.contains(ARC_DIRECT_UFCS),
            "collision/primitive_3d.rs must not call Arc3DDistance::distance_to_point directly"
        );
        assert!(
            !intersection_arc_point_section.contains(ARC_DIRECT_UFCS),
            "intersection/primitive_3d.rs must not call Arc3DDistance::distance_to_point directly"
        );
    }

    #[test]
    fn circle_point_boundary_guard_keeps_collision_and_intersection_on_distance_entrypoint() {
        const CIRCLE_DIRECT_DISTANCE: &str = "circle.distance_to_point_3d";
        const CIRCLE_DIRECT_CONTAINS: &str = "circle.contains_point_3d";
        const CIRCLE_POINT_ENTRYPOINT: &str = "crate::distance::circle3d_point3d_distance";

        fn section<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
            let start_index = source
                .find(start)
                .unwrap_or_else(|| panic!("missing start marker: {start}"));
            let tail = &source[start_index..];
            let end_index = tail
                .find(end)
                .unwrap_or_else(|| panic!("missing end marker: {end}"));
            &tail[..end_index]
        }

        let collision_source = include_str!("../collision/primitive_3d.rs");
        let intersection_source = include_str!("../intersection/primitive_3d/circular_family.rs");
        let collision_circle_point_section = section(
            collision_source,
            "pub fn circle3d_point3d_collides",
            "pub fn plane3d_point3d_collides",
        );
        let intersection_circle_point_section = section(
            intersection_source,
            "fn circle3d_point3d_intersection_raw",
            "fn circle3d_line_segment3d_intersection_raw",
        );

        assert!(
            collision_circle_point_section.contains(CIRCLE_POINT_ENTRYPOINT),
            "collision/primitive_3d.rs should route circle point checks through the distance entrypoint"
        );
        assert!(
            intersection_circle_point_section.contains(CIRCLE_POINT_ENTRYPOINT),
            "intersection/primitive_3d.rs should route circle point checks through the distance entrypoint"
        );
        assert!(
            !collision_circle_point_section.contains(CIRCLE_DIRECT_DISTANCE),
            "collision/primitive_3d.rs must not call circle.distance_to_point_3d directly"
        );
        assert!(
            !intersection_circle_point_section.contains(CIRCLE_DIRECT_DISTANCE),
            "intersection/primitive_3d.rs must not call circle.distance_to_point_3d directly"
        );
        assert!(
            !intersection_circle_point_section.contains(CIRCLE_DIRECT_CONTAINS),
            "intersection/primitive_3d.rs must not call circle.contains_point_3d directly"
        );
    }

    #[test]
    fn ellipsoidal_solid_point_boundary_guard_keeps_collision_and_intersection_on_distance_entrypoint(
    ) {
        const ELLIPSOIDAL_SOLID_DIRECT_DISTANCE: &str = "ellipsoid.distance_to_surface";
        const ELLIPSOIDAL_SOLID_DIRECT_CONTAINS: &str = "ellipsoid.contains_point";
        const ELLIPSOIDAL_SOLID_POINT_ENTRYPOINT: &str =
            "crate::distance::ellipsoidal_solid3d_point3d_distance";

        fn section<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
            let start_index = source
                .find(start)
                .unwrap_or_else(|| panic!("missing start marker: {start}"));
            let tail = &source[start_index..];
            let end_index = tail
                .find(end)
                .unwrap_or_else(|| panic!("missing end marker: {end}"));
            &tail[..end_index]
        }

        let collision_source = include_str!("../collision/primitive_3d.rs");
        let intersection_source =
            include_str!("../intersection/primitive_3d/spherical_and_quadric_family.rs");
        let collision_ellipsoidal_solid_point_section = section(
            collision_source,
            "pub fn ellipsoidal_solid3d_point3d_collides",
            "pub fn ellipsoidal_surface3d_point3d_collides",
        );
        let intersection_ellipsoidal_solid_point_section = section(
            intersection_source,
            "fn ellipsoidal_solid3d_point3d_intersection_raw",
            "fn ellipsoidal_surface3d_point3d_intersection_raw",
        );

        assert!(
            collision_ellipsoidal_solid_point_section.contains(ELLIPSOIDAL_SOLID_POINT_ENTRYPOINT),
            "collision/primitive_3d.rs should route ellipsoidal solid point checks through the distance entrypoint"
        );
        assert!(
            intersection_ellipsoidal_solid_point_section.contains(ELLIPSOIDAL_SOLID_POINT_ENTRYPOINT),
            "intersection/primitive_3d.rs should route ellipsoidal solid point checks through the distance entrypoint"
        );
        assert!(
            !collision_ellipsoidal_solid_point_section.contains(ELLIPSOIDAL_SOLID_DIRECT_DISTANCE),
            "collision/primitive_3d.rs must not call ellipsoid.distance_to_surface directly"
        );
        assert!(
            !collision_ellipsoidal_solid_point_section.contains(ELLIPSOIDAL_SOLID_DIRECT_CONTAINS),
            "collision/primitive_3d.rs must not call ellipsoid.contains_point directly"
        );
        assert!(
            !intersection_ellipsoidal_solid_point_section
                .contains(ELLIPSOIDAL_SOLID_DIRECT_CONTAINS),
            "intersection/primitive_3d.rs must not call ellipsoid.contains_point directly"
        );
    }

    #[test]
    fn ellipsoidal_surface_point_boundary_guard_keeps_collision_and_intersection_on_distance_entrypoint(
    ) {
        const ELLIPSOIDAL_SURFACE_DIRECT_CONTAINS: &str = "ellipsoid.contains_point";
        const ELLIPSOIDAL_SURFACE_POINT_ENTRYPOINT: &str =
            "crate::distance::ellipsoidal_surface3d_point3d_distance";

        fn section<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
            let start_index = source
                .find(start)
                .unwrap_or_else(|| panic!("missing start marker: {start}"));
            let tail = &source[start_index..];
            let end_index = tail
                .find(end)
                .unwrap_or_else(|| panic!("missing end marker: {end}"));
            &tail[..end_index]
        }

        let collision_source = include_str!("../collision/primitive_3d.rs");
        let intersection_source =
            include_str!("../intersection/primitive_3d/spherical_and_quadric_family.rs");
        let collision_ellipsoidal_surface_point_section = section(
            collision_source,
            "pub fn ellipsoidal_surface3d_point3d_collides",
            "pub fn torus_solid3d_point3d_collides",
        );
        let intersection_ellipsoidal_surface_point_section = section(
            intersection_source,
            "fn ellipsoidal_surface3d_point3d_intersection_raw",
            "fn spherical_solid3d_point3d_intersection_raw",
        );

        assert!(
            collision_ellipsoidal_surface_point_section.contains(ELLIPSOIDAL_SURFACE_POINT_ENTRYPOINT),
            "collision/primitive_3d.rs should route ellipsoidal surface point checks through the distance entrypoint"
        );
        assert!(
            intersection_ellipsoidal_surface_point_section.contains(ELLIPSOIDAL_SURFACE_POINT_ENTRYPOINT),
            "intersection/primitive_3d.rs should route ellipsoidal surface point checks through the distance entrypoint"
        );
        assert!(
            !collision_ellipsoidal_surface_point_section
                .contains(ELLIPSOIDAL_SURFACE_DIRECT_CONTAINS),
            "collision/primitive_3d.rs must not call ellipsoid.contains_point directly"
        );
        assert!(
            !intersection_ellipsoidal_surface_point_section
                .contains(ELLIPSOIDAL_SURFACE_DIRECT_CONTAINS),
            "intersection/primitive_3d.rs must not call ellipsoid.contains_point directly"
        );
    }

    #[test]
    fn conical_surface_point_boundary_guard_keeps_intersection_on_distance_entrypoint() {
        const CONICAL_SURFACE_DIRECT_CONTAINS: &str = "cone.contains_point";
        const CONICAL_SURFACE_POINT_ENTRYPOINT: &str =
            "crate::distance::conical_surface3d_point3d_distance";

        fn section<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
            let start_index = source
                .find(start)
                .unwrap_or_else(|| panic!("missing start marker: {start}"));
            let tail = &source[start_index..];
            let end_index = tail
                .find(end)
                .unwrap_or_else(|| panic!("missing end marker: {end}"));
            &tail[..end_index]
        }

        let intersection_source =
            include_str!("../intersection/primitive_3d/cylindrical_and_conical_family.rs");
        let intersection_conical_surface_point_section = section(
            intersection_source,
            "fn conical_surface3d_point3d_intersection_raw",
            "pub fn conical_surface3d_point3d_intersection",
        );

        assert!(
            intersection_conical_surface_point_section.contains(CONICAL_SURFACE_POINT_ENTRYPOINT),
            "intersection/primitive_3d.rs should route conical surface point checks through the distance entrypoint"
        );
        assert!(
            !intersection_conical_surface_point_section.contains(CONICAL_SURFACE_DIRECT_CONTAINS),
            "intersection/primitive_3d.rs must not call cone.contains_point directly"
        );
    }

    #[test]
    fn spherical_solid_point_boundary_guard_keeps_collision_and_intersection_on_distance_entrypoint(
    ) {
        const SPHERICAL_SOLID_DIRECT_DISTANCE: &str = "sphere.distance_to_surface";
        const SPHERICAL_SOLID_DIRECT_CONTAINS: &str = "sphere.contains_point";
        const SPHERICAL_SOLID_POINT_ENTRYPOINT: &str =
            "crate::distance::spherical_solid3d_point3d_distance";

        fn section<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
            let start_index = source
                .find(start)
                .unwrap_or_else(|| panic!("missing start marker: {start}"));
            let tail = &source[start_index..];
            let end_index = tail
                .find(end)
                .unwrap_or_else(|| panic!("missing end marker: {end}"));
            &tail[..end_index]
        }

        let collision_source = include_str!("../collision/primitive_3d.rs");
        let intersection_source =
            include_str!("../intersection/primitive_3d/spherical_and_quadric_family.rs");
        let collision_spherical_solid_point_section = section(
            collision_source,
            "pub fn spherical_solid3d_point3d_collides",
            "pub fn cylindrical_solid3d_point3d_collides",
        );
        let intersection_spherical_solid_point_section = section(
            intersection_source,
            "fn spherical_solid3d_point3d_intersection_raw",
            "fn spherical_solid3d_line3d_intersection_raw",
        );

        assert!(
            collision_spherical_solid_point_section.contains(SPHERICAL_SOLID_POINT_ENTRYPOINT),
            "collision/primitive_3d.rs should route spherical solid point checks through the distance entrypoint"
        );
        assert!(
            intersection_spherical_solid_point_section.contains(SPHERICAL_SOLID_POINT_ENTRYPOINT),
            "intersection/primitive_3d.rs should route spherical solid point checks through the distance entrypoint"
        );
        assert!(
            !collision_spherical_solid_point_section.contains(SPHERICAL_SOLID_DIRECT_DISTANCE),
            "collision/primitive_3d.rs must not call sphere.distance_to_surface directly"
        );
        assert!(
            !intersection_spherical_solid_point_section.contains(SPHERICAL_SOLID_DIRECT_CONTAINS),
            "intersection/primitive_3d.rs must not call sphere.contains_point directly"
        );
    }

    #[test]
    fn torus_solid_point_boundary_guard_keeps_collision_and_intersection_on_distance_entrypoint() {
        const TORUS_SOLID_DIRECT_CONTAINS: &str = "torus.contains_point";
        const TORUS_SOLID_POINT_ENTRYPOINT: &str =
            "crate::distance::torus_solid3d_point3d_distance";

        fn section<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
            let start_index = source
                .find(start)
                .unwrap_or_else(|| panic!("missing start marker: {start}"));
            let tail = &source[start_index..];
            let end_index = tail
                .find(end)
                .unwrap_or_else(|| panic!("missing end marker: {end}"));
            &tail[..end_index]
        }

        let collision_source = include_str!("../collision/primitive_3d.rs");
        let intersection_source =
            include_str!("../intersection/primitive_3d/spherical_and_quadric_family.rs");
        let collision_torus_solid_point_section = section(
            collision_source,
            "pub fn torus_solid3d_point3d_collides",
            "pub fn torus_surface3d_point3d_collides",
        );
        let intersection_torus_solid_point_section = section(
            intersection_source,
            "fn torus_solid3d_point3d_intersection_raw",
            "fn torus_surface3d_point3d_intersection_raw",
        );

        assert!(
            collision_torus_solid_point_section.contains(TORUS_SOLID_POINT_ENTRYPOINT),
            "collision/primitive_3d.rs should route torus solid point checks through the distance entrypoint"
        );
        assert!(
            intersection_torus_solid_point_section.contains(TORUS_SOLID_POINT_ENTRYPOINT),
            "intersection/primitive_3d.rs should route torus solid point checks through the distance entrypoint"
        );
        assert!(
            !collision_torus_solid_point_section.contains(TORUS_SOLID_DIRECT_CONTAINS),
            "collision/primitive_3d.rs must not call torus.contains_point directly"
        );
        assert!(
            !intersection_torus_solid_point_section.contains(TORUS_SOLID_DIRECT_CONTAINS),
            "intersection/primitive_3d.rs must not call torus.contains_point directly"
        );
    }

    #[test]
    fn torus_surface_point_boundary_guard_keeps_collision_and_intersection_on_distance_entrypoint()
    {
        const TORUS_SURFACE_DIRECT_UFCS: &str =
            "<TorusSurface3D<T> as TorusSurface3DDistance<T>>::distance_to_point";
        const TORUS_SURFACE_POINT_ENTRYPOINT: &str =
            "crate::distance::torus_surface3d_point3d_distance";

        fn section<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
            let start_index = source
                .find(start)
                .unwrap_or_else(|| panic!("missing start marker: {start}"));
            let tail = &source[start_index..];
            let end_index = tail
                .find(end)
                .unwrap_or_else(|| panic!("missing end marker: {end}"));
            &tail[..end_index]
        }

        let collision_source = include_str!("../collision/primitive_3d.rs");
        let intersection_source =
            include_str!("../intersection/primitive_3d/spherical_and_quadric_family.rs");
        let collision_torus_surface_point_section = section(
            collision_source,
            "pub fn torus_surface3d_point3d_collides",
            "pub fn triangle3d_point3d_collides",
        );
        let intersection_torus_surface_point_section = section(
            intersection_source,
            "fn torus_surface3d_point3d_intersection_raw",
            "pub fn torus_surface3d_point3d_intersection",
        );

        assert!(
            collision_torus_surface_point_section.contains(TORUS_SURFACE_POINT_ENTRYPOINT),
            "collision/primitive_3d.rs should route torus surface point checks through the distance entrypoint"
        );
        assert!(
            intersection_torus_surface_point_section.contains(TORUS_SURFACE_POINT_ENTRYPOINT),
            "intersection/primitive_3d.rs should route torus surface point checks through the distance entrypoint"
        );
        assert!(
            !collision_torus_surface_point_section.contains(TORUS_SURFACE_DIRECT_UFCS),
            "collision/primitive_3d.rs must not call TorusSurface3DDistance::distance_to_point directly"
        );
        assert!(
            !intersection_torus_surface_point_section.contains(TORUS_SURFACE_DIRECT_UFCS),
            "intersection/primitive_3d.rs must not call TorusSurface3DDistance::distance_to_point directly"
        );
    }

    #[test]
    fn triangle_point_boundary_guard_keeps_collision_and_intersection_on_distance_entrypoint() {
        const TRIANGLE_POINT_ENTRYPOINT: &str = "crate::distance::triangle3d_point3d_distance";

        fn section<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
            let start_index = source
                .find(start)
                .unwrap_or_else(|| panic!("missing start marker: {start}"));
            let tail = &source[start_index..];
            let end_index = tail
                .find(end)
                .unwrap_or_else(|| panic!("missing end marker: {end}"));
            &tail[..end_index]
        }

        let collision_source = include_str!("../collision/primitive_3d.rs");
        let intersection_source =
            include_str!("../intersection/primitive_3d/planar_and_mesh_family.rs");
        let collision_triangle_point_section = section(
            collision_source,
            "pub fn triangle3d_point3d_collides",
            "pub fn triangle_mesh3d_point3d_collides",
        );
        let intersection_triangle_point_section = section(
            intersection_source,
            "fn triangle3d_point3d_intersection_raw",
            "fn triangle_mesh3d_point3d_intersection_raw",
        );

        assert!(
            collision_triangle_point_section.contains(TRIANGLE_POINT_ENTRYPOINT),
            "collision/primitive_3d.rs should route triangle point checks through the distance entrypoint"
        );
        assert!(
            intersection_triangle_point_section.contains(TRIANGLE_POINT_ENTRYPOINT),
            "intersection/primitive_3d.rs should route triangle point checks through the distance entrypoint"
        );
    }

    #[test]
    fn triangle_mesh_point_boundary_guard_keeps_collision_and_intersection_on_distance_entrypoint()
    {
        const TRIANGLE_MESH_POINT_ENTRYPOINT: &str =
            "crate::distance::triangle_mesh3d_point3d_distance";

        fn section<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
            let start_index = source
                .find(start)
                .unwrap_or_else(|| panic!("missing start marker: {start}"));
            let tail = &source[start_index..];
            let end_index = tail
                .find(end)
                .unwrap_or_else(|| panic!("missing end marker: {end}"));
            &tail[..end_index]
        }

        let collision_source = include_str!("../collision/primitive_3d.rs");
        let intersection_source =
            include_str!("../intersection/primitive_3d/planar_and_mesh_family.rs");
        let collision_triangle_mesh_point_section = section(
            collision_source,
            "pub fn triangle_mesh3d_point3d_collides",
            "pub fn arc3d_point3d_collides",
        );
        let intersection_triangle_mesh_point_section = section(
            intersection_source,
            "fn triangle_mesh3d_point3d_intersection_raw",
            "fn plane3d_point3d_intersection_raw",
        );

        assert!(
            collision_triangle_mesh_point_section.contains(TRIANGLE_MESH_POINT_ENTRYPOINT),
            "collision/primitive_3d.rs should route triangle mesh point checks through the distance entrypoint"
        );
        assert!(
            intersection_triangle_mesh_point_section.contains(TRIANGLE_MESH_POINT_ENTRYPOINT),
            "intersection/primitive_3d.rs should route triangle mesh point checks through the distance entrypoint"
        );
    }
}
