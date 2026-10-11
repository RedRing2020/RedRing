//! `NurbsCurve3D` のテスト

use super::*;
use crate::clamped_knot_vector;
use crate::constants;
use crate::{AdaptiveTessellationSettings, NurbsCurveAdaptiveTessellation};
use geo_contracts::default_kernel_numerical_zero_tolerance;
use geo_contracts::{Bounded, PrimitiveKind, PrimitiveMetadata};

#[test]
fn test_nurbs_curve_3d_creation() {
    use geo_contracts::NurbsCurve3DConstructor;
    let control_points = vec![(0.0, 0.0, 0.0), (1.0, 1.0, 0.0), (2.0, 0.0, 1.0)];
    let weights = Some(vec![1.0, 1.0, 1.0]);
    let knot_vector = vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0];
    let degree = 2;

    let curve = <NurbsCurve3D<f64> as NurbsCurve3DConstructor<f64>>::new(
        degree,
        knot_vector,
        control_points,
        weights,
    );
    assert!(curve.is_ok());

    let curve = curve.unwrap();
    assert_eq!(curve.degree(), 2);
    assert_eq!(curve.num_points(), 3);
}

#[test]
fn test_curve_3d_evaluation() {
    use geo_contracts::NurbsCurve3DConstructor;
    let control_points = vec![(0.0, 0.0, 0.0), (1.0, 1.0, 1.0), (2.0, 0.0, 0.0)];
    let weights = Some(vec![1.0, 1.0, 1.0]);
    let knot_vector = vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0];
    let degree = 2;

    let curve = <NurbsCurve3D<f64> as NurbsCurve3DConstructor<f64>>::new(
        degree,
        knot_vector,
        control_points,
        weights,
    )
    .unwrap();

    // 開始点と終了点のテスト
    let start_point = curve.evaluate_at(0.0);
    let end_point = curve.evaluate_at(1.0);

    // 開始点は最初の制御点に近い
    assert!((start_point.x() - 0.0).abs() < 1e-10);
    assert!((start_point.y() - 0.0).abs() < 1e-10);
    assert!((start_point.z() - 0.0).abs() < 1e-10);

    // 終了点は最後の制御点に近い
    assert!((end_point.x() - 2.0).abs() < 1e-10);
    assert!((end_point.y() - 0.0).abs() < 1e-10);
    assert!((end_point.z() - 0.0).abs() < 1e-10);
}

#[test]
fn test_tangent_calculation() {
    use geo_contracts::NurbsCurve3DConstructor;
    let control_points = vec![(0.0, 0.0, 0.0), (1.0, 0.0, 0.0), (2.0, 0.0, 0.0)];
    let weights = Some(vec![1.0, 1.0, 1.0]);
    let knot_vector = vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0];
    let degree = 2;

    let curve = <NurbsCurve3D<f64> as NurbsCurve3DConstructor<f64>>::new(
        degree,
        knot_vector,
        control_points,
        weights,
    )
    .unwrap();
    let tangent = curve.tangent_at(0.5);

    // 直線に近い曲線なのでX軸方向の接線
    assert!((tangent.x() - 1.0).abs() < 0.1);
    assert!(tangent.y().abs() < 0.1);
    assert!(tangent.z().abs() < 0.1);
}

#[test]
fn test_approximate_length_3d() {
    use geo_contracts::NurbsCurve3DConstructor;
    let control_points = vec![(0.0, 0.0, 0.0), (1.0, 0.0, 0.0), (2.0, 0.0, 0.0)];
    let weights = Some(vec![1.0, 1.0, 1.0]);
    let knot_vector = vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0];
    let degree = 2;

    let curve = <NurbsCurve3D<f64> as NurbsCurve3DConstructor<f64>>::new(
        degree,
        knot_vector,
        control_points,
        weights,
    )
    .unwrap();
    let length = curve.approximate_length(100);

    // 直線に近い曲線なので長さは約2.0
    assert!((length - 2.0).abs() < 0.1);
}

