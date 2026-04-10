//! NURBS3D × Primitives 交点計算実装
//!
//! geo_nurbs と geo_primitives 間の交点計算を
//! geo_algorithms で集約して実装するモジュールです。
//!
//! NurbsCurve3D / NurbsSurface3D と各種3D Primitive の交点計算を統一します。
//!
//! ## 設計方針
//!
//! - NurbsCurve3D × Primitive（9ペア）
//! - NurbsSurface3D × Primitive（9ペア）の交点計算を提供
//! - collision 実装と対称性を保つ（同一ペア数・同一形状セット）
//! - orphan rules への対応は collision 側の Newtype パターンを参照
//! - 現実装では tolerance ベースの交点判定を提供する
//! - 一部 API は厳密交点ではなく代表点を返す
//! - `LineSegment*::start/end` は ideal endpoint 基準で解釈する
//! - 共通化や抽出の余地がある処理は今後の整理対象とする

use crate::{
    Circle3D, CylindricalSolid3D, EllipsoidalSolid3D, InfiniteLine3D, LineSegment3D, Plane3D,
    Point3D, Ray3D, SphericalSolid3D,
};
use geo_contracts::Scalar;
use geo_nurbs::{NurbsCurve3D, NurbsSurface3D};

// ── Helper ────────────────────────────────────────────────────────────────────

/// 交点条件の判定に基づき Point3D を返す
fn point_intersection_if<T: Scalar>(point: &Point3D<T>, condition: bool) -> Option<Point3D<T>> {
    if condition {
        Some(Point3D::new(point.x(), point.y(), point.z()))
    } else {
        None
    }
}

fn representative_curve_domain_start_point<T: Scalar>(curve: &NurbsCurve3D<T>) -> Point3D<T> {
    let (u_min, _) = curve.parameter_domain();
    let point = curve.evaluate_at(u_min);
    Point3D::new(point.x(), point.y(), point.z())
}

fn representative_surface_domain_start_point<T: Scalar>(surface: &NurbsSurface3D<T>) -> Point3D<T> {
    let ((u_min, _), (v_min, _)) = surface.parameter_domain();
    let point = surface.evaluate_at(u_min, v_min);
    Point3D::new(point.x(), point.y(), point.z())
}

pub fn nurbscurve3d_point3d_intersection<T: Scalar>(
    curve: &NurbsCurve3D<T>,
    point: &Point3D<T>,
    tolerance: T,
) -> Option<Point3D<T>> {
    let distance = crate::collision::nurbscurve3d_point3d_distance(curve, point);
    point_intersection_if(point, distance <= tolerance)
}

pub fn nurbscurve3d_line_segment3d_intersection<T: Scalar>(
    curve: &NurbsCurve3D<T>,
    segment: &LineSegment3D<T>,
    tolerance: T,
) -> Option<Point3D<T>> {
    let distance = crate::collision::nurbscurve3d_line_segment3d_distance(curve, segment);

    if distance <= tolerance {
        // 現実装では代表点として線分の ideal start endpoint を返す。
        Some(segment.start())
    } else {
        None
    }
}

pub fn nurbscurve3d_ray3d_intersection<T: Scalar>(
    curve: &NurbsCurve3D<T>,
    ray: &Ray3D<T>,
    tolerance: T,
) -> Option<Point3D<T>> {
    let distance = crate::collision::nurbscurve3d_ray3d_distance(curve, ray);

    if distance <= tolerance {
        // 現実装では代表点として Ray origin を返す。
        Some(ray.origin())
    } else {
        None
    }
}

pub fn nurbscurve3d_infinite_line3d_intersection<T: Scalar>(
    curve: &NurbsCurve3D<T>,
    line: &InfiniteLine3D<T>,
    tolerance: T,
) -> Option<Point3D<T>> {
    let distance = crate::collision::nurbscurve3d_infinite_line3d_distance(curve, line);

    if distance <= tolerance {
        // 現実装では代表点として InfiniteLine の基準点を返す。
        let (px, py, pz) = geo_contracts::InfiniteLine3DProperties::point(line);
        Some(Point3D::new(px, py, pz))
    } else {
        None
    }
}

