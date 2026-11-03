//! LineSegment3D のテスト

use super::*;
use crate::{LineSegment3D, Point3D, Vector3D};
use geo_contracts::{
    default_distance_tolerance, Bounded, InfiniteLine3DConstructor, LineSegment3DConstructor,
    LineSegment3DContainment, LineSegment3DDistance, LineSegment3DEvaluation,
    LineSegment3DProjection, LineSegment3DProperties, LineSegment3DSupportLine,
};

use analysis::test_constants::TOLERANCE_F64;

/// 同じ support line 上で始点と終点を入れ替えた線分（始点のパラメータが終点より大きい）
fn reversed_on_support_line(segment: &LineSegment3D<f64>) -> LineSegment3D<f64> {
    LineSegment3D::from_support_line_and_constraint_points(
        *segment.support_line(),
        segment.end_point(),
        segment.start_point(),
    )
    .unwrap()
}

#[test]
fn test_line_segment3d_creation() {
    let start = Point3D::new(0.0, 0.0, 0.0);
    let end = Point3D::new(3.0, 4.0, 0.0);

    let segment = LineSegment3D::new(start, end).unwrap();
    assert_eq!(segment.start_point(), start);
    assert_eq!(segment.end_point(), end);
    assert_eq!(segment.length(), 5.0); // 3-4-5直角三角形
}

#[test]
fn test_line_segment3d_invalid_creation() {
    let point = Point3D::new(1.0, 2.0, 3.0);

    // 同じ点では線分作成不可
    assert!(LineSegment3D::new(point, point).is_none());
}

#[test]
fn test_line_segment3d_parametric_points() {
    let segment =
        LineSegment3D::new(Point3D::new(0.0, 0.0, 0.0), Point3D::new(10.0, 0.0, 0.0)).unwrap();

    // t=0で始点
    let p0 = segment.point_at_parameter(0.0);
    assert_eq!(p0, Some(Point3D::new(0.0, 0.0, 0.0)));

    // t=0.5で中点
    let p05 = segment.point_at_parameter(0.5);
    assert_eq!(p05, Some(Point3D::new(5.0, 0.0, 0.0)));

    // t=1で終点
    let p1 = segment.point_at_parameter(1.0);
    assert_eq!(p1, Some(Point3D::new(10.0, 0.0, 0.0)));
}

#[test]
fn test_line_segment3d_point_at_parameter_domain() {
    let segment = LineSegment3D::new(
        Point3D::new(0.0_f64, 0.0, 0.0),
        Point3D::new(10.0, 0.0, 0.0),
    )
    .unwrap();
    let margin = default_distance_tolerance::<f64>() / segment.length();

    // 距離トレランス内のはみ出しは外挿し、評価点は線分上と判定される
    for t in [-0.5 * margin, 1.0 + 0.5 * margin] {
        let point = segment.point_at_parameter(t).unwrap();
        assert!(segment.contains_point(point));
    }
    let beyond_end = segment.point_at_parameter(1.0 + 0.5 * margin).unwrap();
    assert!(beyond_end.x() > 10.0);

    // 距離トレランスを超えるはみ出しと有限でない値は評価できない
    assert!(segment.point_at_parameter(1.0 + 2.0 * margin).is_none());
    assert!(segment.point_at_parameter(-2.0 * margin).is_none());
    assert!(segment.point_at_parameter(1.5).is_none());
    assert!(segment.point_at_parameter(f64::NAN).is_none());
    assert!(segment.point_at_parameter(f64::INFINITY).is_none());
}

#[test]
fn test_line_segment3d_point_projection() {
    let segment =
        LineSegment3D::new(Point3D::new(0.0, 0.0, 0.0), Point3D::new(10.0, 0.0, 0.0)).unwrap();

    // 線分内の点への投影
    let point_above = Point3D::new(5.0, 3.0, 0.0);
    let projected = segment.closest_point(point_above);
    assert_eq!(projected, Point3D::new(5.0, 0.0, 0.0));

    // 線分外の点（始点側）
    let point_before = Point3D::new(-5.0, 2.0, 0.0);
    let projected_start = segment.closest_point(point_before);
    assert_eq!(projected_start, segment.start_point());

    // 線分外の点（終点側）
    let point_after = Point3D::new(15.0, 2.0, 0.0);
    let projected_end = segment.closest_point(point_after);
    assert_eq!(projected_end, segment.end_point());
}

