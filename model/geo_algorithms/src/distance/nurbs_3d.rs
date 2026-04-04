//! NurbsCurve3D 距離計算アルゴリズム
//!
//! `geo_nurbs` の NurbsCurve3D と各 3D primitive 間の最短距離計算。
//!
//! 命名規則: `nurbscurve3d_{shape}_distance` / `{shape}_nurbscurve3d_distance`

use crate::{
    Circle3D, CylindricalSolid3D, EllipsoidalSolid3D, InfiniteLine3D, LineSegment3D, NurbsCurve3D,
    Plane3D, Point3D, Ray3D, SphericalSolid3D,
};
use geo_contracts::{CylindricalSolid3DDistance, DistanceConvergenceError, Scalar};

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

/// NurbsCurve3D-LineSegment3D 間の最短距離（フォールバック付き）
pub fn nurbscurve3d_linesegment3d_distance<T: Scalar>(
    curve: &NurbsCurve3D<T>,
    segment: &LineSegment3D<T>,
) -> T {
    match nurbscurve3d_linesegment3d_try_distance(curve, segment, 64) {
        Ok(distance) => distance,
        Err(_) => crate::collision::nurbscurve3d_line_segment3d_distance(curve, segment),
    }
}

/// 逆向きラッパー: linesegment-nurbscurve
pub fn linesegment3d_nurbscurve3d_distance<T: Scalar>(
    segment: &LineSegment3D<T>,
    curve: &NurbsCurve3D<T>,
) -> T {
    nurbscurve3d_linesegment3d_distance(curve, segment)
}

/// NurbsCurve3D-LineSegment3D 間の最短距離（fallible）
pub fn nurbscurve3d_linesegment3d_try_distance<T: Scalar>(
    curve: &NurbsCurve3D<T>,
    segment: &LineSegment3D<T>,
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
        min_distance = min_distance.min(segment.distance_to_point(&curve_point));
    }

    if min_distance == T::INFINITY {
        return Err(DistanceConvergenceError::NumericalFailure);
    }

    Ok(min_distance)
}

/// NurbsCurve3D-Ray3D 間の最短距離（フォールバック付き）
pub fn nurbscurve3d_ray3d_distance<T: Scalar>(curve: &NurbsCurve3D<T>, ray: &Ray3D<T>) -> T {
    match nurbscurve3d_ray3d_try_distance(curve, ray, 64) {
        Ok(distance) => distance,
        Err(_) => crate::collision::nurbscurve3d_ray3d_distance(curve, ray),
    }
}

/// 逆向きラッパー: ray-nurbscurve
pub fn ray3d_nurbscurve3d_distance<T: Scalar>(ray: &Ray3D<T>, curve: &NurbsCurve3D<T>) -> T {
    nurbscurve3d_ray3d_distance(curve, ray)
}

/// NurbsCurve3D-Ray3D 間の最短距離（fallible）
pub fn nurbscurve3d_ray3d_try_distance<T: Scalar>(
    curve: &NurbsCurve3D<T>,
    ray: &Ray3D<T>,
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
        min_distance = min_distance.min(ray.distance_to_point(&curve_point));
    }

    if min_distance == T::INFINITY {
        return Err(DistanceConvergenceError::NumericalFailure);
    }

    Ok(min_distance)
}

/// NurbsCurve3D-InfiniteLine3D 間の最短距離（フォールバック付き）
pub fn nurbscurve3d_infinite_line3d_distance<T: Scalar>(
    curve: &NurbsCurve3D<T>,
    line: &InfiniteLine3D<T>,
) -> T {
    match nurbscurve3d_infinite_line3d_try_distance(curve, line, 64) {
        Ok(distance) => distance,
        Err(_) => crate::collision::nurbscurve3d_infinite_line3d_distance(curve, line),
    }
}

/// 逆向きラッパー: infinite_line-nurbscurve
pub fn infinite_line3d_nurbscurve3d_distance<T: Scalar>(
    line: &InfiniteLine3D<T>,
    curve: &NurbsCurve3D<T>,
) -> T {
    nurbscurve3d_infinite_line3d_distance(curve, line)
}