pub fn nurbscurve3d_circle3d_intersection<T: Scalar>(
    curve: &NurbsCurve3D<T>,
    circle: &Circle3D<T>,
    tolerance: T,
) -> Option<Point3D<T>> {
    let distance = crate::collision::nurbscurve3d_circle3d_distance(curve, circle);

    if distance <= tolerance {
        // 現実装では代表点として Circle center を返す。
        let (cx, cy, cz) = geo_contracts::Circle3DProperties::center(circle);
        Some(Point3D::new(cx, cy, cz))
    } else {
        None
    }
}

pub fn nurbscurve3d_plane3d_intersection<T: Scalar>(
    curve: &NurbsCurve3D<T>,
    plane: &Plane3D<T>,
    tolerance: T,
) -> Option<Point3D<T>> {
    let distance = crate::collision::nurbscurve3d_plane3d_distance(curve, plane);

    if distance <= tolerance {
        // 現実装では代表点として Plane origin を返す。
        Some(plane.origin())
    } else {
        None
    }
}

pub fn nurbscurve3d_spherical_solid3d_intersection<T: Scalar>(
    curve: &NurbsCurve3D<T>,
    sphere: &SphericalSolid3D<T>,
    tolerance: T,
) -> Option<Point3D<T>> {
    let distance = crate::collision::nurbscurve3d_spherical_solid3d_distance(curve, sphere);

    if distance <= tolerance {
        // 現実装では代表点として NURBS curve の parameter domain 始端を返す。
        Some(representative_curve_domain_start_point(curve))
    } else {
        None
    }
}

pub fn nurbscurve3d_ellipsoidal_solid3d_intersection<T: Scalar>(
    curve: &NurbsCurve3D<T>,
    ellipsoid: &EllipsoidalSolid3D<T>,
    tolerance: T,
) -> Option<Point3D<T>> {
    let distance = crate::collision::nurbscurve3d_ellipsoidal_solid3d_distance(curve, ellipsoid);

    if distance <= tolerance {
        // 現実装では代表点として NURBS curve の parameter domain 始端を返す。
        Some(representative_curve_domain_start_point(curve))
    } else {
        None
    }
}

pub fn nurbscurve3d_cylindrical_solid3d_intersection<T: Scalar>(
    curve: &NurbsCurve3D<T>,
    cylinder: &CylindricalSolid3D<T>,
    tolerance: T,
) -> Option<Point3D<T>> {
    let distance = crate::collision::nurbscurve3d_cylindrical_solid3d_distance(curve, cylinder);

    if distance <= tolerance {
        // 現実装では代表点として NURBS curve の parameter domain 始端を返す。
        Some(representative_curve_domain_start_point(curve))
    } else {
        None
    }
}

pub fn nurbssurface3d_point3d_intersection<T: Scalar>(
    surface: &NurbsSurface3D<T>,
    point: &Point3D<T>,
    tolerance: T,
) -> Option<Point3D<T>> {
    let distance = crate::collision::nurbssurface3d_point3d_distance(surface, point);
    point_intersection_if(point, distance <= tolerance)
}

pub fn nurbssurface3d_plane3d_intersection<T: Scalar>(
    surface: &NurbsSurface3D<T>,
    plane: &Plane3D<T>,
    tolerance: T,
) -> Option<Point3D<T>> {
    let distance = crate::collision::nurbssurface3d_plane3d_distance(surface, plane);

    if distance <= tolerance {
        // 現実装では代表点として Plane origin を返す。
        Some(plane.origin())
    } else {
        None
    }
}

pub fn nurbssurface3d_ray3d_intersection<T: Scalar>(
    surface: &NurbsSurface3D<T>,
    ray: &Ray3D<T>,
    tolerance: T,
) -> Option<Point3D<T>> {
    let distance = crate::collision::nurbssurface3d_ray3d_distance(surface, ray);

    if distance <= tolerance {
        // 現実装では代表点として Ray origin を返す。
        Some(ray.origin())
    } else {
        None
    }
}

