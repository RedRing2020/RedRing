//! `NurbsCurve2D` のテスト

use crate::knot::clamped_knot_vector;
use crate::NurbsCurve2D;
use geo_contracts::{
    Bounded, NurbsCurve2DConstructor, NurbsCurve2DDerived, NurbsCurve2DEvaluation,
    NurbsCurve2DProperties, NurbsError, PrimitiveKind, PrimitiveMetadata,
};
use geo_core::{Point2D, Vector2D};
use std::f64::consts::{FRAC_1_SQRT_2, FRAC_PI_2};

/// 制御点 (0, 0), (1, 1), (2, 0) の 2 次 Bézier 曲線
fn parabola() -> NurbsCurve2D<f64> {
    NurbsCurve2D::new(
        2,
        clamped_knot_vector(2, 3),
        &[
            Point2D::new(0.0, 0.0),
            Point2D::new(1.0, 1.0),
            Point2D::new(2.0, 0.0),
        ],
        None,
    )
    .unwrap()
}

/// 単位円の 1/4（(1, 0) から (0, 1)）の有理 2 次曲線
fn quarter_circle() -> NurbsCurve2D<f64> {
    NurbsCurve2D::new(
        2,
        clamped_knot_vector(2, 3),
        &[
            Point2D::new(1.0, 0.0),
            Point2D::new(1.0, 1.0),
            Point2D::new(0.0, 1.0),
        ],
        Some(vec![1.0, FRAC_1_SQRT_2, 1.0]),
    )
    .unwrap()
}

#[test]
fn test_creation_and_properties() {
    let curve = parabola();
    assert_eq!(curve.primitive_kind(), PrimitiveKind::NurbsCurve2D);
    assert_eq!(curve.degree(), 2);
    assert_eq!(curve.control_point_count(), 3);
    assert_eq!(curve.control_point(1), Some(Point2D::new(1.0, 1.0)));
    assert_eq!(curve.control_point(3), None);
    assert_eq!(curve.weight(0), Some(1.0));
    assert_eq!(curve.weights(), None);
    assert_eq!(curve.coordinates(), &[0.0, 0.0, 1.0, 1.0, 2.0, 0.0]);
    assert_eq!(curve.parameter_domain(), (0.0, 1.0));
}

#[test]
fn test_point_and_derivative() {
    let curve = parabola();

    // C(t) = (2t, 2t(1-t))、C'(t) = (2, 2 - 4t)
    for t in [0.0, 0.25, 0.5, 1.0] {
        let point = curve.point_at_parameter(t).unwrap();
        assert!(point.distance_to(Point2D::new(2.0 * t, 2.0 * t * (1.0 - t))) < 1e-12);
        let derivative = curve.derivative_at(t).unwrap();
        assert!((derivative - Vector2D::new(2.0, 2.0 - 4.0 * t)).length() < 1e-12);
    }
}

#[test]
fn test_rational_point_and_derivative() {
    let curve = quarter_circle();

    // 有理曲線の点は単位円上、接線は半径に直交
    for t in [0.0, 0.3, 0.5, 0.8, 1.0] {
        let point = curve.point_at_parameter(t).unwrap();
        assert!((point.distance_to(Point2D::origin()) - 1.0).abs() < 1e-12);
        let derivative = curve.derivative_at(t).unwrap();
        assert!((derivative.x() * point.x() + derivative.y() * point.y()).abs() < 1e-12);
    }
}

#[test]
fn test_evaluation_outside_domain() {
    let curve = parabola();
    assert!(curve.point_at_parameter(-0.1).is_none());
    assert!(curve.point_at_parameter(1.1).is_none());
    assert!(curve.point_at_parameter(f64::NAN).is_none());
    assert!(curve.derivative_at(1.1).is_none());
}

#[test]
fn test_length() {
    // 直線上に並んだ制御点の曲線は線分の長さ
    let line: NurbsCurve2D<f64> = NurbsCurve2D::new(
        2,
        clamped_knot_vector(2, 3),
        &[
            Point2D::new(0.0, 0.0),
            Point2D::new(1.0, 0.0),
            Point2D::new(2.0, 0.0),
        ],
        None,
    )
    .unwrap();
    assert!((line.length() - 2.0).abs() < 1e-9);

    // 単位円の 1/4 の長さは π/2
    assert!((quarter_circle().length() - FRAC_PI_2).abs() < 1e-9);
}

#[test]
fn test_invalid_creation() {
    let points = [
        Point2D::new(0.0, 0.0),
        Point2D::new(1.0, 1.0),
        Point2D::new(2.0, 0.0),
    ];
    let knots = clamped_knot_vector::<f64>(2, 3);

    assert_eq!(
        NurbsCurve2D::new(3, knots.clone(), &points, None).unwrap_err(),
        NurbsError::InsufficientControlPoints {
            actual: 3,
            required: 4,
            degree: 3,
        }
    );
    assert_eq!(
        NurbsCurve2D::new(2, knots.clone(), &points, Some(vec![1.0, 1.0])).unwrap_err(),
        NurbsError::WeightCountMismatch {
            actual: 2,
            expected: 3,
        }
    );
    assert_eq!(
        NurbsCurve2D::new(2, knots, &points, Some(vec![1.0, 0.0, 1.0])).unwrap_err(),
        NurbsError::NonPositiveWeight {
            index: 1,
            weight: 0.0,
        }
    );
    assert_eq!(
        NurbsCurve2D::new(2, vec![0.0, 0.0, 0.0, 1.0, 1.0], &points, None).unwrap_err(),
        NurbsError::KnotCountMismatch {
            actual: 5,
            expected: 6,
        }
    );
}

#[test]
fn test_bounds_contain_control_points() {
    let bbox = parabola().aabb().unwrap();
    assert_eq!(bbox.min_point(), Point2D::new(0.0, 0.0));
    assert_eq!(bbox.max_point(), Point2D::new(2.0, 1.0));
}