/// NurbsCurve3D-InfiniteLine3D 間の最短距離（fallible）
pub fn nurbscurve3d_infinite_line3d_try_distance<T: Scalar>(
    curve: &NurbsCurve3D<T>,
    line: &InfiniteLine3D<T>,
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
        min_distance = min_distance.min(line.distance_to_point(&curve_point));
    }

    if min_distance == T::INFINITY {
        return Err(DistanceConvergenceError::NumericalFailure);
    }

    Ok(min_distance)
}

/// NurbsCurve3D-Circle3D 間の最短距離（フォールバック付き）
pub fn nurbscurve3d_circle3d_distance<T: Scalar>(
    curve: &NurbsCurve3D<T>,
    circle: &Circle3D<T>,
) -> T {
    match nurbscurve3d_circle3d_try_distance(curve, circle, 64) {
        Ok(distance) => distance,
        Err(_) => crate::collision::nurbscurve3d_circle3d_distance(curve, circle),
    }
}

/// 逆向きラッパー: circle-nurbscurve
pub fn circle3d_nurbscurve3d_distance<T: Scalar>(
    circle: &Circle3D<T>,
    curve: &NurbsCurve3D<T>,
) -> T {
    nurbscurve3d_circle3d_distance(curve, circle)
}

/// NurbsCurve3D-Circle3D 間の最短距離（fallible）
pub fn nurbscurve3d_circle3d_try_distance<T: Scalar>(
    curve: &NurbsCurve3D<T>,
    circle: &Circle3D<T>,
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
        min_distance = min_distance.min(circle.distance_to_point_3d(curve_point));
    }

    if min_distance == T::INFINITY {
        return Err(DistanceConvergenceError::NumericalFailure);
    }

    Ok(min_distance)
}

/// NurbsCurve3D-Plane3D 間の最短距離（フォールバック付き）
pub fn nurbscurve3d_plane3d_distance<T: Scalar>(curve: &NurbsCurve3D<T>, plane: &Plane3D<T>) -> T {
    match nurbscurve3d_plane3d_try_distance(curve, plane, 64) {
        Ok(distance) => distance,
        Err(_) => crate::collision::nurbscurve3d_plane3d_distance(curve, plane),
    }
}

/// 逆向きラッパー: plane-nurbscurve
pub fn plane3d_nurbscurve3d_distance<T: Scalar>(plane: &Plane3D<T>, curve: &NurbsCurve3D<T>) -> T {
    nurbscurve3d_plane3d_distance(curve, plane)
}

/// NurbsCurve3D-Plane3D 間の最短距離（fallible）
pub fn nurbscurve3d_plane3d_try_distance<T: Scalar>(
    curve: &NurbsCurve3D<T>,
    plane: &Plane3D<T>,
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
        min_distance = min_distance.min(plane.distance_to_point(curve_point).abs());
    }

    if min_distance == T::INFINITY {
        return Err(DistanceConvergenceError::NumericalFailure);
    }

    Ok(min_distance)
}

/// NurbsCurve3D-SphericalSolid3D 間の最短距離（フォールバック付き）
pub fn nurbscurve3d_spherical_solid3d_distance<T: Scalar>(
    curve: &NurbsCurve3D<T>,
    sphere: &SphericalSolid3D<T>,
) -> T {
    match nurbscurve3d_spherical_solid3d_try_distance(curve, sphere, 64) {
        Ok(distance) => distance,
        Err(_) => crate::collision::nurbscurve3d_spherical_solid3d_distance(curve, sphere),
    }
}

/// 逆向きラッパー: spherical_solid-nurbscurve
pub fn spherical_solid3d_nurbscurve3d_distance<T: Scalar>(
    sphere: &SphericalSolid3D<T>,
    curve: &NurbsCurve3D<T>,
) -> T {
    nurbscurve3d_spherical_solid3d_distance(curve, sphere)
}

/// NurbsCurve3D-SphericalSolid3D 間の最短距離（fallible）
pub fn nurbscurve3d_spherical_solid3d_try_distance<T: Scalar>(
    curve: &NurbsCurve3D<T>,
    sphere: &SphericalSolid3D<T>,
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
        min_distance = min_distance.min(sphere.distance_to_surface(curve_point));
    }

    if min_distance == T::INFINITY {
        return Err(DistanceConvergenceError::NumericalFailure);
    }

    Ok(min_distance)
}

