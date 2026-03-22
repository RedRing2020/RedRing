//! 3D Primitive distance algorithms
//!
//! `geo_primitives` に実装済みの各 3D 形状の距離計算メソッドを
//! `geo_algorithms` 層の公開 entrypoint としてまとめた薄いラッパー群。
//!
//! 命名規則: `{shape_a}_{shape_b}_distance`

use crate::{Circle3D, InfiniteLine3D, LineSegment3D, NurbsCurve3D, Plane3D, Point3D, Ray3D};
use geo_contracts::{DistanceConvergenceError, Scalar};

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

/// NurbsCurve3D-点 間の最短距離（フォールバック付き）
pub fn nurbscurve3d_point3d_distance<T: Scalar>(curve: &NurbsCurve3D<T>, point: &Point3D<T>) -> T {
    match nurbscurve3d_point3d_try_distance(curve, point, 64) {
        Ok(distance) => distance,
        Err(_) => crate::collision::nurbscurve3d_point3d_distance(curve, point),
    }
}

/// 逆向きラッパー: point-nurbscurve
pub fn point3d_nurbscurve3d_distance<T: Scalar>(point: &Point3D<T>, curve: &NurbsCurve3D<T>) -> T {
    nurbscurve3d_point3d_distance(curve, point)
}

/// NurbsCurve3D-点 間の最短距離（fallible）
pub fn nurbscurve3d_point3d_try_distance<T: Scalar>(
    curve: &NurbsCurve3D<T>,
    point: &Point3D<T>,
    max_samples: usize,
) -> Result<T, DistanceConvergenceError<T>> {
    if max_samples == 0 {
        return Err(DistanceConvergenceError::InvalidInitialization);
    }

    if max_samples == 1 {
        return Err(DistanceConvergenceError::NotConverged {
            iterations: 1,
            residual: T::INFINITY,
        });
    }

    let (u_min, u_max) = curve.parameter_domain();
    let delta_u = (u_max - u_min) / T::from_usize(max_samples);

    let mut min_distance = T::INFINITY;
    for i in 0..=max_samples {
        let u = u_min + delta_u * T::from_usize(i);
        let curve_vec = curve.evaluate_at(u);
        let curve_point = Point3D::new(curve_vec.x(), curve_vec.y(), curve_vec.z());
        min_distance = min_distance.min(curve_point.distance_to(point));
    }

    if min_distance == T::INFINITY {
        return Err(DistanceConvergenceError::NumericalFailure);
    }

    Ok(min_distance)
}

#[cfg(test)]
mod tests {
    use super::*;
    use geo_contracts::NurbsCurve3DConstructor;

    fn create_test_curve() -> NurbsCurve3D<f64> {
        let control_points = vec![
            (0.0, 0.0, 0.0),
            (1.0, 0.0, 0.0),
            (1.0, 1.0, 0.0),
            (0.0, 1.0, 0.0),
        ];
        let knots = vec![0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0];
        let weights = Some(vec![1.0, 1.0, 1.0, 1.0]);

        <NurbsCurve3D<f64> as NurbsCurve3DConstructor<f64>>::new(3, knots, control_points, weights)
            .unwrap()
    }

    #[test]
    fn infinite_line3d_distance_parallel_lines() {
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
        assert!((d - 2.0).abs() < 1e-10);
    }

    #[test]
    fn infinite_line3d_distance_intersecting_lines() {
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
        assert!(d.abs() < 1e-10);
    }

    #[test]
    fn nurbscurve_try_distance_rejects_zero_samples() {
        let curve = create_test_curve();
        let point = Point3D::new(0.2, 0.2, 0.0);

        let result = nurbscurve3d_point3d_try_distance(&curve, &point, 0);
        assert!(matches!(
            result,
            Err(DistanceConvergenceError::InvalidInitialization)
        ));
    }

    #[test]
    fn nurbscurve_try_distance_reports_not_converged_for_one_sample() {
        let curve = create_test_curve();
        let point = Point3D::new(0.2, 0.2, 0.0);

        let result = nurbscurve3d_point3d_try_distance(&curve, &point, 1);
        assert!(matches!(
            result,
            Err(DistanceConvergenceError::NotConverged { iterations: 1, .. })
        ));
    }

    #[test]
    fn point_to_nurbscurve_wrapper_is_symmetric() {
        let curve = create_test_curve();
        let point = Point3D::new(0.2, 0.2, 0.0);

        let a_to_b = nurbscurve3d_point3d_distance(&curve, &point);
        let b_to_a = point3d_nurbscurve3d_distance(&point, &curve);
        assert!((a_to_b - b_to_a).abs() < 1e-10);
    }
}
