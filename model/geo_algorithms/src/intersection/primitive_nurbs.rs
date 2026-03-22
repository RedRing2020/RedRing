//! NURBS × Primitives 交点計算実装（最小スライス）
//!
//! geo_nurbs と geo_primitives 間の交点計算を
//! geo_algorithms で集約して実装するモジュールです。
//!
//! ## 設計方針
//!
//! - NurbsCurve3D × Primitive（9ペア）の交点計算を提供
//! - collision 実装と対称性を保つ（同一ペア数・同一形状セット）
//! - orphan rules への対応は collision 側の NurbsCurveCollider Newtype パターンを参照
//! - Phase C Step B では最小実装（tolerance ベース交点判定）を提供
//! - 重複排除・pair_base 抽出は Step C で実施

use crate::{
    Circle3D, CylindricalSolid3D, EllipsoidalSolid3D, InfiniteLine3D, LineSegment3D, Plane3D,
    Ray3D, SphericalSolid3D,
};
use geo_contracts::{BasicCollision, Scalar};
use geo_core::Point3D;
use geo_nurbs::NurbsCurve3D;

// ── Helper ────────────────────────────────────────────────────────────────────

/// 交点条件の判定に基づき Point3D を返す
fn point_intersection_if<T: Scalar>(point: &Point3D<T>, condition: bool) -> Option<Point3D<T>> {
    if condition {
        Some(Point3D::new(point.x(), point.y(), point.z()))
    } else {
        None
    }
}

// ── NurbsCurve3D × Point3D ────────────────────────────────────────────────────

pub fn nurbscurve3d_point3d_intersection<T: Scalar>(
    curve: &NurbsCurve3D<T>,
    point: &Point3D<T>,
    tolerance: T,
) -> Option<Point3D<T>> {
    // collision の distance_to を使用
    // 距離がtolerance 以内なら交点と判定
    let curve_collider = crate::collision::NurbsCurveCollider::new(curve.clone());
    let distance = curve_collider.distance_to(point);
    point_intersection_if(point, distance <= tolerance)
}

// ── NurbsCurve3D × LineSegment3D ──────────────────────────────────────────────

pub fn nurbscurve3d_line_segment3d_intersection<T: Scalar>(
    curve: &NurbsCurve3D<T>,
    segment: &LineSegment3D<T>,
    tolerance: T,
) -> Option<Point3D<T>> {
    // collision の distance_to を使用
    let curve_collider = crate::collision::NurbsCurveCollider::new(curve.clone());
    let distance = curve_collider.distance_to(segment);

    // 交差の場合、とりあえず線分の始点を返す（Step C で改良）
    if distance <= tolerance {
        Some(segment.start())
    } else {
        None
    }
}

// ── NurbsCurve3D × Ray3D ──────────────────────────────────────────────────────

pub fn nurbscurve3d_ray3d_intersection<T: Scalar>(
    curve: &NurbsCurve3D<T>,
    ray: &Ray3D<T>,
    tolerance: T,
) -> Option<Point3D<T>> {
    // collision の distance_to を使用
    let curve_collider = crate::collision::NurbsCurveCollider::new(curve.clone());
    let distance = curve_collider.distance_to(ray);

    if distance <= tolerance {
        Some(ray.origin())
    } else {
        None
    }
}

// ── NurbsCurve3D × InfiniteLine3D ────────────────────────────────────────────

pub fn nurbscurve3d_infinite_line3d_intersection<T: Scalar>(
    curve: &NurbsCurve3D<T>,
    line: &InfiniteLine3D<T>,
    tolerance: T,
) -> Option<Point3D<T>> {
    // collision の distance_to を使用
    let curve_collider = crate::collision::NurbsCurveCollider::new(curve.clone());
    let distance = curve_collider.distance_to(line);

    if distance <= tolerance {
        let (px, py, pz) = geo_contracts::InfiniteLine3DProperties::point(line);
        Some(Point3D::new(px, py, pz))
    } else {
        None
    }
}

// ── NurbsCurve3D × Circle3D ────────────────────────────────────────────────────

pub fn nurbscurve3d_circle3d_intersection<T: Scalar>(
    curve: &NurbsCurve3D<T>,
    circle: &Circle3D<T>,
    tolerance: T,
) -> Option<Point3D<T>> {
    // collision の distance_to を使用
    let curve_collider = crate::collision::NurbsCurveCollider::new(curve.clone());
    let distance = curve_collider.distance_to(circle);

    if distance <= tolerance {
        let (cx, cy, cz) = geo_contracts::Circle3DProperties::center(circle);
        Some(Point3D::new(cx, cy, cz))
    } else {
        None
    }
}

