//! LineSegment2D のテスト

use crate::{LineSegment2D, Point2D, Vector2D};
use geo_contracts::Scalar;

#[cfg(test)]
mod tests {
    use super::*;
    use analysis::test_constants::{TOLERANCE_F32, TOLERANCE_F64};
    use geo_contracts::default_distance_tolerance;

    #[test]
    fn test_basic_creation() {
        // 基本的な線分作成
        let start = Point2D::new(0.0, 0.0);
        let end = Point2D::new(3.0, 4.0);
        let segment = LineSegment2D::new(start, end).unwrap();

        assert_eq!(segment.start_point(), start);
        assert_eq!(segment.end_point(), end);
        assert_eq!(segment.length(), 5.0); // 3-4-5三角形

        // 退化した線分（同じ点）
        let degenerate = LineSegment2D::new(start, start);
        assert!(degenerate.is_none());
    }

    #[test]
    fn test_creation_methods() {
        // 点と方向から作成
        let start = Point2D::new(1.0, 2.0);
        let direction = Vector2D::new(1.0, 0.0); // X軸方向
        let length = 5.0;
        let segment = LineSegment2D::from_point_direction_length(start, direction, length).unwrap();

        assert_eq!(segment.start_point(), start);
        assert_eq!(segment.end_point(), Point2D::new(6.0, 2.0));
        assert_eq!(segment.length(), length);
    }

    #[test]
    fn test_midpoint_and_direction() {
        let segment = LineSegment2D::new(Point2D::new(1.0, 2.0), Point2D::new(5.0, 6.0)).unwrap();

        // 中点
        let midpoint = segment.midpoint();
        assert!(midpoint.distance_to(&Point2D::new(3.0, 4.0)) < TOLERANCE_F64);

        // 方向ベクトル
        let direction = segment.direction();
        let expected_direction = Vector2D::new(4.0, 4.0).normalize();
        assert!((direction.x() - expected_direction.x()).abs() < TOLERANCE_F64);
        assert!((direction.y() - expected_direction.y()).abs() < TOLERANCE_F64);

        // ベクトル表現
        let vector = segment.vector();
        assert!((vector - Vector2D::new(4.0, 4.0)).length() < TOLERANCE_F64);
    }

    #[test]
    fn test_parametric_operations() {
        let segment = LineSegment2D::new(Point2D::new(0.0, 0.0), Point2D::new(4.0, 0.0)).unwrap();

        // パラメータでの点取得
        assert_eq!(
            segment.point_at_parameter(0.0),
            Some(Point2D::new(0.0, 0.0))
        );
        assert_eq!(
            segment.point_at_parameter(0.5),
            Some(Point2D::new(2.0, 0.0))
        );
        assert_eq!(
            segment.point_at_parameter(1.0),
            Some(Point2D::new(4.0, 0.0))
        );

        // 範囲外パラメータは評価できない
        assert!(segment.point_at_parameter(-0.5).is_none());
        assert!(segment.point_at_parameter(1.5).is_none());

        // 点からパラメータ取得
        assert!(
            (segment.parameter_for_point(&Point2D::new(1.0, 0.0)) - 0.25).abs() < TOLERANCE_F64
        );
        assert!(
            (segment.parameter_for_point(&Point2D::new(3.0, 0.0)) - 0.75).abs() < TOLERANCE_F64
        );
    }

    #[test]
    fn test_point_at_parameter_domain() {
        let segment =
            LineSegment2D::new(Point2D::new(0.0_f64, 0.0), Point2D::new(4.0, 0.0)).unwrap();
        let margin = default_distance_tolerance::<f64>() / segment.length();

        // 距離トレランス内のはみ出しは外挿し、評価点は線分上と判定される
        for t in [-0.5 * margin, 1.0 + 0.5 * margin] {
            let point = segment.point_at_parameter(t).unwrap();
            assert!(segment.contains_point(&point, default_distance_tolerance()));
        }
        let beyond_end = segment.point_at_parameter(1.0 + 0.5 * margin).unwrap();
        assert!(beyond_end.x() > 4.0);

        // 距離トレランスを超えるはみ出しと有限でない値は評価できない
        assert!(segment.point_at_parameter(1.0 + 2.0 * margin).is_none());
        assert!(segment.point_at_parameter(-2.0 * margin).is_none());
        assert!(segment.point_at_parameter(f64::NAN).is_none());
        assert!(segment.point_at_parameter(f64::INFINITY).is_none());
    }