#[test]
fn test_line_segment3d_distance_to_point() {
    let segment = LineSegment3D::new(
        Point3D::new(0.0_f64, 0.0, 0.0),
        Point3D::new(10.0, 0.0, 0.0),
    )
    .unwrap();

    // 線分上の点（距離0）
    let point_on_segment = Point3D::new(5.0, 0.0, 0.0);
    assert!(segment.distance_to_point(point_on_segment) < TOLERANCE_F64);

    // 線分に垂直な点
    let point_perpendicular = Point3D::new(5.0, 3.0, 0.0);
    assert!((segment.distance_to_point(point_perpendicular) - 3.0).abs() < TOLERANCE_F64);

    // 線分外の点（端点への距離）
    let point_beyond = Point3D::new(15.0, 0.0, 0.0);
    assert!((segment.distance_to_point(point_beyond) - 5.0).abs() < TOLERANCE_F64);
}

#[test]
fn test_line_segment3d_contains_point() {
    let segment =
        LineSegment3D::new(Point3D::new(0.0, 0.0, 0.0), Point3D::new(10.0, 0.0, 0.0)).unwrap();

    assert!(segment.contains_point(Point3D::new(5.0, 0.0, 0.0)));
    assert!(segment.contains_point(segment.start_point()));
    assert!(segment.contains_point(segment.end_point()));
    assert!(!segment.contains_point(Point3D::new(15.0, 0.0, 0.0)));
    assert!(!segment.contains_point(Point3D::new(5.0, 1.0, 0.0)));
}

#[test]
fn test_line_segment3d_reversed_projection() {
    let segment = LineSegment3D::new(
        Point3D::new(0.0_f64, 0.0, 0.0),
        Point3D::new(10.0, 0.0, 0.0),
    )
    .unwrap();
    let reversed = reversed_on_support_line(&segment);

    assert_eq!(reversed.start_point(), Point3D::new(10.0, 0.0, 0.0));
    assert_eq!(reversed.end_point(), Point3D::new(0.0, 0.0, 0.0));

    // 正規化パラメータは始点 (10, 0, 0) からの比率になる
    assert!(
        reversed
            .point_at_parameter(0.7)
            .unwrap()
            .distance_to(Point3D::new(3.0, 0.0, 0.0))
            < TOLERANCE_F64
    );

    // 投影は線分内の点はそのまま、範囲外は近い端点に制限される
    let projected = reversed.closest_point(Point3D::new(3.0, 2.0, 0.0));
    assert!(projected.distance_to(Point3D::new(3.0, 0.0, 0.0)) < TOLERANCE_F64);
    let beyond_start = reversed.closest_point(Point3D::new(15.0, 1.0, 0.0));
    assert!(beyond_start.distance_to(Point3D::new(10.0, 0.0, 0.0)) < TOLERANCE_F64);
    let beyond_end = reversed.closest_point(Point3D::new(-5.0, 1.0, 0.0));
    assert!(beyond_end.distance_to(Point3D::new(0.0, 0.0, 0.0)) < TOLERANCE_F64);
}

#[test]
fn test_aabb() {
    let segment =
        LineSegment3D::new(Point3D::new(1.0, 2.0, 3.0), Point3D::new(4.0, 6.0, 8.0)).unwrap();

    let bbox = segment.aabb().unwrap();
    assert_eq!(bbox.min(), Point3D::new(1.0, 2.0, 3.0));
    assert_eq!(bbox.max(), Point3D::new(4.0, 6.0, 8.0));
}

#[test]
fn test_line_segment3d_f32_compatibility() {
    let segment_f32 = LineSegment3D::new(
        Point3D::new(0.0f32, 0.0f32, 0.0f32),
        Point3D::new(3.0f32, 4.0f32, 0.0f32),
    )
    .unwrap();

    let segment_f64 = LineSegment3D::new(
        Point3D::new(0.0f64, 0.0f64, 0.0f64),
        Point3D::new(3.0f64, 4.0f64, 0.0f64),
    )
    .unwrap();

    assert_eq!(segment_f32.length(), 5.0f32);
    assert_eq!(segment_f64.length(), 5.0f64);
}

#[test]
fn public_endpoints_follow_ideal_support_line() {
    let line = InfiniteLine3D::new(Point3D::origin(), Vector3D::new(1.0_f64, 0.0, 0.0)).unwrap();
    let start = Point3D::new(0.0, 1.0, 0.0);
    let end = Point3D::new(2.0, 1.0, 0.0);

    let segment = LineSegment3D::from_support_line_and_constraint_points(line, start, end).unwrap();

    // 公開の端点は support line 上の理想端点、拘束端点は別に保持する
    assert_eq!(segment.constraint_start_point(), start);
    assert_eq!(segment.constraint_end_point(), end);
    assert_eq!(segment.start_point(), Point3D::new(0.0, 0.0, 0.0));
    assert_eq!(segment.end_point(), Point3D::new(2.0, 0.0, 0.0));
    assert_eq!(segment.start_param(), 0.0);
    assert_eq!(segment.end_param(), 2.0);
    assert_eq!(
        segment.point_at_parameter(0.5),
        Some(Point3D::new(1.0, 0.0, 0.0))
    );
    assert_eq!(segment.length(), 2.0);
}