pub fn nurbssurface3d_line_segment3d_intersection<T: Scalar>(
    surface: &NurbsSurface3D<T>,
    segment: &LineSegment3D<T>,
    tolerance: T,
) -> Option<Point3D<T>> {
    let distance = crate::collision::nurbssurface3d_line_segment3d_distance(surface, segment);

    if distance <= tolerance {
        // 現実装では代表点として線分の ideal start endpoint を返す。
        Some(segment.start())
    } else {
        None
    }
}

pub fn nurbssurface3d_infinite_line3d_intersection<T: Scalar>(
    surface: &NurbsSurface3D<T>,
    line: &InfiniteLine3D<T>,
    tolerance: T,
) -> Option<Point3D<T>> {
    let distance = crate::collision::nurbssurface3d_infinite_line3d_distance(surface, line);

    if distance <= tolerance {
        // 現実装では代表点として InfiniteLine の基準点を返す。
        let (px, py, pz) = geo_contracts::InfiniteLine3DProperties::point(line);
        Some(Point3D::new(px, py, pz))
    } else {
        None
    }
}

pub fn nurbssurface3d_circle3d_intersection<T: Scalar>(
    surface: &NurbsSurface3D<T>,
    circle: &Circle3D<T>,
    tolerance: T,
) -> Option<Point3D<T>> {
    let distance = crate::collision::nurbssurface3d_circle3d_distance(surface, circle);

    if distance <= tolerance {
        // 現実装では代表点として Circle center を返す。
        let (cx, cy, cz) = geo_contracts::Circle3DProperties::center(circle);
        Some(Point3D::new(cx, cy, cz))
    } else {
        None
    }
}

pub fn nurbssurface3d_spherical_solid3d_intersection<T: Scalar>(
    surface: &NurbsSurface3D<T>,
    sphere: &SphericalSolid3D<T>,
    tolerance: T,
) -> Option<Point3D<T>> {
    let distance = crate::collision::nurbssurface3d_spherical_solid3d_distance(surface, sphere);

    if distance <= tolerance {
        // 現実装では代表点として surface parameter domain の始端評価点を返す。
        Some(representative_surface_domain_start_point(surface))
    } else {
        None
    }
}

pub fn nurbssurface3d_ellipsoidal_solid3d_intersection<T: Scalar>(
    surface: &NurbsSurface3D<T>,
    ellipsoid: &EllipsoidalSolid3D<T>,
    tolerance: T,
) -> Option<Point3D<T>> {
    let distance =
        crate::collision::nurbssurface3d_ellipsoidal_solid3d_distance(surface, ellipsoid);

    if distance <= tolerance {
        // 現実装では代表点として surface parameter domain の始端評価点を返す。
        Some(representative_surface_domain_start_point(surface))
    } else {
        None
    }
}

