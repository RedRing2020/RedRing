//! `NurbsCurve3D` のテスト

use super::*;
use crate::clamped_knot_vector;
use crate::{AdaptiveTessellationSettings, NurbsCurveAdaptiveTessellation};
use geo_contracts::{Bounded, PrimitiveKind, PrimitiveMetadata};

/// 制御点 (0, 0, 0), (1, 1, 1), (2, 0, 0) の 2 次 Bézier 曲線
fn bezier() -> NurbsCurve3D<f64> {
    NurbsCurve3D::new(
        2,
        clamped_knot_vector(2, 3),
        &[
            Point3D::new(0.0, 0.0, 0.0),
            Point3D::new(1.0, 1.0, 1.0),
            Point3D::new(2.0, 0.0, 0.0),
        ],
        None,
    )
    .unwrap()
}

/// 内部ノット 0.5 を持つ 2 次の曲線（4 つの制御点）
fn spline_with_interior_knot() -> NurbsCurve3D<f64> {
    NurbsCurve3D::new(
        2,
        vec![0.0, 0.0, 0.0, 0.5, 1.0, 1.0, 1.0],
        &[
            Point3D::new(0.0, 0.0, 0.0),
            Point3D::new(1.0, 2.0, 0.0),
            Point3D::new(3.0, 2.0, 1.0),
            Point3D::new(4.0, 0.0, 0.0),
        ],
        Some(vec![1.0, 2.0, 0.5, 1.0]),
    )
    .unwrap()
}

#[test]
fn test_creation_and_properties() {
    let curve = bezier();
    assert_eq!(curve.primitive_kind(), PrimitiveKind::NurbsCurve3D);
    assert_eq!(curve.degree(), 2);
    assert_eq!(curve.control_point_count(), 3);
    assert_eq!(curve.knot_vector(), &[0.0, 0.0, 0.0, 1.0, 1.0, 1.0]);
    assert_eq!(curve.control_point(2), Some(Point3D::new(2.0, 0.0, 0.0)));
    assert_eq!(curve.control_point(3), None);
    assert_eq!(curve.weight(1), Some(1.0));
    assert_eq!(curve.weight(3), None);
    assert_eq!(curve.weights(), None);
    assert_eq!(
        curve.coordinates(),
        &[0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 2.0, 0.0, 0.0]
    );

    let rational = spline_with_interior_knot();
    assert_eq!(rational.weights(), Some(&[1.0, 2.0, 0.5, 1.0][..]));
    assert_eq!(rational.weight(2), Some(0.5));
}

#[test]
fn test_point_and_derivative() {
    let curve = bezier();

    // C(t) = (2t, 2t(1-t), 2t(1-t))、C'(t) = (2, 2 - 4t, 2 - 4t)
    for t in [0.0, 0.3, 0.5, 1.0] {
        let y = 2.0 * t * (1.0 - t);
        let point = curve.point_at_parameter(t).unwrap();
        assert!(point.distance_to(Point3D::new(2.0 * t, y, y)) < 1e-12);
        let derivative = curve.derivative_at(t).unwrap();
        assert!((derivative - Vector3D::new(2.0, 2.0 - 4.0 * t, 2.0 - 4.0 * t)).length() < 1e-12);
    }
}

#[test]
fn test_derivative_matches_difference_of_points() {
    // 内部ノット・重みを持つ有理曲線でも、導関数は点の中央差分と一致する。
    // 内部ノットでは 2 階導関数が不連続なため、中央差分の誤差は h の 1 次になる
    let curve = spline_with_interior_knot();
    let h = 1e-6;
    for t in [0.2, 0.5, 0.7] {
        let difference = Vector3D::from_points(
            curve.point_at_parameter(t - h).unwrap(),
            curve.point_at_parameter(t + h).unwrap(),
        ) / (2.0 * h);
        assert!(
            (curve.derivative_at(t).unwrap() - difference).length() < 1e-5,
            "t = {t}"
        );
    }

    // 端点では C'(0) = p / (u_{p+1} - u_p) · (w1 / w0) · (P1 - P0)（終端も同様）
    let start = curve.derivative_at(0.0).unwrap();
    assert!((start - Vector3D::new(1.0, 2.0, 0.0) * (2.0 / 0.5 * 2.0)).length() < 1e-12);
    let end = curve.derivative_at(1.0).unwrap();
    assert!((end - Vector3D::new(1.0, -2.0, -1.0) * (2.0 / 0.5 * 0.5)).length() < 1e-12);
}

#[test]
fn test_evaluation_at_and_outside_domain() {
    let curve = bezier();
    assert_eq!(
        curve.point_at_parameter(0.0),
        Some(Point3D::new(0.0, 0.0, 0.0))
    );
    assert!(
        curve
            .point_at_parameter(1.0)
            .unwrap()
            .distance_to(Point3D::new(2.0, 0.0, 0.0))
            < 1e-12
    );
    assert!(curve.point_at_parameter(-0.5).is_none());
    assert!(curve.point_at_parameter(1.5).is_none());
    assert!(curve.derivative_at(f64::INFINITY).is_none());
}

#[test]
fn test_length() {
    // 直線上の制御点
    let line = NurbsCurve3D::new(
        2,
        clamped_knot_vector(2, 3),
        &[
            Point3D::new(0.0, 0.0, 0.0),
            Point3D::new(1.0, 1.0, 1.0),
            Point3D::new(3.0, 3.0, 3.0),
        ],
        None,
    )
    .unwrap();
    assert!((line.length() - 3.0 * 3.0_f64.sqrt()).abs() < 1e-9);

    // 半径 2 の円の 1/4（有理 2 次）の長さは π
    let arc = NurbsCurve3D::new(
        2,
        clamped_knot_vector(2, 3),
        &[
            Point3D::new(2.0, 0.0, 1.0),
            Point3D::new(2.0, 2.0, 1.0),
            Point3D::new(0.0, 2.0, 1.0),
        ],
        Some(vec![1.0, std::f64::consts::FRAC_1_SQRT_2, 1.0]),
    )
    .unwrap();
    assert!((arc.length() - std::f64::consts::PI).abs() < 1e-9);
}

#[test]
fn test_bounds_contain_control_points() {
    let bbox = bezier()
        .aabb()
        .expect("NURBS curve should have bounding box");
    assert_eq!(bbox.min(), Point3D::new(0.0, 0.0, 0.0));
    assert_eq!(bbox.max(), Point3D::new(2.0, 1.0, 1.0));
}

#[test]
fn test_curve_adaptive_params_line() {
    let curve = NurbsCurve3D::new(
        1,
        clamped_knot_vector(1, 2),
        &[Point3D::new(0.0, 0.0, 0.0), Point3D::new(1.0, 0.0, 0.0)],
        None,
    )
    .unwrap();

    let settings = AdaptiveTessellationSettings::default_with_tolerance(0.01);
    let params = curve.adaptive_params_curve(&settings);

    assert!(params.params.len() > settings.min_segments as usize);

    let (t_min, t_max) = curve.parameter_domain();
    assert!((params.params.first().unwrap() - t_min).abs() < 1e-10);
    assert!((params.params.last().unwrap() - t_max).abs() < 1e-10);
}