/// NurbsCurve3D-EllipsoidalSolid3D 間の最短距離（フォールバック付き）
pub fn nurbscurve3d_ellipsoidal_solid3d_distance<T: Scalar>(
    curve: &NurbsCurve3D<T>,
    ellipsoid: &EllipsoidalSolid3D<T>,
) -> T {
    match nurbscurve3d_ellipsoidal_solid3d_try_distance(curve, ellipsoid, 64) {
        Ok(distance) => distance,
        Err(_) => crate::collision::nurbscurve3d_ellipsoidal_solid3d_distance(curve, ellipsoid),
    }
}

/// 逆向きラッパー: ellipsoidal_solid-nurbscurve
pub fn ellipsoidal_solid3d_nurbscurve3d_distance<T: Scalar>(
    ellipsoid: &EllipsoidalSolid3D<T>,
    curve: &NurbsCurve3D<T>,
) -> T {
    nurbscurve3d_ellipsoidal_solid3d_distance(curve, ellipsoid)
}

/// NurbsCurve3D-EllipsoidalSolid3D 間の最短距離（fallible）
pub fn nurbscurve3d_ellipsoidal_solid3d_try_distance<T: Scalar>(
    curve: &NurbsCurve3D<T>,
    ellipsoid: &EllipsoidalSolid3D<T>,
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
        let d = if ellipsoid.contains_point(&curve_point) {
            T::ZERO
        } else {
            ellipsoid.distance_to_surface(&curve_point)
        };
        min_distance = min_distance.min(d);
    }

    if min_distance == T::INFINITY {
        return Err(DistanceConvergenceError::NumericalFailure);
    }

    Ok(min_distance)
}

/// NurbsCurve3D-CylindricalSolid3D 間の最短距離（フォールバック付き）
pub fn nurbscurve3d_cylindrical_solid3d_distance<T: Scalar>(
    curve: &NurbsCurve3D<T>,
    cylinder: &CylindricalSolid3D<T>,
) -> T {
    match nurbscurve3d_cylindrical_solid3d_try_distance(curve, cylinder, 64) {
        Ok(distance) => distance,
        Err(_) => crate::collision::nurbscurve3d_cylindrical_solid3d_distance(curve, cylinder),
    }
}

/// 逆向きラッパー: cylindrical_solid-nurbscurve
pub fn cylindrical_solid3d_nurbscurve3d_distance<T: Scalar>(
    cylinder: &CylindricalSolid3D<T>,
    curve: &NurbsCurve3D<T>,
) -> T {
    nurbscurve3d_cylindrical_solid3d_distance(curve, cylinder)
}

