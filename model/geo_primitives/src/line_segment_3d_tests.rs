//! LineSegment3D のテスト

use crate::{LineSegment3D, Point3D, Vector3D};

#[cfg(test)]
mod tests {
    use super::*;
    use analysis::test_constants::TOLERANCE_F64;
    use geo_contracts::LineSegment3DDerived;

    #[test]
    fn test_line_segment3d_creation() {
        let start = Point3D::new(0.0, 0.0, 0.0);
        let end = Point3D::new(3.0, 4.0, 0.0);

        let segment = LineSegment3D::new(start, end).unwrap();
        assert_eq!(segment.start(), start);
        assert_eq!(segment.end(), end);
        assert_eq!(segment.length(), 5.0); // 3-4-5直角三角形
    }

    #[test]
    fn test_line_segment3d_invalid_creation() {
        let point = Point3D::new(1.0, 2.0, 3.0);

        // 同じ点では線分作成不可
        assert!(LineSegment3D::new(point, point).is_none());

        // 長さ0の線分
        assert!(
            LineSegment3D::from_point_direction_length(point, Vector3D::unit_x(), 0.0).is_none()
        );

        // 負の長さ
        assert!(
            LineSegment3D::from_point_direction_length(point, Vector3D::unit_x(), -1.0).is_none()
        );
    }

    #[test]
    fn test_line_segment3d_from_direction_length() {
        let start = Point3D::new(1.0, 2.0, 3.0);
        let direction = Vector3D::unit_x();
        let length = 5.0;

        let segment = LineSegment3D::from_point_direction_length(start, direction, length).unwrap();
        assert_eq!(segment.start(), start);
        assert_eq!(segment.end(), Point3D::new(6.0, 2.0, 3.0));
        assert_eq!(segment.length(), length);
    }

    #[test]
    fn test_line_segment3d_midpoint() {
        let segment =
            LineSegment3D::new(Point3D::new(0.0_f64, 0.0, 0.0), Point3D::new(4.0, 6.0, 8.0))
                .unwrap();

        let midpoint = segment.midpoint();
        let expected = Point3D::new(2.0, 3.0, 4.0);

        // 浮動小数点誤差を考慮
        assert!((midpoint.x() - expected.x()).abs() < TOLERANCE_F64);
        assert!((midpoint.y() - expected.y()).abs() < TOLERANCE_F64);
        assert!((midpoint.z() - expected.z()).abs() < TOLERANCE_F64);
    }

    #[test]
    fn test_line_segment3d_direction_and_vector() {
        let segment =
            LineSegment3D::new(Point3D::new(0.0_f64, 0.0, 0.0), Point3D::new(3.0, 4.0, 0.0))
                .unwrap();

        let direction = segment.direction();
        let vector = Vector3D::from(LineSegment3DDerived::as_vector(&segment));

        // 方向ベクトルは正規化済み
        assert!((direction.length() - 1.0).abs() < TOLERANCE_F64);
        assert!((direction - Vector3D::new(0.6, 0.8, 0.0)).length() < TOLERANCE_F64);

        // ベクトルは始点から終点へ
        assert_eq!(vector, Vector3D::new(3.0, 4.0, 0.0));
    }

    #[test]
    fn test_line_segment3d_parametric_points() {
        let segment =
            LineSegment3D::new(Point3D::new(0.0, 0.0, 0.0), Point3D::new(10.0, 0.0, 0.0)).unwrap();

        // t=0で始点
        let p0 = segment.point_at_parameter(0.0);
        assert_eq!(p0, Point3D::new(0.0, 0.0, 0.0));

        // t=0.5で中点
        let p05 = segment.point_at_parameter(0.5);
        assert_eq!(p05, Point3D::new(5.0, 0.0, 0.0));

        // t=1で終点
        let p1 = segment.point_at_parameter(1.0);
        assert_eq!(p1, Point3D::new(10.0, 0.0, 0.0));
    }

