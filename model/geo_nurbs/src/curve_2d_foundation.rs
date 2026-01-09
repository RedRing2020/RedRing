//! `NurbsCurve2D` Foundation Pattern 統合テスト
//!
//! Core Traits (Constructor/Properties/Measure) の動作検証

#[cfg(test)]
mod tests {
    use crate::knot::clamped_knot_vector;
    use crate::NurbsCurve2D;
    use geo_foundation::{
        Bounded, ExtensionFoundation, NurbsCurve2DConstructor, NurbsCurve2DMeasure,
        NurbsCurve2DProperties, PrimitiveKind,
    };

    // ============================================================================
    // Core Traits Constructor テスト
    // ============================================================================

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
    fn test_core_traits_from_control_points() {
        let control_points = &[(0.0, 0.0), (1.0, 1.0), (2.0, 0.0)];
        let degree = 2;

        let curve = <NurbsCurve2D<f64> as NurbsCurve2DConstructor<f64>>::from_control_points(
            control_points,
            degree,
        );

        assert!(curve.is_ok());
        let curve = curve.unwrap();
        assert_eq!(
            <NurbsCurve2D<f64> as NurbsCurve2DProperties<f64>>::degree(&curve),
            2
        );
        assert_eq!(
            <NurbsCurve2D<f64> as NurbsCurve2DProperties<f64>>::num_control_points(&curve),
            3
        );
    }

    #[test]
    fn test_core_traits_unit_line() {
        let curve = <NurbsCurve2D<f64> as NurbsCurve2DConstructor<f64>>::unit_line();

        assert_eq!(
            <NurbsCurve2D<f64> as NurbsCurve2DProperties<f64>>::degree(&curve),
            1
        );

        let start = <NurbsCurve2D<f64> as NurbsCurve2DMeasure<f64>>::point_at(&curve, 0.0);
        let end = <NurbsCurve2D<f64> as NurbsCurve2DMeasure<f64>>::point_at(&curve, 1.0);

        assert!((start.0 - 0.0).abs() < 1e-10);
        assert!((start.1 - 0.0).abs() < 1e-10);
        assert!((end.0 - 1.0).abs() < 1e-10);
        assert!((end.1 - 0.0).abs() < 1e-10);
    }

    // ============================================================================
    // Core Traits Properties テスト
    // ============================================================================

    #[test]
    fn test_core_traits_properties() {
        let control_points = &[(0.0, 0.0), (1.0, 1.0), (2.0, 0.0)];
        let knots = clamped_knot_vector(2, 3);
        let curve = <NurbsCurve2D<f64> as NurbsCurve2DConstructor<f64>>::new(
            control_points,
            None,
            knots,
            2,
        )
        .unwrap();

        assert_eq!(
            <NurbsCurve2D<f64> as NurbsCurve2DProperties<f64>>::degree(&curve),
            2
        );
        assert_eq!(
            <NurbsCurve2D<f64> as NurbsCurve2DProperties<f64>>::num_control_points(&curve),
            3
        );
        assert!(!<NurbsCurve2D<f64> as NurbsCurve2DProperties<f64>>::is_rational(&curve));
        assert_eq!(
            <NurbsCurve2D<f64> as NurbsCurve2DProperties<f64>>::dimension(&curve),
            2
        );
    }

    // ============================================================================
    // Core Traits Measure テスト
    // ============================================================================

    #[test]
    fn test_core_traits_measure() {
        let control_points = &[(0.0, 0.0), (1.0, 0.0), (2.0, 0.0)];
        let knots = clamped_knot_vector(2, 3);
        let curve = <NurbsCurve2D<f64> as NurbsCurve2DConstructor<f64>>::new(
            control_points,
            None,
            knots,
            2,
        )
        .unwrap();

        // point_at テスト
        let mid_point = <NurbsCurve2D<f64> as NurbsCurve2DMeasure<f64>>::point_at(&curve, 0.5);
        assert!((mid_point.0 - 1.0).abs() < 0.1);

        // tangent_at テスト
        let tangent = <NurbsCurve2D<f64> as NurbsCurve2DMeasure<f64>>::tangent_at(&curve, 0.5);
        assert!((tangent.0 - 1.0).abs() < 0.1); // X方向の接線

        // length テスト
        let length = <NurbsCurve2D<f64> as NurbsCurve2DMeasure<f64>>::length(&curve);
        assert!((length - 2.0).abs() < 0.1); // 直線なので約2.0

        // curvature_at テスト
        let curvature = <NurbsCurve2D<f64> as NurbsCurve2DMeasure<f64>>::curvature_at(&curve, 0.5);
        assert!(curvature.abs() < 0.1); // 直線なので曲率は0に近い
    }

    // ============================================================================
    // Extension Foundation テスト
    // ============================================================================

    #[test]
    fn test_nurbs_curve_2d_foundation() {
        let control_points = &[(0.0, 0.0), (1.0, 1.0), (2.0, 0.0)];
        let knots = clamped_knot_vector(2, 3);
        let curve = <NurbsCurve2D<f64> as NurbsCurve2DConstructor<f64>>::new(
            control_points,
            None,
            knots,
            2,
        )
        .unwrap();

        // PrimitiveKind の確認
        assert_eq!(curve.primitive_kind(), PrimitiveKind::NurbsCurve2D);

        // 測度の確認（2D曲線の場合は長さ）
        let measure = curve.measure();
        assert!(measure.is_some());
        let length = measure.unwrap();
        assert!(length > 0.0);
    }
}