#[test]
fn test_nurbs_curve_3d_foundation() {
    use geo_contracts::NurbsCurve3DConstructor;
    // 簡単なNURBS曲線を作成
    let control_points = vec![(0.0, 0.0, 0.0), (1.0, 1.0, 0.0), (2.0, 0.0, 0.0)];

    let knots = clamped_knot_vector(2, 3);
    let curve =
        <NurbsCurve3D<f64> as NurbsCurve3DConstructor<f64>>::new(2, knots, control_points, None)
            .unwrap();

    // PrimitiveKind の確認
    assert_eq!(curve.primitive_kind(), PrimitiveKind::NurbsCurve3D);

    // AABB の確認
    let bbox = curve.aabb().expect("NURBS curve should have bounding box");
    let min = bbox.min();
    let max = bbox.max();

    // 境界ボックスは制御点を含む
    assert!((min.x() - 0.0).abs() < 1e-10);
    assert!((min.y() - 0.0).abs() < 1e-10);
    assert!((min.z() - 0.0).abs() < 1e-10);
    assert!((max.x() - 2.0).abs() < 1e-10);
    assert!((max.y() - 1.0).abs() < 1e-10);
    assert!((max.z() - 0.0).abs() < 1e-10);

    // 測度（曲線長）の確認
    let length = curve.approximate_length(constants::CURVE_LENGTH_SUBDIVISIONS);
    assert!(length > 0.0);
}

#[test]
fn test_core_traits_constructor_new() {
    use geo_contracts::NurbsCurve3DConstructor;

    // トレイト経由で作成（型を明示）
    let degree = 2;
    let knots = vec![0.0_f64, 0.0, 0.0, 1.0, 1.0, 1.0];
    let control_points = vec![(0.0_f64, 0.0, 0.0), (0.5, 1.0, 0.0), (1.0, 0.0, 0.0)];
    let weights = None;

    let result = <NurbsCurve3D<f64> as NurbsCurve3DConstructor<f64>>::new(
        degree,
        knots,
        control_points,
        weights,
    );
    assert!(result.is_ok());

    let curve = result.unwrap();
    assert_eq!(curve.primitive_kind(), PrimitiveKind::NurbsCurve3D);
}

#[test]
fn test_core_traits_from_bezier() {
    use geo_contracts::NurbsCurve3DConstructor;

    let control_points = vec![(0.0_f64, 0.0, 0.0), (0.5, 1.0, 0.0), (1.0, 0.0, 0.0)];
    let result = <NurbsCurve3D<f64> as NurbsCurve3DConstructor<f64>>::from_bezier(control_points);
    assert!(result.is_ok());

    let curve = result.unwrap();
    assert_eq!(curve.primitive_kind(), PrimitiveKind::NurbsCurve3D);
}

#[test]
fn test_core_traits_line_segment() {
    use geo_contracts::NurbsCurve3DConstructor;

    let start = (0.0_f64, 0.0, 0.0);
    let end = (1.0, 1.0, 1.0);
    let result = <NurbsCurve3D<f64> as NurbsCurve3DConstructor<f64>>::line_segment(start, end);
    assert!(result.is_ok());

    let curve = result.unwrap();
    assert_eq!(curve.primitive_kind(), PrimitiveKind::NurbsCurve3D);
}

#[test]
fn test_core_traits_line_segment_rejects_identical_points() {
    use geo_contracts::NurbsCurve3DConstructor;

    let start = (1.0_f64, 2.0, 3.0);
    let end = (1.0_f64, 2.0, 3.0);
    let result = <NurbsCurve3D<f64> as NurbsCurve3DConstructor<f64>>::line_segment(start, end);

    assert!(result.is_err());
}

#[test]
fn test_core_traits_line_segment_f64_kernel_tolerance_boundary() {
    use geo_contracts::NurbsCurve3DConstructor;

    let zero_tol = default_kernel_numerical_zero_tolerance::<f64>();
    let start = (0.0_f64, 0.0, 0.0);
    let inside = (zero_tol * 0.5, 0.0, 0.0);
    let inside_result =
        <NurbsCurve3D<f64> as NurbsCurve3DConstructor<f64>>::line_segment(start, inside);
    assert!(inside_result.is_err());

    let outside = (zero_tol * 2.0, 0.0, 0.0);
    let outside_result =
        <NurbsCurve3D<f64> as NurbsCurve3DConstructor<f64>>::line_segment(start, outside);
    assert!(outside_result.is_ok());
}