    #[test]
    fn test_line_segment3d_point_projection() {
        let segment =
            LineSegment3D::new(Point3D::new(0.0, 0.0, 0.0), Point3D::new(10.0, 0.0, 0.0)).unwrap();

        // 線分内の点への投影
        let point_above = Point3D::new(5.0, 3.0, 0.0);
        let projected = segment.project_point(&point_above);
        assert_eq!(projected, Point3D::new(5.0, 0.0, 0.0));

        // 線分外の点（始点側）
        let point_before = Point3D::new(-5.0, 2.0, 0.0);
        let projected_start = segment.project_point(&point_before);
        assert_eq!(projected_start, segment.start());

        // 線分外の点（終点側）
        let point_after = Point3D::new(15.0, 2.0, 0.0);
        let projected_end = segment.project_point(&point_after);
        assert_eq!(projected_end, segment.end());
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
        assert!(segment.distance_to_point(&point_on_segment) < TOLERANCE_F64);

        // 線分に垂直な点
        let point_perpendicular = Point3D::new(5.0, 3.0, 0.0);
        assert!((segment.distance_to_point(&point_perpendicular) - 3.0).abs() < TOLERANCE_F64);

        // 線分外の点（端点への距離）
        let point_beyond = Point3D::new(15.0, 0.0, 0.0);
        assert!((segment.distance_to_point(&point_beyond) - 5.0).abs() < TOLERANCE_F64);
    }

    #[test]
    fn test_line_segment3d_contains_point() {
        let segment =
            LineSegment3D::new(Point3D::new(0.0, 0.0, 0.0), Point3D::new(10.0, 0.0, 0.0)).unwrap();

        assert!(segment.contains_point(&Point3D::new(5.0, 0.0, 0.0), TOLERANCE_F64));
        assert!(segment.contains_point(&segment.start(), TOLERANCE_F64));
        assert!(segment.contains_point(&segment.end(), TOLERANCE_F64));
        assert!(!segment.contains_point(&Point3D::new(15.0, 0.0, 0.0), TOLERANCE_F64));
        assert!(!segment.contains_point(&Point3D::new(5.0, 1.0, 0.0), TOLERANCE_F64));
    }

    #[test]
    fn test_line_segment3d_closest_parameter() {
        let segment = LineSegment3D::new(
            Point3D::new(0.0_f64, 0.0, 0.0),
            Point3D::new(10.0, 0.0, 0.0),
        )
        .unwrap();

        assert!(
            segment
                .closest_parameter(&Point3D::new(0.0, 0.0, 0.0))
                .abs()
                < TOLERANCE_F64
        );
        assert!(
            (segment.closest_parameter(&Point3D::new(5.0, 0.0, 0.0)) - 0.5).abs() < TOLERANCE_F64
        );
        assert!(
            (segment.closest_parameter(&Point3D::new(10.0, 0.0, 0.0)) - 1.0).abs() < TOLERANCE_F64
        );
        // 範囲外は [0, 1] に制限される
        assert!(
            (segment.closest_parameter(&Point3D::new(20.0, 0.0, 0.0)) - 1.0).abs() < TOLERANCE_F64
        );
    }

    #[test]
    fn test_line_segment3d_reversed_closest_parameter_and_projection() {
        // reverse した線分は support line 上で start_param > end_param となる
        let reversed = LineSegment3D::new(
            Point3D::new(0.0_f64, 0.0, 0.0),
            Point3D::new(10.0, 0.0, 0.0),
        )
        .unwrap()
        .reverse();

        // 始点 (10, 0, 0) からの比率になる
        let t = reversed.closest_parameter(&Point3D::new(3.0, 0.0, 0.0));
        assert!((t - 0.7).abs() < TOLERANCE_F64);
        assert!(
            reversed
                .point_at_parameter(t)
                .distance_to(&Point3D::new(3.0, 0.0, 0.0))
                < TOLERANCE_F64
        );
        assert!(
            reversed
                .closest_parameter(&Point3D::new(20.0, 0.0, 0.0))
                .abs()
                < TOLERANCE_F64
        );
        assert!(
            (reversed.closest_parameter(&Point3D::new(-5.0, 0.0, 0.0)) - 1.0).abs() < TOLERANCE_F64
        );

        // 投影は線分内の点はそのまま、範囲外は近い端点に制限される
        let projected = reversed.project_point(&Point3D::new(3.0, 2.0, 0.0));
        assert!(projected.distance_to(&Point3D::new(3.0, 0.0, 0.0)) < TOLERANCE_F64);
        let beyond_start = reversed.project_point(&Point3D::new(15.0, 1.0, 0.0));
        assert!(beyond_start.distance_to(&Point3D::new(10.0, 0.0, 0.0)) < TOLERANCE_F64);
        let beyond_end = reversed.project_point(&Point3D::new(-5.0, 1.0, 0.0));
        assert!(beyond_end.distance_to(&Point3D::new(0.0, 0.0, 0.0)) < TOLERANCE_F64);
    }