pub fn nurbssurface3d_cylindrical_solid3d_intersection<T: Scalar>(
    surface: &NurbsSurface3D<T>,
    cylinder: &CylindricalSolid3D<T>,
    tolerance: T,
) -> Option<Point3D<T>> {
    let distance = crate::collision::nurbssurface3d_cylindrical_solid3d_distance(surface, cylinder);

    if distance <= tolerance {
        // 現実装では代表点として surface parameter domain の始端評価点を返す。
        Some(representative_surface_domain_start_point(surface))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Direction3D, Vector3D};
    use analysis::test_constants;
    use geo_contracts::{NurbsCurve3DConstructor, NurbsSurface3DConstructor};

    const TEST_TOLERANCE: f64 = test_constants::DISTANCE_TOLERANCE_F64;

    fn create_test_curve<T: Scalar>() -> NurbsCurve3D<T> {
        use analysis::linalg::vector::vector3::Vector3;

        // 3次 NURBS 曲線
        let control_points = vec![
            Vector3::new(T::ZERO, T::ZERO, T::ZERO),
            Vector3::new(T::ONE, T::ZERO, T::ZERO),
            Vector3::new(T::ONE, T::ONE, T::ZERO),
            Vector3::new(T::ZERO, T::ONE, T::ZERO),
        ];

        let weights = Some(vec![T::ONE, T::ONE, T::ONE, T::ONE]);
        let knots = vec![
            T::ZERO,
            T::ZERO,
            T::ZERO,
            T::ZERO,
            T::ONE,
            T::ONE,
            T::ONE,
            T::ONE,
        ];

        let control_points_tuples = control_points
            .into_iter()
            .map(|v| (v.x(), v.y(), v.z()))
            .collect();
        <NurbsCurve3D<T> as NurbsCurve3DConstructor<T>>::new(
            3,
            knots,
            control_points_tuples,
            weights,
        )
        .unwrap()
    }

    fn create_test_surface<T: Scalar>() -> NurbsSurface3D<T> {
        <NurbsSurface3D<T> as NurbsSurface3DConstructor<T>>::unit_plane()
    }

    #[test]
    fn test_nurbscurve3d_point3d_intersection_on_curve() {
        let curve = create_test_curve::<f64>();
        let point = Point3D::new(0.0, 0.0, 0.0);
        let tolerance = TEST_TOLERANCE;

        let result = nurbscurve3d_point3d_intersection(&curve, &point, tolerance);
        assert!(result.is_some());
    }

    #[test]
    fn test_nurbscurve3d_point3d_no_intersection() {
        let curve = create_test_curve::<f64>();
        let point = Point3D::new(10.0, 10.0, 10.0);
        let tolerance = TEST_TOLERANCE;

        let result = nurbscurve3d_point3d_intersection(&curve, &point, tolerance);
        assert!(result.is_none());
    }

    #[test]
    fn test_nurbscurve3d_line_segment3d_intersection() {
        let curve = create_test_curve::<f64>();
        let segment =
            LineSegment3D::new(Point3D::new(-0.1, 0.0, 0.0), Point3D::new(0.1, 0.0, 0.0)).unwrap();
        let tolerance = 0.1;

        let result = nurbscurve3d_line_segment3d_intersection(&curve, &segment, tolerance);
        assert!(result.is_some());
    }

    #[test]
    fn test_nurbscurve3d_line_segment3d_intersection_returns_segment_start_as_representative_point()
    {
        let curve = create_test_curve::<f64>();
        let segment =
            LineSegment3D::new(Point3D::new(-0.1, 0.0, 0.0), Point3D::new(0.1, 0.0, 0.0)).unwrap();

        let result = nurbscurve3d_line_segment3d_intersection(&curve, &segment, 0.1);
        assert_eq!(result, Some(segment.start()));
    }

    #[test]
    fn test_nurbscurve3d_ray3d_intersection() {
        let curve = create_test_curve::<f64>();
        let ray = Ray3D::new(Point3D::new(0.0, 0.0, 0.0), Vector3D::new(1.0, 0.0, 0.0)).unwrap();
        let tolerance = TEST_TOLERANCE;

        let result = nurbscurve3d_ray3d_intersection(&curve, &ray, tolerance);
        assert!(result.is_some());
    }

    #[test]
    fn test_nurbscurve3d_ray3d_intersection_returns_ray_origin_as_representative_point() {
        let curve = create_test_curve::<f64>();
        let ray = Ray3D::new(Point3D::new(0.0, 0.0, 0.0), Vector3D::new(1.0, 0.0, 0.0)).unwrap();

        let result = nurbscurve3d_ray3d_intersection(&curve, &ray, TEST_TOLERANCE);
        assert_eq!(result, Some(ray.origin()));
    }

    #[test]
    fn test_nurbscurve3d_spherical_solid3d_intersection_returns_curve_domain_start_point() {
        let curve = create_test_curve::<f64>();
        let sphere = SphericalSolid3D::new(
            Point3D::new(0.5, 0.5, 0.0),
            Vector3D::new(0.0, 0.0, 1.0),
            Vector3D::new(1.0, 0.0, 0.0),
            2.0,
        )
        .unwrap();

        let result = nurbscurve3d_spherical_solid3d_intersection(&curve, &sphere, TEST_TOLERANCE);
        assert_eq!(
            result,
            Some(representative_curve_domain_start_point(&curve))
        );
    }

    #[test]
    fn test_nurbssurface3d_point3d_intersection_on_surface() {
        let surface = create_test_surface::<f64>();
        let point = Point3D::new(0.5, 0.5, 0.0);
        let result = nurbssurface3d_point3d_intersection(&surface, &point, TEST_TOLERANCE);
        assert!(result.is_some());
    }

    #[test]
    fn test_nurbssurface3d_plane3d_intersection_on_same_plane() {
        let surface = create_test_surface::<f64>();
        let plane = Plane3D::xy_plane(0.0);
        let result = nurbssurface3d_plane3d_intersection(&surface, &plane, TEST_TOLERANCE);
        assert!(result.is_some());
    }

    #[test]
    fn test_nurbssurface3d_plane3d_intersection_returns_plane_origin_as_representative_point() {
        let surface = create_test_surface::<f64>();
        let plane = Plane3D::xy_plane(0.0);

        let result = nurbssurface3d_plane3d_intersection(&surface, &plane, TEST_TOLERANCE);
        assert_eq!(result, Some(plane.origin()));
    }

    #[test]
    fn test_nurbssurface3d_ray3d_intersection_vertical_hit() {
        let surface = create_test_surface::<f64>();
        let ray = Ray3D::new(Point3D::new(0.5, 0.5, -1.0), Vector3D::new(0.0, 0.0, 1.0)).unwrap();
        let result = nurbssurface3d_ray3d_intersection(&surface, &ray, TEST_TOLERANCE);
        assert!(result.is_some());
    }

    #[test]
    fn test_nurbssurface3d_line_segment3d_intersection_vertical_hit() {
        let surface = create_test_surface::<f64>();
        let segment =
            LineSegment3D::new(Point3D::new(0.5, 0.5, -1.0), Point3D::new(0.5, 0.5, 1.0)).unwrap();
        let result = nurbssurface3d_line_segment3d_intersection(&surface, &segment, TEST_TOLERANCE);
        assert!(result.is_some());
    }

    #[test]
    fn test_nurbssurface3d_infinite_line3d_intersection_vertical_hit() {
        let surface = create_test_surface::<f64>();
        let line = InfiniteLine3D::from_two_points(
            Point3D::new(0.5, 0.5, -1.0),
            Point3D::new(0.5, 0.5, 1.0),
        )
        .unwrap();
        let result = nurbssurface3d_infinite_line3d_intersection(&surface, &line, TEST_TOLERANCE);
        assert!(result.is_some());
    }

    #[test]
    fn test_nurbssurface3d_circle3d_intersection_same_plane() {
        let surface = create_test_surface::<f64>();
        let circle = Circle3D::new(
            Point3D::new(0.5, 0.5, 0.0),
            Direction3D::from_vector(Vector3D::new(0.0, 0.0, 1.0)).unwrap(),
            0.25,
        )
        .unwrap();
        let result = nurbssurface3d_circle3d_intersection(&surface, &circle, TEST_TOLERANCE);
        assert!(result.is_some());
    }

    #[test]
    fn test_nurbssurface3d_spherical_solid3d_intersection_enclosing() {
        let surface = create_test_surface::<f64>();
        let sphere = SphericalSolid3D::new(
            Point3D::new(0.5, 0.5, 0.0),
            Vector3D::new(0.0, 0.0, 1.0),
            Vector3D::new(1.0, 0.0, 0.0),
            2.0,
        )
        .unwrap();
        let result =
            nurbssurface3d_spherical_solid3d_intersection(&surface, &sphere, TEST_TOLERANCE);
        assert!(result.is_some());
    }

    #[test]
    fn test_nurbssurface3d_spherical_solid3d_intersection_returns_surface_domain_start_point() {
        let surface = create_test_surface::<f64>();
        let sphere = SphericalSolid3D::new(
            Point3D::new(0.5, 0.5, 0.0),
            Vector3D::new(0.0, 0.0, 1.0),
            Vector3D::new(1.0, 0.0, 0.0),
            2.0,
        )
        .unwrap();

        let result =
            nurbssurface3d_spherical_solid3d_intersection(&surface, &sphere, TEST_TOLERANCE);
        assert_eq!(
            result,
            Some(representative_surface_domain_start_point(&surface))
        );
    }
}