#[test]
fn test_core_traits_line_segment_f32_kernel_tolerance_boundary() {
    use geo_contracts::NurbsCurve3DConstructor;

    let zero_tol = default_kernel_numerical_zero_tolerance::<f32>();
    let start = (0.0_f32, 0.0, 0.0);

    let inside = (zero_tol * 0.5, 0.0, 0.0);
    let inside_result =
        <NurbsCurve3D<f32> as NurbsCurve3DConstructor<f32>>::line_segment(start, inside);
    assert!(inside_result.is_err());

    let outside = (zero_tol * 2.0, 0.0, 0.0);
    let outside_result =
        <NurbsCurve3D<f32> as NurbsCurve3DConstructor<f32>>::line_segment(start, outside);
    assert!(outside_result.is_ok());
}

#[test]
fn test_core_traits_properties() {
    use geo_contracts::{NurbsCurve3DConstructor, NurbsCurve3DProperties};

    let curve = <NurbsCurve3D<f64> as NurbsCurve3DConstructor<f64>>::from_bezier(vec![
        (0.0, 0.0, 0.0),
        (0.5, 1.0, 0.0),
        (1.0, 0.0, 0.0),
    ])
    .unwrap();

    // Propertiesトレイトメソッド確認
    assert_eq!(curve.degree(), 2);
    assert_eq!(curve.control_points_count(), 3);
    assert!(!curve.is_rational());

    let (t_min, t_max) = curve.parameter_domain();
    assert!((t_min - 0.0).abs() < 1e-10);
    assert!((t_max - 1.0).abs() < 1e-10);
}

#[test]
fn test_core_traits_measure() {
    use analysis::test_constants::INTEGRATION_TOLERANCE_STRICT;
    use geo_contracts::{NurbsCurve3DConstructor, NurbsCurve3DDerived, NurbsCurve3DEvaluation};

    let curve = <NurbsCurve3D<f64> as NurbsCurve3DConstructor<f64>>::line_segment(
        (0.0, 0.0, 0.0),
        (3.0, 0.0, 0.0),
    )
    .unwrap();

    // Measureトレイトメソッド確認
    let tolerance = INTEGRATION_TOLERANCE_STRICT;
    let total_length = curve.arc_length_total(tolerance);
    assert!((total_length - 3.0).abs() < 1e-3);

    let half_length = curve.arc_length(0.0, 0.5, tolerance);
    assert!((half_length - 1.5).abs() < 1e-3);

    let point_opt = curve.evaluate(0.5);
    assert!(point_opt.is_some());
    let point = point_opt.unwrap();
    assert!((point.0 - 1.5).abs() < 1e-10);
}

#[test]
fn test_evaluate_checked_rejects_nan_parameter() {
    use geo_contracts::{NurbsCurve3DConstructor, NurbsCurve3DEvaluation};

    let curve = <NurbsCurve3D<f64> as NurbsCurve3DConstructor<f64>>::line_segment(
        (0.0, 0.0, 0.0),
        (1.0, 0.0, 0.0),
    )
    .unwrap();

    let value =
        <NurbsCurve3D<f64> as NurbsCurve3DEvaluation<f64>>::evaluate_checked(&curve, f64::NAN);

    assert!(value.is_none());

    let (u_min, u_max) = curve.parameter_domain();
    let below_domain =
        <NurbsCurve3D<f64> as NurbsCurve3DEvaluation<f64>>::evaluate_checked(&curve, u_min - 0.1);
    let above_domain =
        <NurbsCurve3D<f64> as NurbsCurve3DEvaluation<f64>>::evaluate_checked(&curve, u_max + 0.1);

    assert!(below_domain.is_none());
    assert!(above_domain.is_none());
}

#[test]
fn test_rough_bbox() {
    use geo_contracts::NurbsCurve3DConstructor;
    let control_points = vec![(0.0, 0.0, 0.0), (1.0, 2.0, 0.0), (2.0, 0.0, 0.0)];

    let knots = clamped_knot_vector(2, 3);
    let curve =
        <NurbsCurve3D<f64> as NurbsCurve3DConstructor<f64>>::new(2, knots, control_points, None)
            .unwrap();

    let bbox = curve.bounding_box_with_options(AabbOptions::Rough);

    // 制御点ベースなので、制御点を含む
    assert!((bbox.min().x() - 0.0).abs() < 1e-10);
    assert!((bbox.max().x() - 2.0).abs() < 1e-10);
    assert!((bbox.max().y() - 2.0).abs() < 1e-10); // 制御点(1,2,0)を含む
}

