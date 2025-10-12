//! LineSegment3D のテスト

use crate::{LineSegment3D, Point3D, Vector3D};
use geo_foundation::abstract_types::geometry::core_foundation::*;

#[cfg(test)]
mod tests {
    use super::*;

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
    fn test_line_segment3d_axis_constructors() {
        let start = Point3D::new(1.0, 2.0, 3.0);
        let length = 4.0;

        let x_segment = LineSegment3D::x_axis_segment(start, length).unwrap();
        let y_segment = LineSegment3D::y_axis_segment(start, length).unwrap();
        let z_segment = LineSegment3D::z_axis_segment(start, length).unwrap();

        assert_eq!(x_segment.end(), Point3D::new(5.0, 2.0, 3.0));
        assert_eq!(y_segment.end(), Point3D::new(1.0, 6.0, 3.0));
        assert_eq!(z_segment.end(), Point3D::new(1.0, 2.0, 7.0));
    }

    #[test]
    fn test_line_segment3d_midpoint() {
        let segment =
            LineSegment3D::new(Point3D::new(0.0_f64, 0.0, 0.0), Point3D::new(4.0, 6.0, 8.0))
                .unwrap();

        let midpoint = segment.midpoint();
        let expected = Point3D::new(2.0, 3.0, 4.0);

        // 浮動小数点誤差を考慮
        assert!((midpoint.x() - expected.x()).abs() < 1e-10);
        assert!((midpoint.y() - expected.y()).abs() < 1e-10);
        assert!((midpoint.z() - expected.z()).abs() < 1e-10);
    }

    #[test]
    fn test_line_segment3d_direction_and_vector() {
        let segment =
            LineSegment3D::new(Point3D::new(0.0_f64, 0.0, 0.0), Point3D::new(3.0, 4.0, 0.0))
                .unwrap();

        let direction = segment.direction();
        let vector = segment.vector();

        // 方向ベクトルは正規化済み
        assert!((direction.length() - 1.0).abs() < 1e-10);
        assert_eq!(direction, Vector3D::new(0.6, 0.8, 0.0));

        // ベクトルは始点から終点へ
        assert_eq!(vector, Vector3D::new(3.0, 4.0, 0.0));
    }

    #[test]
    fn test_line_segment3d_parametric_points() {
        let segment =
            LineSegment3D::new(Point3D::new(0.0, 0.0, 0.0), Point3D::new(10.0, 0.0, 0.0)).unwrap();

        // t=0で始点
        let p0 = segment.point_at_normalized_parameter(0.0);
        assert_eq!(p0, Point3D::new(0.0, 0.0, 0.0));

        // t=0.5で中点
        let p05 = segment.point_at_normalized_parameter(0.5);
        assert_eq!(p05, Point3D::new(5.0, 0.0, 0.0));

        // t=1で終点
        let p1 = segment.point_at_normalized_parameter(1.0);
        assert_eq!(p1, Point3D::new(10.0, 0.0, 0.0));

        // 範囲外パラメータ（制限される）
        let p_neg = segment.point_at_normalized_parameter(-0.5);
        assert_eq!(p_neg, segment.start());

        let p_over = segment.point_at_normalized_parameter(1.5);
        assert_eq!(p_over, segment.end());
    }

    #[test]
    fn test_line_segment3d_point_projection() {
        let segment =
            LineSegment3D::new(Point3D::new(0.0, 0.0, 0.0), Point3D::new(10.0, 0.0, 0.0)).unwrap();

        // 線分内の点への投影
        let point_above = Point3D::new(5.0, 3.0, 0.0);
        let projected = segment.project_point_to_segment(&point_above);
        assert_eq!(projected, Point3D::new(5.0, 0.0, 0.0));

        // 線分外の点（始点側）
        let point_before = Point3D::new(-5.0, 2.0, 0.0);
        let projected_start = segment.project_point_to_segment(&point_before);
        assert_eq!(projected_start, segment.start());

        // 線分外の点（終点側）
        let point_after = Point3D::new(15.0, 2.0, 0.0);
        let projected_end = segment.project_point_to_segment(&point_after);
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
        assert!(segment.distance_to_point(&point_on_segment) < 1e-10);

        // 線分に垂直な点
        let point_perpendicular = Point3D::new(5.0, 3.0, 0.0);
        assert!((segment.distance_to_point(&point_perpendicular) - 3.0).abs() < 1e-10);

        // 線分外の点（端点への距離）
        let point_beyond = Point3D::new(15.0, 0.0, 0.0);
        assert!((segment.distance_to_point(&point_beyond) - 5.0).abs() < 1e-10);
    }