    #[test]
    fn test_parameter_for_point_and_closest_parameter() {
        let segment =
            LineSegment2D::new(Point2D::new(0.0_f64, 0.0), Point2D::new(10.0, 0.0)).unwrap();

        // parameter_for_point は [0, 1] に制限しない
        assert!(
            (segment.parameter_for_point(&Point2D::new(15.0, 2.0)) - 1.5).abs() < TOLERANCE_F64
        );
        assert!(
            (segment.parameter_for_point(&Point2D::new(-5.0, 2.0)) + 0.5).abs() < TOLERANCE_F64
        );

        // closest_parameter は [0, 1] に制限する
        assert!((segment.closest_parameter(&Point2D::new(3.0, 2.0)) - 0.3).abs() < TOLERANCE_F64);
        assert!((segment.closest_parameter(&Point2D::new(15.0, 2.0)) - 1.0).abs() < TOLERANCE_F64);
        assert!(segment.closest_parameter(&Point2D::new(-5.0, 2.0)).abs() < TOLERANCE_F64);

        // reverse した線分では始点 (10, 0) からの比率になる
        let reversed = segment.reverse();
        assert!(
            (reversed.parameter_for_point(&Point2D::new(3.0, 0.0)) - 0.7).abs() < TOLERANCE_F64
        );
        assert!((reversed.closest_parameter(&Point2D::new(15.0, 0.0))).abs() < TOLERANCE_F64);
        assert!((reversed.closest_parameter(&Point2D::new(-5.0, 0.0)) - 1.0).abs() < TOLERANCE_F64);
        let t = reversed.closest_parameter(&Point2D::new(3.0, 2.0));
        let point = reversed.point_at_parameter(t).unwrap();
        assert!(point.distance_to(&Point2D::new(3.0, 0.0)) < TOLERANCE_F64);
    }

    #[test]
    fn test_point_operations() {
        let segment = LineSegment2D::new(Point2D::new(0.0, 0.0), Point2D::new(4.0, 0.0)).unwrap();

        // 点の投影
        let above_point = Point2D::new(2.0, 3.0);
        let projected = segment.closest_point(&above_point);
        assert_eq!(projected, Point2D::new(2.0, 0.0));

        // 線分外への投影（クランプされる）
        let outside_point = Point2D::new(-1.0, 2.0);
        let projected_outside = segment.closest_point(&outside_point);
        assert_eq!(projected_outside, Point2D::new(0.0, 0.0));

        // 距離計算
        assert!((segment.distance_to_point(&above_point) - 3.0).abs() < TOLERANCE_F64);
        assert!((segment.distance_to_point(&Point2D::new(2.0, 0.0)) - 0.0).abs() < TOLERANCE_F64);

        // 点の包含判定
        assert!(segment.contains_point(&Point2D::new(2.0, 0.0), TOLERANCE_F64));
        assert!(!segment.contains_point(&Point2D::new(2.0, 1.0), TOLERANCE_F64));
    }

    #[test]
    fn test_reverse() {
        let segment = LineSegment2D::new(Point2D::new(1.0, 1.0), Point2D::new(3.0, 1.0)).unwrap();

        let reversed = segment.reverse();
        assert_eq!(reversed.start_point(), Point2D::new(3.0, 1.0));
        assert_eq!(reversed.end_point(), Point2D::new(1.0, 1.0));
    }

    #[test]
    fn test_line_relationships() {
        let segment1 = LineSegment2D::new(Point2D::new(0.0, 0.0), Point2D::new(2.0, 0.0)).unwrap();

        let segment2 = LineSegment2D::new(Point2D::new(0.0, 1.0), Point2D::new(2.0, 1.0)).unwrap();

        let segment3 = LineSegment2D::new(Point2D::new(1.0, -1.0), Point2D::new(1.0, 1.0)).unwrap();

        // 平行判定（support line で判定）
        assert!(segment1.line().is_parallel(segment2.line()));
        assert!(!segment1.line().is_parallel(segment3.line()));

        // 垂直判定
        assert!(segment1.line().is_perpendicular(segment3.line()));
        assert!(!segment1.line().is_perpendicular(segment2.line()));

        // 共線判定
        let collinear_segment =
            LineSegment2D::new(Point2D::new(3.0, 0.0), Point2D::new(5.0, 0.0)).unwrap();
        assert!(segment1.line().is_coincident(collinear_segment.line()));
        assert!(!segment1.line().is_coincident(segment2.line()));
    }