// ── NurbsCurve3D × Plane3D ────────────────────────────────────────────────────

pub fn nurbscurve3d_plane3d_intersection<T: Scalar>(
    curve: &NurbsCurve3D<T>,
    plane: &Plane3D<T>,
    tolerance: T,
) -> Option<Point3D<T>> {
    // collision の distance_to を使用
    let curve_collider = crate::collision::NurbsCurveCollider::new(curve.clone());
    let distance = curve_collider.distance_to(plane);

    if distance <= tolerance {
        Some(plane.origin())
    } else {
        None
    }
}

// ── NurbsCurve3D × SphericalSolid3D ────────────────────────────────────────────

pub fn nurbscurve3d_spherical_solid3d_intersection<T: Scalar>(
    curve: &NurbsCurve3D<T>,
    sphere: &SphericalSolid3D<T>,
    tolerance: T,
) -> Option<Point3D<T>> {
    // collision の distance_to を使用
    let curve_collider = crate::collision::NurbsCurveCollider::new(curve.clone());
    let distance = curve_collider.distance_to(sphere);

    if distance <= tolerance {
        // 曲線の始点を返す（Step C で改良）
        let (u_min, _) = curve.parameter_domain();
        let vec = curve.evaluate_at(u_min);
        Some(Point3D::new(vec.x(), vec.y(), vec.z()))
    } else {
        None
    }
}

// ── NurbsCurve3D × EllipsoidalSolid3D ──────────────────────────────────────────

pub fn nurbscurve3d_ellipsoidal_solid3d_intersection<T: Scalar>(
    curve: &NurbsCurve3D<T>,
    ellipsoid: &EllipsoidalSolid3D<T>,
    tolerance: T,
) -> Option<Point3D<T>> {
    // collision の distance_to を使用
    let curve_collider = crate::collision::NurbsCurveCollider::new(curve.clone());
    let distance = curve_collider.distance_to(ellipsoid);

    if distance <= tolerance {
        // 曲線の始点を返す（Step C で改良）
        let (u_min, _) = curve.parameter_domain();
        let vec = curve.evaluate_at(u_min);
        Some(Point3D::new(vec.x(), vec.y(), vec.z()))
    } else {
        None
    }
}

// ── NurbsCurve3D × CylindricalSolid3D ──────────────────────────────────────────

pub fn nurbscurve3d_cylindrical_solid3d_intersection<T: Scalar>(
    curve: &NurbsCurve3D<T>,
    cylinder: &CylindricalSolid3D<T>,
    tolerance: T,
) -> Option<Point3D<T>> {
    // collision の distance_to を使用
    let curve_collider = crate::collision::NurbsCurveCollider::new(curve.clone());
    let distance = curve_collider.distance_to(cylinder);

    if distance <= tolerance {
        // 曲線の始点を返す（Step C で改良）
        let (u_min, _) = curve.parameter_domain();
        let vec = curve.evaluate_at(u_min);
        Some(Point3D::new(vec.x(), vec.y(), vec.z()))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use geo_contracts::NurbsCurve3DConstructor;

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

    #[test]
    fn test_nurbscurve3d_point3d_intersection_on_curve() {
        let curve = create_test_curve::<f64>();
        let point = Point3D::new(0.0, 0.0, 0.0);
        let tolerance = 1e-6;

        let result = nurbscurve3d_point3d_intersection(&curve, &point, tolerance);
        assert!(result.is_some());
    }

    #[test]
    fn test_nurbscurve3d_point3d_no_intersection() {
        let curve = create_test_curve::<f64>();
        let point = Point3D::new(10.0, 10.0, 10.0);
        let tolerance = 1e-6;

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
    fn test_nurbscurve3d_ray3d_intersection() {
        let curve = create_test_curve::<f64>();
        let ray = Ray3D::new(
            Point3D::new(0.0, 0.0, 0.0),
            crate::Vector3D::new(1.0, 0.0, 0.0),
        )
        .unwrap();
        let tolerance = 1e-6;

        let result = nurbscurve3d_ray3d_intersection(&curve, &ray, tolerance);
        assert!(result.is_some());
    }
}