    #[test]
    fn test_line_segment3d_contains_point() {
        let segment =
            LineSegment3D::new(Point3D::new(0.0, 0.0, 0.0), Point3D::new(10.0, 0.0, 0.0)).unwrap();

        assert!(segment.contains_point(&Point3D::new(5.0, 0.0, 0.0), 1e-10));
        assert!(segment.contains_point(&segment.start(), 1e-10));
        assert!(segment.contains_point(&segment.end(), 1e-10));
        assert!(!segment.contains_point(&Point3D::new(15.0, 0.0, 0.0), 1e-10));
        assert!(!segment.contains_point(&Point3D::new(5.0, 1.0, 0.0), 1e-10));
    }

    #[test]
    fn test_line_segment3d_parameter_for_point() {
        let segment =
            LineSegment3D::new(Point3D::new(0.0, 0.0, 0.0), Point3D::new(10.0, 0.0, 0.0)).unwrap();

        assert_eq!(
            segment.parameter_for_point(&Point3D::new(0.0, 0.0, 0.0)),
            0.0
        );
        assert_eq!(
            segment.parameter_for_point(&Point3D::new(5.0, 0.0, 0.0)),
            0.5
        );
        assert_eq!(
            segment.parameter_for_point(&Point3D::new(10.0, 0.0, 0.0)),
            1.0
        );
        assert_eq!(
            segment.parameter_for_point(&Point3D::new(20.0, 0.0, 0.0)),
            2.0
        );
    }

    #[test]
    fn test_line_segment3d_transformations() {
        let segment =
            LineSegment3D::new(Point3D::new(0.0, 0.0, 0.0), Point3D::new(10.0, 0.0, 0.0)).unwrap();

        // 平行移動
        let translated = segment.translate(&Vector3D::new(5.0, 3.0, 2.0));
        assert_eq!(translated.start(), Point3D::new(5.0, 3.0, 2.0));
        assert_eq!(translated.end(), Point3D::new(15.0, 3.0, 2.0));

        // 拡大縮小
        let scaled = segment.scale(2.0).unwrap();
        assert_eq!(scaled.start(), Point3D::new(0.0, 0.0, 0.0));
        assert_eq!(scaled.end(), Point3D::new(20.0, 0.0, 0.0));
        assert_eq!(scaled.length(), 20.0);

        // 方向反転
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

        let (first, second) = segment.split_at(0.3);

        assert_eq!(first.start(), Point3D::new(0.0, 0.0, 0.0));
        assert_eq!(first.end(), Point3D::new(3.0, 0.0, 0.0));
        assert_eq!(second.start(), Point3D::new(3.0, 0.0, 0.0));
        assert_eq!(second.end(), Point3D::new(10.0, 0.0, 0.0));

        assert!((first.length() - 3.0).abs() < 1e-10);
        assert!((second.length() - 7.0).abs() < 1e-10);
    }

    // === foundation トレイトテスト ===

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

        assert_eq!(BasicMetrics::length(&segment), Some(5.0));
    }

    #[test]
    fn test_basic_containment() {
        let segment = LineSegment3D::new(
            Point3D::new(0.0_f64, 0.0, 0.0),
            Point3D::new(10.0, 0.0, 0.0),
        )
        .unwrap();

        assert!(segment.contains_point(&Point3D::new(5.0, 0.0, 0.0), 1e-10));
        assert!(!segment.contains_point(&Point3D::new(15.0, 0.0, 0.0), 1e-10));

        assert!(segment.on_boundary(&Point3D::new(5.0, 0.0, 0.0), 1e-10));

        let distance = segment.distance_to_point(&Point3D::new(5.0, 3.0, 0.0));
        assert!((distance - 3.0).abs() < 1e-10);
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
        // 接線ベクトルは方向ベクトル×長さ
        assert_eq!(tangent, Vector3D::new(10.0, 0.0, 0.0));
    }

    #[test]
    fn test_basic_directional() {
        let segment =
            LineSegment3D::new(Point3D::new(0.0, 0.0, 0.0), Point3D::new(10.0, 0.0, 0.0)).unwrap();

        assert_eq!(segment.direction(), Vector3D::unit_x());

        let reversed = segment.reverse_direction();
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