    #[test]
    fn test_distance_between_segments() {
        let segment1 = LineSegment2D::new(Point2D::new(0.0, 0.0), Point2D::new(2.0, 0.0)).unwrap();

        let segment2 = LineSegment2D::new(Point2D::new(0.0, 3.0), Point2D::new(2.0, 3.0)).unwrap();

        let distance = segment1.distance_to_segment(&segment2);
        assert!((distance - 3.0).abs() < TOLERANCE_F64);
    }

    #[test]
    fn test_geometry_foundation_traits() {
        let segment = LineSegment2D::new(Point2D::new(1.0, 2.0), Point2D::new(5.0, 6.0)).unwrap();

        // CoreFoundation
        let bbox = segment.bounding_box();
        assert!(bbox.min_point().distance_to(&Point2D::new(1.0, 2.0)) < TOLERANCE_F64);
        assert!(bbox.max_point().distance_to(&Point2D::new(5.0, 6.0)) < TOLERANCE_F64);
    }

    #[test]
    fn test_basic_metrics_trait() {
        let segment = LineSegment2D::new(Point2D::new(0.0, 0.0), Point2D::new(3.0, 4.0)).unwrap();

        // BasicMetrics
        let length = segment.length();
        assert!((length - 5.0).abs() < TOLERANCE_F64);
    }

    #[test]
    fn test_basic_containment_trait() {
        let segment = LineSegment2D::new(Point2D::new(0.0, 0.0), Point2D::new(4.0, 0.0)).unwrap();

        // BasicContainment
        assert!(segment.contains_point(&Point2D::new(2.0, 0.0), TOLERANCE_F64));
        assert!(!segment.contains_point(&Point2D::new(2.0, 1.0), TOLERANCE_F64));

        let distance = segment.distance_to_point(&Point2D::new(2.0, 3.0));
        assert!((distance - 3.0).abs() < TOLERANCE_F64);
    }

    #[test]
    fn test_basic_parametric_trait() {
        let segment = LineSegment2D::new(Point2D::new(0.0, 0.0), Point2D::new(4.0, 0.0)).unwrap();

        // BasicParametric
        let (start_param, end_param) = segment.parameter_range();
        assert_eq!(start_param, 0.0);
        assert_eq!(end_param, 1.0);

        let mid_point = segment.point_at_parameter(0.5);
        assert_eq!(mid_point, Some(Point2D::new(2.0, 0.0)));

        let tangent = segment.tangent_at_parameter(0.5);
        assert_eq!(tangent, Vector2D::new(1.0, 0.0)); // 正規化された接線方向
    }

    #[test]
    fn test_basic_directional_trait() {
        let segment = LineSegment2D::new(Point2D::new(0.0, 0.0), Point2D::new(3.0, 4.0)).unwrap();

        // BasicDirectional
        let direction = segment.direction();
        let expected = Vector2D::new(3.0, 4.0).normalize();
        assert!((direction.x() - expected.x()).abs() < TOLERANCE_F64);
        assert!((direction.y() - expected.y()).abs() < TOLERANCE_F64);

        let reversed = segment.reverse();
        assert_eq!(reversed.start_point(), Point2D::new(3.0, 4.0));
        assert_eq!(reversed.end_point(), Point2D::new(0.0, 0.0));
    }

    #[test]
    fn test_f32_compatibility() {
        // f32での基本操作
        let segment =
            LineSegment2D::new(Point2D::new(0.0f32, 0.0f32), Point2D::new(3.0f32, 4.0f32)).unwrap();

        assert!((segment.length() - 5.0f32).abs() < TOLERANCE_F32);

        let midpoint = segment.midpoint();
        assert!((midpoint.x() - 1.5f32).abs() < TOLERANCE_F32);
        assert!((midpoint.y() - 2.0f32).abs() < TOLERANCE_F32);

        // foundation トレイト
        let bbox = segment.bounding_box();
        assert_eq!(bbox.min_point(), Point2D::new(0.0f32, 0.0f32));
        assert_eq!(bbox.max_point(), Point2D::new(3.0f32, 4.0f32));
    }
}
