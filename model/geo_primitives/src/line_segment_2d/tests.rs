//! LineSegment2D のテスト

use super::*;
use crate::{LineSegment2D, Point2D};
use geo_contracts::{
    default_distance_tolerance, Bounded, LineSegment2DConstructor, LineSegment2DContainment,
    LineSegment2DDistance, LineSegment2DEvaluation, LineSegment2DProjection,
    LineSegment2DProperties, ParallelRelation,
};

use analysis::test_constants::{TOLERANCE_F32, TOLERANCE_F64};

/// 同じ support line 上で始点と終点を入れ替えた線分（始点のパラメータが終点より大きい）
fn reversed_on_support_line(segment: &LineSegment2D<f64>) -> LineSegment2D<f64> {
    LineSegment2D {
        line: segment.line,
        start_param: segment.end_param,
        end_param: segment.start_param,
        start_point: segment.end_point,
        end_point: segment.start_point,
    }
}

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
    assert!(segment.point_at_parameter(f64::NAN).is_none());
}

#[test]
fn test_point_at_parameter_domain() {
    let segment = LineSegment2D::new(Point2D::new(0.0_f64, 0.0), Point2D::new(4.0, 0.0)).unwrap();
    let margin = default_distance_tolerance::<f64>() / segment.length();

    // 距離トレランス内のはみ出しは外挿し、評価点は線分上と判定される
    for t in [-0.5 * margin, 1.0 + 0.5 * margin] {
        let point = segment.point_at_parameter(t).unwrap();
        assert!(segment.contains_point(point));
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
fn test_point_operations() {
    let segment = LineSegment2D::new(Point2D::new(0.0, 0.0), Point2D::new(4.0, 0.0)).unwrap();

    // 点の投影
    let above_point = Point2D::new(2.0, 3.0);
    let projected = segment.closest_point(above_point);
    assert_eq!(projected, Point2D::new(2.0, 0.0));

    // 線分外への投影（クランプされる）
    let outside_point = Point2D::new(-1.0, 2.0);
    let projected_outside = segment.closest_point(outside_point);
    assert_eq!(projected_outside, Point2D::new(0.0, 0.0));

    // 距離計算
    assert!((segment.distance_to_point(above_point) - 3.0).abs() < TOLERANCE_F64);
    assert!(segment.distance_to_point(Point2D::new(2.0, 0.0)).abs() < TOLERANCE_F64);

    // 点の包含判定（既定の許容誤差）
    assert!(segment.contains_point(Point2D::new(2.0, 0.0)));
    assert!(!segment.contains_point(Point2D::new(2.0, 1.0)));
    assert!(!segment.contains_point(Point2D::new(5.0, 0.0)));
}

#[test]
fn test_closest_point_on_reversed_segment() {
    let segment = LineSegment2D::new(Point2D::new(0.0_f64, 0.0), Point2D::new(10.0, 0.0)).unwrap();
    let reversed = reversed_on_support_line(&segment);

    assert_eq!(reversed.start_point(), Point2D::new(10.0, 0.0));
    assert_eq!(reversed.end_point(), Point2D::new(0.0, 0.0));

    // 始点のパラメータが終点より大きくても、線分の範囲で制限する
    assert!(
        reversed
            .closest_point(Point2D::new(3.0, 2.0))
            .distance_to(Point2D::new(3.0, 0.0))
            < TOLERANCE_F64
    );
    assert!(
        reversed
            .closest_point(Point2D::new(15.0, 0.0))
            .distance_to(Point2D::new(10.0, 0.0))
            < TOLERANCE_F64
    );
    assert!(
        reversed
            .closest_point(Point2D::new(-5.0, 0.0))
            .distance_to(Point2D::new(0.0, 0.0))
            < TOLERANCE_F64
    );

    // 正規化パラメータは始点 (10, 0) からの比率になる
    let point = reversed.point_at_parameter(0.7).unwrap();
    assert!(point.distance_to(Point2D::new(3.0, 0.0)) < TOLERANCE_F64);
}

#[test]
fn test_support_lines_are_parallel() {
    let segment1 = LineSegment2D::new(Point2D::new(0.0, 0.0), Point2D::new(2.0, 0.0)).unwrap();
    let segment2 = LineSegment2D::new(Point2D::new(0.0, 1.0), Point2D::new(2.0, 1.0)).unwrap();
    let segment3 = LineSegment2D::new(Point2D::new(1.0, -1.0), Point2D::new(1.0, 1.0)).unwrap();

    assert!(segment1.line.is_parallel_to(&segment2.line));
    assert!(!segment1.line.is_parallel_to(&segment3.line));
}

#[test]
fn test_aabb() {
    let segment = LineSegment2D::new(Point2D::new(1.0, 2.0), Point2D::new(5.0, 6.0)).unwrap();

    let bbox = segment.aabb().unwrap();
    assert!(bbox.min_point().distance_to(Point2D::new(1.0, 2.0)) < TOLERANCE_F64);
    assert!(bbox.max_point().distance_to(Point2D::new(5.0, 6.0)) < TOLERANCE_F64);
}

#[test]
fn test_f32_compatibility() {
    let segment =
        LineSegment2D::new(Point2D::new(0.0f32, 0.0f32), Point2D::new(3.0f32, 4.0f32)).unwrap();

    assert!((segment.length() - 5.0f32).abs() < TOLERANCE_F32);

    let bbox = segment.aabb().unwrap();
    assert_eq!(bbox.min_point(), Point2D::new(0.0f32, 0.0f32));
    assert_eq!(bbox.max_point(), Point2D::new(3.0f32, 4.0f32));
}