    #[test]
    fn test_line_segment3d_reverse() {
        let segment =
            LineSegment3D::new(Point3D::new(0.0, 0.0, 0.0), Point3D::new(10.0, 0.0, 0.0)).unwrap();

        let reversed = segment.reverse();
        assert_eq!(reversed.start(), Point3D::new(10.0, 0.0, 0.0));
        assert_eq!(reversed.end(), Point3D::new(0.0, 0.0, 0.0));
    }

    #[test]
    fn test_line_segment3d_split() {
        let segment = LineSegment3D::new(
            Point3D::new(0.0_f64, 0.0, 0.0),
            Point3D::new(10.0, 0.0, 0.0),
        )
        .unwrap();

        let first = segment.sub_segment(0.0, 0.3).unwrap();
        let second = segment.sub_segment(0.3, 1.0).unwrap();

        assert_eq!(first.start(), Point3D::new(0.0, 0.0, 0.0));
        assert_eq!(first.end(), Point3D::new(3.0, 0.0, 0.0));
        assert_eq!(second.start(), Point3D::new(3.0, 0.0, 0.0));
        assert_eq!(second.end(), Point3D::new(10.0, 0.0, 0.0));

        assert!((first.length() - 3.0).abs() < TOLERANCE_F64);
        assert!((second.length() - 7.0).abs() < TOLERANCE_F64);
    }

    #[test]
    fn test_geometry_foundation() {
        let segment =
            LineSegment3D::new(Point3D::new(1.0, 2.0, 3.0), Point3D::new(4.0, 6.0, 8.0)).unwrap();

        let bbox = segment.bounding_box();
        assert_eq!(bbox.min(), Point3D::new(1.0, 2.0, 3.0));
        assert_eq!(bbox.max(), Point3D::new(4.0, 6.0, 8.0));
    }

    #[test]
    fn test_basic_metrics() {
        let segment =
            LineSegment3D::new(Point3D::new(0.0, 0.0, 0.0), Point3D::new(3.0, 4.0, 0.0)).unwrap();

        assert_eq!(segment.length(), 5.0);
    }

    #[test]
    fn test_basic_containment() {
        let segment = LineSegment3D::new(
            Point3D::new(0.0_f64, 0.0, 0.0),
            Point3D::new(10.0, 0.0, 0.0),
        )
        .unwrap();

        assert!(segment.contains_point(&Point3D::new(5.0, 0.0, 0.0), TOLERANCE_F64));
        assert!(!segment.contains_point(&Point3D::new(15.0, 0.0, 0.0), TOLERANCE_F64));

        assert!(segment.on_boundary(&Point3D::new(5.0, 0.0, 0.0), TOLERANCE_F64));

        let distance = segment.distance_to_point(&Point3D::new(5.0, 3.0, 0.0));
        assert!((distance - 3.0).abs() < TOLERANCE_F64);
    }

    #[test]
    fn test_basic_parametric() {
        let segment =
            LineSegment3D::new(Point3D::new(0.0, 0.0, 0.0), Point3D::new(10.0, 0.0, 0.0)).unwrap();

        let (min_t, max_t) = segment.parameter_range();
        assert_eq!(min_t, 0.0);
        assert_eq!(max_t, 1.0);

        let point = segment.point_at_parameter(0.5);
        assert_eq!(point, Point3D::new(5.0, 0.0, 0.0));

        let tangent = segment.tangent_at_parameter(0.5);
        // 接線は正規化された方向ベクトル（2D の線分と同じ）
        assert!((tangent - Vector3D::new(1.0, 0.0, 0.0)).length() < TOLERANCE_F64);
    }

    #[test]
    fn test_basic_directional() {
        let segment =
            LineSegment3D::new(Point3D::new(0.0, 0.0, 0.0), Point3D::new(10.0, 0.0, 0.0)).unwrap();

        assert_eq!(segment.direction(), Vector3D::unit_x());

        let reversed = segment.reverse();
        assert_eq!(reversed.start(), Point3D::new(10.0, 0.0, 0.0));
        assert_eq!(reversed.end(), Point3D::new(0.0, 0.0, 0.0));
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
}
