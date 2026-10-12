//! `NurbsCurve2D` のテスト

use crate::knot::clamped_knot_vector;
use crate::NurbsCurve2D;
use geo_contracts::{
    NurbsCurve2DConstructor, NurbsCurve2DDerived, NurbsCurve2DEvaluation, NurbsCurve2DProperties,
};
use geo_contracts::{PrimitiveKind, PrimitiveMetadata};

#[test]
fn test_nurbs_curve_2d_creation() {
    use geo_contracts::NurbsCurve2DConstructor;
    let control_points = &[(0.0, 0.0), (1.0, 1.0), (2.0, 0.0)];
    let weights = Some(vec![1.0, 1.0, 1.0]);
    let knot_vector = vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0];
    let degree = 2;

    let curve = <NurbsCurve2D<f64> as NurbsCurve2DConstructor<f64>>::new(
        control_points,
        weights,
        knot_vector,
        degree,
    );
    assert!(curve.is_ok());

    let curve = curve.unwrap();
    assert_eq!(curve.degree(), 2);
    assert_eq!(curve.num_points(), 3);
}

#[test]
fn test_curve_evaluation() {
    use geo_contracts::NurbsCurve2DConstructor;
    let control_points = &[(0.0, 0.0), (1.0, 1.0), (2.0, 0.0)];
    let weights = Some(vec![1.0, 1.0, 1.0]);
    let knot_vector = vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0];
    let degree = 2;

    let curve = <NurbsCurve2D<f64> as NurbsCurve2DConstructor<f64>>::new(
        control_points,
        weights,
        knot_vector,
        degree,
    )
    .unwrap();

    // 開始点と終了点のテスト
    let start_point = curve.evaluate_at(0.0);
    let end_point = curve.evaluate_at(1.0);

    // 開始点は最初の制御点に近い
    assert!((start_point.x() - 0.0).abs() < 1e-10);
    assert!((start_point.y() - 0.0).abs() < 1e-10);

    // 終了点は最後の制御点に近い
    assert!((end_point.x() - 2.0).abs() < 1e-10);
    assert!((end_point.y() - 0.0).abs() < 1e-10);
}

#[test]
fn test_approximate_length() {
    use geo_contracts::NurbsCurve2DConstructor;
    let control_points = &[(0.0, 0.0), (1.0, 0.0), (2.0, 0.0)];
    let weights = Some(vec![1.0, 1.0, 1.0]);
    let knot_vector = vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0];
    let degree = 2;

    let curve = <NurbsCurve2D<f64> as NurbsCurve2DConstructor<f64>>::new(
        control_points,
        weights,
        knot_vector,
        degree,
    )
    .unwrap();
    let length = curve.approximate_length(100);

    // 直線に近い曲線なので長さは約2.0
    assert!((length - 2.0).abs() < 0.1);
}

#[test]
fn test_core_traits_constructor_new() {
    let control_points = &[(0.0, 0.0), (1.0, 1.0), (2.0, 0.0)];
    let weights = Some(vec![1.0, 1.0, 1.0]);
    let knots = vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0];
    let degree = 2;

    let curve = <NurbsCurve2D<f64> as NurbsCurve2DConstructor<f64>>::new(
        control_points,
        weights,
        knots,
        degree,
    );

    assert!(curve.is_ok());
    let curve = curve.unwrap();
    assert_eq!(
        <NurbsCurve2D<f64> as NurbsCurve2DProperties<f64>>::degree(&curve),
        2
    );
}

#[test]
fn test_core_traits_properties() {
    let control_points = &[(0.0, 0.0), (1.0, 1.0), (2.0, 0.0)];
    let knots = clamped_knot_vector(2, 3);
    let curve =
        <NurbsCurve2D<f64> as NurbsCurve2DConstructor<f64>>::new(control_points, None, knots, 2)
            .unwrap();

    assert_eq!(
        <NurbsCurve2D<f64> as NurbsCurve2DProperties<f64>>::degree(&curve),
        2
    );
    assert_eq!(
        <NurbsCurve2D<f64> as NurbsCurve2DProperties<f64>>::num_control_points(&curve),
        3
    );
    assert_eq!(
        <NurbsCurve2D<f64> as NurbsCurve2DProperties<f64>>::dimension(&curve),
        2
    );
}

#[test]
fn test_core_traits_evaluation_and_derived() {
    let control_points = &[(0.0, 0.0), (1.0, 0.0), (2.0, 0.0)];
    let knots = clamped_knot_vector(2, 3);
    let curve =
        <NurbsCurve2D<f64> as NurbsCurve2DConstructor<f64>>::new(control_points, None, knots, 2)
            .unwrap();

    // Evaluation trait 経由で端点評価できることを確認
    let start = <NurbsCurve2D<f64> as NurbsCurve2DEvaluation<f64>>::point_at(&curve, 0.0);
    let end = <NurbsCurve2D<f64> as NurbsCurve2DEvaluation<f64>>::point_at(&curve, 1.0);
    assert!((start.0 - 0.0).abs() < 1e-10);
    assert!(start.1.abs() < 1e-10);
    assert!((end.0 - 2.0).abs() < 1e-10);
    assert!(end.1.abs() < 1e-10);

    // length テスト
    let length = <NurbsCurve2D<f64> as NurbsCurve2DDerived<f64>>::length(&curve);
    assert!((length - 2.0).abs() < 0.1); // 直線なので約2.0
}

#[test]
fn test_nurbs_curve_2d_foundation() {
    let control_points = &[(0.0, 0.0), (1.0, 1.0), (2.0, 0.0)];
    let knots = clamped_knot_vector(2, 3);
    let curve =
        <NurbsCurve2D<f64> as NurbsCurve2DConstructor<f64>>::new(control_points, None, knots, 2)
            .unwrap();

    // PrimitiveKind の確認
    assert_eq!(curve.primitive_kind(), PrimitiveKind::NurbsCurve2D);

    // 測度の確認（2D曲線の場合は長さ）
    let length = curve.approximate_length(crate::constants::CURVE_LENGTH_SUBDIVISIONS);
    assert!(length > 0.0);
}