/// NurbsCurve3D-CylindricalSolid3D 間の最短距離（fallible）
pub fn nurbscurve3d_cylindrical_solid3d_try_distance<T: Scalar>(
    curve: &NurbsCurve3D<T>,
    cylinder: &CylindricalSolid3D<T>,
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
        let d = <CylindricalSolid3D<T> as CylindricalSolid3DDistance<T>>::distance_to_point(
            cylinder,
            (curve_point.x(), curve_point.y(), curve_point.z()),
        );
        min_distance = min_distance.min(d);
    }

    if min_distance == T::INFINITY {
        return Err(DistanceConvergenceError::NumericalFailure);
    }

    Ok(min_distance)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Direction3D, Vector3D};
    use geo_contracts::{
        CylindricalSolid3DConstructor, EllipsoidalSolid3DConstructor, NurbsCurve3DConstructor,
        SphericalSolid3DConstructor, ToleranceSettings,
    };

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

    fn standard_distance_tol() -> f64 {
        ToleranceSettings::<f64>::standard().distance_tolerance
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
        let tol = standard_distance_tol();
        let curve = create_test_curve();
        let point = Point3D::new(0.2, 0.2, 0.0);

        let a_to_b = nurbscurve3d_point3d_distance(&curve, &point);
        let b_to_a = point3d_nurbscurve3d_distance(&point, &curve);
        assert!((a_to_b - b_to_a).abs() < tol);
    }

    #[test]
    fn nurbscurve3d_linesegment3d_distance_normal() {
        let curve = create_test_curve();
        let segment =
            LineSegment3D::new(Point3D::new(0.5, -0.5, 0.0), Point3D::new(0.5, 0.5, 0.0)).unwrap();

        let distance = nurbscurve3d_linesegment3d_distance(&curve, &segment);
        assert!(distance >= 0.0);
        assert!(distance < 1.0);
    }

    #[test]
    fn nurbscurve3d_linesegment3d_distance_zero() {
        let curve = create_test_curve();
        let segment =
            LineSegment3D::new(Point3D::new(0.0, 0.0, 0.0), Point3D::new(1.0, 0.0, 0.0)).unwrap();

        let distance = nurbscurve3d_linesegment3d_distance(&curve, &segment);
        assert!(distance < 0.1);
    }

    #[test]
    fn nurbscurve3d_linesegment3d_symmetric() {
        let tol = standard_distance_tol();
        let curve = create_test_curve();
        let segment =
            LineSegment3D::new(Point3D::new(0.5, -0.5, 0.0), Point3D::new(0.5, 0.5, 0.0)).unwrap();

        let a_to_b = nurbscurve3d_linesegment3d_distance(&curve, &segment);
        let b_to_a = linesegment3d_nurbscurve3d_distance(&segment, &curve);
        assert!((a_to_b - b_to_a).abs() < tol);
    }

    #[test]
    fn nurbscurve3d_linesegment3d_try_distance_invalid_samples() {
        let curve = create_test_curve();
        let segment =
            LineSegment3D::new(Point3D::new(0.5, -0.5, 0.0), Point3D::new(0.5, 0.5, 0.0)).unwrap();

        let result = nurbscurve3d_linesegment3d_try_distance(&curve, &segment, 0);
        assert!(matches!(
            result,
            Err(DistanceConvergenceError::InvalidInitialization)
        ));
    }

    #[test]
    fn nurbscurve3d_linesegment3d_try_distance_one_sample() {
        let curve = create_test_curve();
        let segment =
            LineSegment3D::new(Point3D::new(0.5, -0.5, 0.0), Point3D::new(0.5, 0.5, 0.0)).unwrap();

        let result = nurbscurve3d_linesegment3d_try_distance(&curve, &segment, 1);
        assert!(matches!(
            result,
            Err(DistanceConvergenceError::NotConverged { iterations: 1, .. })
        ));
    }

    #[test]
    fn nurbscurve3d_ray3d_distance_normal() {
        let curve = create_test_curve();
        let ray = Ray3D::new(Point3D::new(0.5, -0.5, 0.0), Vector3D::new(0.0, 1.0, 0.0)).unwrap();

        let distance = nurbscurve3d_ray3d_distance(&curve, &ray);
        assert!(distance >= 0.0);
        assert!(distance < 1.0);
    }

    #[test]
    fn nurbscurve3d_ray3d_distance_zero() {
        let curve = create_test_curve();
        let ray = Ray3D::new(Point3D::new(0.0, 0.0, 0.0), Vector3D::new(1.0, 0.0, 0.0)).unwrap();

        let distance = nurbscurve3d_ray3d_distance(&curve, &ray);
        assert!(distance < 0.1);
    }

    #[test]
    fn nurbscurve3d_ray3d_symmetric() {
        let tol = standard_distance_tol();
        let curve = create_test_curve();
        let ray = Ray3D::new(Point3D::new(0.5, -0.5, 0.0), Vector3D::new(0.0, 1.0, 0.0)).unwrap();

        let a_to_b = nurbscurve3d_ray3d_distance(&curve, &ray);
        let b_to_a = ray3d_nurbscurve3d_distance(&ray, &curve);
        assert!((a_to_b - b_to_a).abs() < tol);
    }

    #[test]
    fn nurbscurve3d_ray3d_try_distance_invalid_samples() {
        let curve = create_test_curve();
        let ray = Ray3D::new(Point3D::new(0.5, -0.5, 0.0), Vector3D::new(0.0, 1.0, 0.0)).unwrap();

        let result = nurbscurve3d_ray3d_try_distance(&curve, &ray, 0);
        assert!(matches!(
            result,
            Err(DistanceConvergenceError::InvalidInitialization)
        ));
    }

    #[test]
    fn nurbscurve3d_ray3d_try_distance_one_sample() {
        let curve = create_test_curve();
        let ray = Ray3D::new(Point3D::new(0.5, -0.5, 0.0), Vector3D::new(0.0, 1.0, 0.0)).unwrap();

        let result = nurbscurve3d_ray3d_try_distance(&curve, &ray, 1);
        assert!(matches!(
            result,
            Err(DistanceConvergenceError::NotConverged { iterations: 1, .. })
        ));
    }

    #[test]
    fn nurbscurve3d_infinite_line3d_distance_normal() {
        let curve = create_test_curve();
        let line = InfiniteLine3D::from_two_points(
            Point3D::new(0.5, -0.5, 0.0),
            Point3D::new(0.5, 0.5, 0.0),
        )
        .unwrap();

        let distance = nurbscurve3d_infinite_line3d_distance(&curve, &line);
        assert!(distance >= 0.0);
        assert!(distance < 1.0);
    }

    #[test]
    fn nurbscurve3d_infinite_line3d_distance_zero() {
        let curve = create_test_curve();
        let line = InfiniteLine3D::from_two_points(
            Point3D::new(0.0, 0.0, 0.0),
            Point3D::new(1.0, 0.0, 0.0),
        )
        .unwrap();

        let distance = nurbscurve3d_infinite_line3d_distance(&curve, &line);
        assert!(distance < 0.1);
    }

    #[test]
    fn nurbscurve3d_infinite_line3d_symmetric() {
        let tol = standard_distance_tol();
        let curve = create_test_curve();
        let line = InfiniteLine3D::from_two_points(
            Point3D::new(0.5, -0.5, 0.0),
            Point3D::new(0.5, 0.5, 0.0),
        )
        .unwrap();

        let a_to_b = nurbscurve3d_infinite_line3d_distance(&curve, &line);
        let b_to_a = infinite_line3d_nurbscurve3d_distance(&line, &curve);
        assert!((a_to_b - b_to_a).abs() < tol);
    }

    #[test]
    fn nurbscurve3d_infinite_line3d_try_distance_invalid_samples() {
        let curve = create_test_curve();
        let line = InfiniteLine3D::from_two_points(
            Point3D::new(0.5, -0.5, 0.0),
            Point3D::new(0.5, 0.5, 0.0),
        )
        .unwrap();

        let result = nurbscurve3d_infinite_line3d_try_distance(&curve, &line, 0);
        assert!(matches!(
            result,
            Err(DistanceConvergenceError::InvalidInitialization)
        ));
    }

    #[test]
    fn nurbscurve3d_infinite_line3d_try_distance_one_sample() {
        let curve = create_test_curve();
        let line = InfiniteLine3D::from_two_points(
            Point3D::new(0.5, -0.5, 0.0),
            Point3D::new(0.5, 0.5, 0.0),
        )
        .unwrap();

        let result = nurbscurve3d_infinite_line3d_try_distance(&curve, &line, 1);
        assert!(matches!(
            result,
            Err(DistanceConvergenceError::NotConverged { iterations: 1, .. })
        ));
    }

    #[test]
    fn nurbscurve3d_circle3d_distance_normal() {
        let curve = create_test_curve();
        let circle = Circle3D::new(
            Point3D::new(0.5, 0.5, 0.0),
            Direction3D::new(0.0, 0.0, 1.0).unwrap(),
            0.4,
        )
        .unwrap();

        let distance = nurbscurve3d_circle3d_distance(&curve, &circle);
        assert!(distance >= 0.0);
        assert!(distance < 1.0);
    }

    #[test]
    fn nurbscurve3d_circle3d_distance_zero() {
        let curve = create_test_curve();
        let circle = Circle3D::new(
            Point3D::new(0.0, 0.0, 0.0),
            Direction3D::new(0.0, 0.0, 1.0).unwrap(),
            0.5,
        )
        .unwrap();

        let distance = nurbscurve3d_circle3d_distance(&curve, &circle);
        assert!(distance < 0.1);
    }

    #[test]
    fn nurbscurve3d_circle3d_symmetric() {
        let tol = standard_distance_tol();
        let curve = create_test_curve();
        let circle = Circle3D::new(
            Point3D::new(0.5, 0.5, 0.0),
            Direction3D::new(0.0, 0.0, 1.0).unwrap(),
            0.4,
        )
        .unwrap();

        let a_to_b = nurbscurve3d_circle3d_distance(&curve, &circle);
        let b_to_a = circle3d_nurbscurve3d_distance(&circle, &curve);
        assert!((a_to_b - b_to_a).abs() < tol);
    }

    #[test]
    fn nurbscurve3d_circle3d_try_distance_invalid_samples() {
        let curve = create_test_curve();
        let circle = Circle3D::new(
            Point3D::new(0.5, 0.5, 0.0),
            Direction3D::new(0.0, 0.0, 1.0).unwrap(),
            0.4,
        )
        .unwrap();

        let result = nurbscurve3d_circle3d_try_distance(&curve, &circle, 0);
        assert!(matches!(
            result,
            Err(DistanceConvergenceError::InvalidInitialization)
        ));
    }

    #[test]
    fn nurbscurve3d_circle3d_try_distance_one_sample() {
        let curve = create_test_curve();
        let circle = Circle3D::new(
            Point3D::new(0.5, 0.5, 0.0),
            Direction3D::new(0.0, 0.0, 1.0).unwrap(),
            0.4,
        )
        .unwrap();

        let result = nurbscurve3d_circle3d_try_distance(&curve, &circle, 1);
        assert!(matches!(
            result,
            Err(DistanceConvergenceError::NotConverged { iterations: 1, .. })
        ));
    }

    #[test]
    fn nurbscurve3d_plane3d_distance_normal() {
        let curve = create_test_curve();
        let plane = Plane3D::xy_plane(1.0);

        let distance = nurbscurve3d_plane3d_distance(&curve, &plane);
        assert!(distance >= 0.0);
        assert!(distance < 2.0);
    }

    #[test]
    fn nurbscurve3d_plane3d_distance_zero() {
        let curve = create_test_curve();
        // 曲線は z=0 平面上にあるので距離 0
        let plane = Plane3D::xy_plane(0.0);

        let distance = nurbscurve3d_plane3d_distance(&curve, &plane);
        let tol = standard_distance_tol();
        assert!(distance < tol);
    }

    #[test]
    fn nurbscurve3d_plane3d_symmetric() {
        let tol = standard_distance_tol();
        let curve = create_test_curve();
        let plane = Plane3D::xy_plane(1.0);

        let a_to_b = nurbscurve3d_plane3d_distance(&curve, &plane);
        let b_to_a = plane3d_nurbscurve3d_distance(&plane, &curve);
        assert!((a_to_b - b_to_a).abs() < tol);
    }

    #[test]
    fn nurbscurve3d_plane3d_try_distance_invalid_samples() {
        let curve = create_test_curve();
        let plane = Plane3D::xy_plane(1.0);

        let result = nurbscurve3d_plane3d_try_distance(&curve, &plane, 0);
        assert!(matches!(
            result,
            Err(DistanceConvergenceError::InvalidInitialization)
        ));
    }

    #[test]
    fn nurbscurve3d_plane3d_try_distance_one_sample() {
        let curve = create_test_curve();
        let plane = Plane3D::xy_plane(1.0);

        let result = nurbscurve3d_plane3d_try_distance(&curve, &plane, 1);
        assert!(matches!(
            result,
            Err(DistanceConvergenceError::NotConverged { iterations: 1, .. })
        ));
    }

    // ===== SphericalSolid3D テスト =====

    #[test]
    fn nurbscurve3d_spherical_solid3d_distance_normal() {
        let curve = create_test_curve();
        let sphere = <SphericalSolid3D<f64> as SphericalSolid3DConstructor<f64>>::new_standard(
            (5.0, 5.0, 5.0),
            0.5,
        )
        .unwrap();
        let distance = nurbscurve3d_spherical_solid3d_distance(&curve, &sphere);
        assert!(distance > 0.0);
        assert!(distance < 20.0);
    }

    #[test]
    fn nurbscurve3d_spherical_solid3d_distance_intersecting() {
        let curve = create_test_curve();
        // 曲線全体を包含する球（曲線が内部にあるため distance_to_surface は負値）
        let sphere = <SphericalSolid3D<f64> as SphericalSolid3DConstructor<f64>>::new_standard(
            (0.5, 0.5, 0.0),
            2.0,
        )
        .unwrap();
        let distance = nurbscurve3d_spherical_solid3d_distance(&curve, &sphere);
        assert!(distance < 0.0);
    }

    #[test]
    fn nurbscurve3d_spherical_solid3d_symmetric() {
        let tol = standard_distance_tol();
        let curve = create_test_curve();
        let sphere = <SphericalSolid3D<f64> as SphericalSolid3DConstructor<f64>>::new_standard(
            (5.0, 5.0, 5.0),
            0.5,
        )
        .unwrap();
        let a_to_b = nurbscurve3d_spherical_solid3d_distance(&curve, &sphere);
        let b_to_a = spherical_solid3d_nurbscurve3d_distance(&sphere, &curve);
        assert!((a_to_b - b_to_a).abs() < tol);
    }

    #[test]
    fn nurbscurve3d_spherical_solid3d_try_distance_invalid_samples() {
        let curve = create_test_curve();
        let sphere = <SphericalSolid3D<f64> as SphericalSolid3DConstructor<f64>>::new_standard(
            (5.0, 5.0, 5.0),
            0.5,
        )
        .unwrap();
        let result = nurbscurve3d_spherical_solid3d_try_distance(&curve, &sphere, 0);
        assert!(matches!(
            result,
            Err(DistanceConvergenceError::InvalidInitialization)
        ));
    }

    #[test]
    fn nurbscurve3d_spherical_solid3d_try_distance_one_sample() {
        let curve = create_test_curve();
        let sphere = <SphericalSolid3D<f64> as SphericalSolid3DConstructor<f64>>::new_standard(
            (5.0, 5.0, 5.0),
            0.5,
        )
        .unwrap();
        let result = nurbscurve3d_spherical_solid3d_try_distance(&curve, &sphere, 1);
        assert!(matches!(
            result,
            Err(DistanceConvergenceError::NotConverged { iterations: 1, .. })
        ));
    }

    // ===== EllipsoidalSolid3D テスト =====

    #[test]
    fn nurbscurve3d_ellipsoidal_solid3d_distance_normal() {
        let curve = create_test_curve();
        let ellipsoid =
            <EllipsoidalSolid3D<f64> as EllipsoidalSolid3DConstructor<f64>>::new_standard(
                (5.0, 5.0, 5.0),
                0.5,
                0.5,
                0.5,
            )
            .unwrap();
        let distance = nurbscurve3d_ellipsoidal_solid3d_distance(&curve, &ellipsoid);
        assert!(distance >= 0.0);
        assert!(distance < 20.0);
    }

    #[test]
    fn nurbscurve3d_ellipsoidal_solid3d_distance_zero() {
        let curve = create_test_curve();
        // 曲線全体を包含する楕円体（内部では 0 を返す）
        let ellipsoid =
            <EllipsoidalSolid3D<f64> as EllipsoidalSolid3DConstructor<f64>>::new_standard(
                (0.5, 0.5, 0.0),
                2.0,
                2.0,
                2.0,
            )
            .unwrap();
        let distance = nurbscurve3d_ellipsoidal_solid3d_distance(&curve, &ellipsoid);
        let tol = standard_distance_tol();
        assert!(distance < tol);
    }

    #[test]
    fn nurbscurve3d_ellipsoidal_solid3d_symmetric() {
        let tol = standard_distance_tol();
        let curve = create_test_curve();
        let ellipsoid =
            <EllipsoidalSolid3D<f64> as EllipsoidalSolid3DConstructor<f64>>::new_standard(
                (5.0, 5.0, 5.0),
                0.5,
                0.5,
                0.5,
            )
            .unwrap();
        let a_to_b = nurbscurve3d_ellipsoidal_solid3d_distance(&curve, &ellipsoid);
        let b_to_a = ellipsoidal_solid3d_nurbscurve3d_distance(&ellipsoid, &curve);
        assert!((a_to_b - b_to_a).abs() < tol);
    }

    #[test]
    fn nurbscurve3d_ellipsoidal_solid3d_try_distance_invalid_samples() {
        let curve = create_test_curve();
        let ellipsoid =
            <EllipsoidalSolid3D<f64> as EllipsoidalSolid3DConstructor<f64>>::new_standard(
                (5.0, 5.0, 5.0),
                0.5,
                0.5,
                0.5,
            )
            .unwrap();
        let result = nurbscurve3d_ellipsoidal_solid3d_try_distance(&curve, &ellipsoid, 0);
        assert!(matches!(
            result,
            Err(DistanceConvergenceError::InvalidInitialization)
        ));
    }

    #[test]
    fn nurbscurve3d_ellipsoidal_solid3d_try_distance_one_sample() {
        let curve = create_test_curve();
        let ellipsoid =
            <EllipsoidalSolid3D<f64> as EllipsoidalSolid3DConstructor<f64>>::new_standard(
                (5.0, 5.0, 5.0),
                0.5,
                0.5,
                0.5,
            )
            .unwrap();
        let result = nurbscurve3d_ellipsoidal_solid3d_try_distance(&curve, &ellipsoid, 1);
        assert!(matches!(
            result,
            Err(DistanceConvergenceError::NotConverged { iterations: 1, .. })
        ));
    }

    // ===== CylindricalSolid3D テスト =====

    #[test]
    fn nurbscurve3d_cylindrical_solid3d_distance_normal() {
        let curve = create_test_curve();
        let cylinder =
            <CylindricalSolid3D<f64> as CylindricalSolid3DConstructor<f64>>::new_standard(
                (5.0, 5.0, 5.0),
                0.5,
                2.0,
            )
            .unwrap();
        let distance = nurbscurve3d_cylindrical_solid3d_distance(&curve, &cylinder);
        assert!(distance >= 0.0);
        assert!(distance < 20.0);
    }

    #[test]
    fn nurbscurve3d_cylindrical_solid3d_distance_zero() {
        let curve = create_test_curve();
        // 曲線全体を包含する円柱（z軸方向、z=-1〜z=2 の範囲）
        let cylinder =
            <CylindricalSolid3D<f64> as CylindricalSolid3DConstructor<f64>>::new_standard(
                (0.5, 0.5, -1.0),
                2.0,
                3.0,
            )
            .unwrap();
        let distance = nurbscurve3d_cylindrical_solid3d_distance(&curve, &cylinder);
        let tol = standard_distance_tol();
        assert!(distance < tol);
    }

    #[test]
    fn nurbscurve3d_cylindrical_solid3d_symmetric() {
        let tol = standard_distance_tol();
        let curve = create_test_curve();
        let cylinder =
            <CylindricalSolid3D<f64> as CylindricalSolid3DConstructor<f64>>::new_standard(
                (5.0, 5.0, 5.0),
                0.5,
                2.0,
            )
            .unwrap();
        let a_to_b = nurbscurve3d_cylindrical_solid3d_distance(&curve, &cylinder);
        let b_to_a = cylindrical_solid3d_nurbscurve3d_distance(&cylinder, &curve);
        assert!((a_to_b - b_to_a).abs() < tol);
    }

    #[test]
    fn nurbscurve3d_cylindrical_solid3d_try_distance_invalid_samples() {
        let curve = create_test_curve();
        let cylinder =
            <CylindricalSolid3D<f64> as CylindricalSolid3DConstructor<f64>>::new_standard(
                (5.0, 5.0, 5.0),
                0.5,
                2.0,
            )
            .unwrap();
        let result = nurbscurve3d_cylindrical_solid3d_try_distance(&curve, &cylinder, 0);
        assert!(matches!(
            result,
            Err(DistanceConvergenceError::InvalidInitialization)
        ));
    }

    #[test]
    fn nurbscurve3d_cylindrical_solid3d_try_distance_one_sample() {
        let curve = create_test_curve();
        let cylinder =
            <CylindricalSolid3D<f64> as CylindricalSolid3DConstructor<f64>>::new_standard(
                (5.0, 5.0, 5.0),
                0.5,
                2.0,
            )
            .unwrap();
        let result = nurbscurve3d_cylindrical_solid3d_try_distance(&curve, &cylinder, 1);
        assert!(matches!(
            result,
            Err(DistanceConvergenceError::NotConverged { iterations: 1, .. })
        ));
    }
}
