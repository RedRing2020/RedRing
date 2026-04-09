//! 3D Primitive distance algorithms
//!
//! `geo_primitives` に実装済みの各 3D 形状の距離計算メソッドを
//! `geo_algorithms` 層の公開 entrypoint としてまとめた薄いラッパー群。
//!
//! 命名規則: `{shape_a}_{shape_b}_distance`

use crate::{
    Circle3D, CylindricalSolid3D, CylindricalSurface3D, Ellipse3D, InfiniteLine3D, LineSegment3D,
    Plane3D, Point3D, Ray3D,
};
use geo_contracts::{
    CylindricalSolid3DDistance, CylindricalSurface3DDistance, Ellipse3DDistance, Scalar,
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

/// Ray3D-点 間の最短距離（Ray の有効範囲を考慮）
pub fn ray3d_point3d_distance<T: Scalar>(ray: &Ray3D<T>, point: &Point3D<T>) -> T {
    ray.distance_to_point(point)
}

/// 逆向きラッパー: point-ray
pub fn point3d_ray3d_distance<T: Scalar>(point: &Point3D<T>, ray: &Ray3D<T>) -> T {
    ray.distance_to_point(point)
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
        let intersection_source = include_str!("../intersection/primitive_3d.rs");
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
        let intersection_source = include_str!("../intersection/primitive_3d.rs");
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
}