#[test]
fn test_precise_bbox() {
    use geo_contracts::NurbsCurve3DConstructor;
    let control_points = vec![(0.0, 0.0, 0.0), (1.0, 2.0, 0.0), (2.0, 0.0, 0.0)];

    let knots = clamped_knot_vector(2, 3);
    let curve =
        <NurbsCurve3D<f64> as NurbsCurve3DConstructor<f64>>::new(2, knots, control_points, None)
            .unwrap();

    let precise_bbox = curve.bounding_box_with_options(AabbOptions::Precise { tolerance: 0.01 });

    let rough_bbox = curve.bounding_box_with_options(AabbOptions::Rough);

    // 精密版は制御点ベースより小さいか同等
    assert!(precise_bbox.max().y() <= rough_bbox.max().y());
}

#[test]
fn test_adaptive_bbox() {
    use geo_contracts::NurbsCurve3DConstructor;
    let control_points = vec![(0.0, 0.0, 0.0), (1.0, 1.0, 0.0), (2.0, 0.0, 0.0)];

    let knots = clamped_knot_vector(2, 3);
    let curve =
        <NurbsCurve3D<f64> as NurbsCurve3DConstructor<f64>>::new(2, knots, control_points, None)
            .unwrap();

    // 分割数を変えてテスト
    let bbox_100 = curve.bounding_box_with_options(AabbOptions::Adaptive {
        max_subdivisions: 100,
    });
    let bbox_500 = curve.bounding_box_with_options(AabbOptions::Adaptive {
        max_subdivisions: 500,
    });

    // より細かい分割は精度が向上（または同等）
    let diff_100 = bbox_100.max().y() - bbox_100.min().y();
    let diff_500 = bbox_500.max().y() - bbox_500.min().y();

    assert!(diff_500 <= diff_100 * 1.01); // 誤差を考慮
}

#[test]
fn test_bbox_comparison() {
    use geo_contracts::NurbsCurve3DConstructor;
    let control_points = vec![
        (0.0, 0.0, 0.0),
        (1.0, 2.0, 1.0),
        (2.0, 0.0, 0.0),
        (3.0, 0.0, 0.0),
    ];

    let knots = clamped_knot_vector(3, 4);
    let curve =
        <NurbsCurve3D<f64> as NurbsCurve3DConstructor<f64>>::new(3, knots, control_points, None)
            .unwrap();

    let rough = curve.bounding_box_with_options(AabbOptions::Rough);
    let precise = curve.bounding_box_with_options(AabbOptions::Precise { tolerance: 0.01 });
    let adaptive = curve.bounding_box_with_options(AabbOptions::Adaptive {
        max_subdivisions: 1000,
    });

    // Rough は最も大きい（保守的）
    assert!(rough.max().y() >= precise.max().y());
    assert!(rough.max().y() >= adaptive.max().y());

    // Precise と Adaptive はほぼ同等（トレランス依存）
    let diff = (precise.max().y() - adaptive.max().y()).abs();
    assert!(diff < 0.1); // 適度な誤差範囲
}

#[test]
fn test_curve_adaptive_params_line() {
    use geo_contracts::NurbsCurve3DConstructor;

    let control_points = vec![(0.0, 0.0, 0.0), (1.0, 0.0, 0.0)];
    let knots = clamped_knot_vector(1, 2);
    let curve =
        <NurbsCurve3D<f64> as NurbsCurve3DConstructor<f64>>::new(1, knots, control_points, None)
            .unwrap();

    let settings = AdaptiveTessellationSettings::default_with_tolerance(0.01);
    let params = curve.adaptive_params_curve(&settings);

    assert!(params.params.len() > settings.min_segments as usize);

    let (t_min, t_max) = curve.parameter_domain();
    assert!((params.params.first().unwrap() - t_min).abs() < 1e-10);
    assert!((params.params.last().unwrap() - t_max).abs() < 1e-10);
}
