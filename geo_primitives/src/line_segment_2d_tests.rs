//! LineSegment2D のテスト

use crate::{LineSegment2D, Point2D, Vector2D};
use geo_foundation::{core_foundation::*, Angle, Scalar};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_creation() {
        // 基本的な線分作成
        let start = Point2D::new(0.0, 0.0);
        let end = Point2D::new(3.0, 4.0);
        let segment = LineSegment2D::new(start, end).unwrap();

        assert_eq!(segment.start(), start);
        assert_eq!(segment.end(), end);
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

        assert_eq!(segment.start(), start);
        assert_eq!(segment.end(), Point2D::new(6.0, 2.0));
        assert_eq!(segment.length(), length);

        // 軸方向線分
        let x_segment = LineSegment2D::x_axis_segment(Point2D::new(2.0, 3.0), 4.0).unwrap();
        assert_eq!(x_segment.start(), Point2D::new(2.0, 3.0));
        assert_eq!(x_segment.end(), Point2D::new(6.0, 3.0));

        let y_segment = LineSegment2D::y_axis_segment(Point2D::new(1.0, 2.0), 3.0).unwrap();
        assert_eq!(y_segment.start(), Point2D::new(1.0, 2.0));
        assert_eq!(y_segment.end(), Point2D::new(1.0, 5.0));
    }

    #[test]
    fn test_special_segments() {
        // 水平線分
        let horizontal = LineSegment2D::horizontal_segment(2.0, 1.0, 5.0).unwrap();
        assert_eq!(horizontal.start(), Point2D::new(1.0, 2.0));
        assert_eq!(horizontal.end(), Point2D::new(5.0, 2.0));
        // 水平性をdirectionで確認
        let dir = horizontal.direction();
        assert!((dir.y().abs()) < f64::ANGLE_TOLERANCE);

        // 垂直線分
        let vertical = LineSegment2D::vertical_segment(3.0, 1.0, 4.0).unwrap();
        assert_eq!(vertical.start(), Point2D::new(3.0, 1.0));
        assert_eq!(vertical.end(), Point2D::new(3.0, 4.0));
        // 垂直性をdirectionで確認
        let dir = vertical.direction();
        assert!((dir.x().abs()) < f64::ANGLE_TOLERANCE);

        // 原点からの線分
        let from_origin = LineSegment2D::from_origin(Point2D::new(3.0, 4.0)).unwrap();
        assert_eq!(from_origin.start(), Point2D::origin());
        assert_eq!(from_origin.end(), Point2D::new(3.0, 4.0));
    }

    #[test]
    fn test_midpoint_and_direction() {
        let segment = LineSegment2D::new(Point2D::new(1.0, 2.0), Point2D::new(5.0, 6.0)).unwrap();

        // 中点
        let midpoint = segment.midpoint();
        assert_eq!(midpoint, Point2D::new(3.0, 4.0));

        // 方向ベクトル
        let direction = segment.direction();
        let expected_direction = Vector2D::new(4.0, 4.0).normalize();
        assert!((direction.x() - expected_direction.x()).abs() < f64::EPSILON);
        assert!((direction.y() - expected_direction.y()).abs() < f64::EPSILON);

        // ベクトル表現
        let vector = segment.vector();
        assert_eq!(vector, Vector2D::new(4.0, 4.0));
    }

    #[test]
    fn test_parametric_operations() {
        let segment = LineSegment2D::new(Point2D::new(0.0, 0.0), Point2D::new(4.0, 0.0)).unwrap();

        // パラメータでの点取得
        assert_eq!(
            segment.point_at_normalized_parameter(0.0),
            Point2D::new(0.0, 0.0)
        );
        assert_eq!(
            segment.point_at_normalized_parameter(0.5),
            Point2D::new(2.0, 0.0)
        );
        assert_eq!(
            segment.point_at_normalized_parameter(1.0),
            Point2D::new(4.0, 0.0)
        );

        // 範囲外パラメータ（クランプされる）
        assert_eq!(
            segment.point_at_normalized_parameter(-0.5),
            Point2D::new(0.0, 0.0)
        );
        assert_eq!(
            segment.point_at_normalized_parameter(1.5),
            Point2D::new(4.0, 0.0)
        );

        // 点からパラメータ取得
        assert!((segment.parameter_for_point(&Point2D::new(1.0, 0.0)) - 0.25).abs() < f64::EPSILON);
        assert!((segment.parameter_for_point(&Point2D::new(3.0, 0.0)) - 0.75).abs() < f64::EPSILON);
    }

    #[test]
    fn test_point_operations() {
        let segment = LineSegment2D::new(Point2D::new(0.0, 0.0), Point2D::new(4.0, 0.0)).unwrap();

        // 点の投影
        let above_point = Point2D::new(2.0, 3.0);
        let projected = segment.project_point_to_segment(&above_point);
        assert_eq!(projected, Point2D::new(2.0, 0.0));

        // 線分外への投影（クランプされる）
        let outside_point = Point2D::new(-1.0, 2.0);
        let projected_outside = segment.project_point_to_segment(&outside_point);
        assert_eq!(projected_outside, Point2D::new(0.0, 0.0));

        // 距離計算
        assert!((segment.distance_to_point(&above_point) - 3.0).abs() < f64::EPSILON);
        assert!((segment.distance_to_point(&Point2D::new(2.0, 0.0)) - 0.0).abs() < f64::EPSILON);

        // 点の包含判定
        assert!(segment.contains_point(&Point2D::new(2.0, 0.0), f64::TOLERANCE));
        assert!(!segment.contains_point(&Point2D::new(2.0, 1.0), f64::TOLERANCE));
    }

    #[test]
    fn test_transformations() {
        let segment = LineSegment2D::new(Point2D::new(1.0, 1.0), Point2D::new(3.0, 1.0)).unwrap();

        // 平行移動
        let vector = Vector2D::new(2.0, 3.0);
        let translated = segment.translate(&vector);
        assert_eq!(translated.start(), Point2D::new(3.0, 4.0));
        assert_eq!(translated.end(), Point2D::new(5.0, 4.0));

        // スケーリング（原点基準）
        let scaled = segment.scale(2.0).unwrap();
        assert_eq!(scaled.start(), Point2D::new(2.0, 2.0)); // 原点から2倍
        assert_eq!(scaled.end(), Point2D::new(6.0, 2.0)); // 原点から2倍

        // 長さスケーリング（始点基準）
        let length_scaled = segment.scale_length(2.0).unwrap();
        assert_eq!(length_scaled.start(), Point2D::new(1.0, 1.0)); // 始点固定
        assert_eq!(length_scaled.end(), Point2D::new(5.0, 1.0)); // 長さ2倍

        // 方向反転
        let reversed = segment.reverse();
        assert_eq!(reversed.start(), Point2D::new(3.0, 1.0));
        assert_eq!(reversed.end(), Point2D::new(1.0, 1.0));
    }

    #[test]
    fn test_line_relationships() {
        let segment1 = LineSegment2D::new(Point2D::new(0.0, 0.0), Point2D::new(2.0, 0.0)).unwrap();

        let segment2 = LineSegment2D::new(Point2D::new(0.0, 1.0), Point2D::new(2.0, 1.0)).unwrap();

        let segment3 = LineSegment2D::new(Point2D::new(1.0, -1.0), Point2D::new(1.0, 1.0)).unwrap();

        // 平行判定
        assert!(segment1.is_parallel_to(&segment2, f64::ANGLE_TOLERANCE));
        assert!(!segment1.is_parallel_to(&segment3, f64::ANGLE_TOLERANCE));

        // 垂直判定
        assert!(segment1.is_perpendicular_to(&segment3, f64::ANGLE_TOLERANCE));
        assert!(!segment1.is_perpendicular_to(&segment2, f64::ANGLE_TOLERANCE));

        // 共線判定
        let collinear_segment =
            LineSegment2D::new(Point2D::new(3.0, 0.0), Point2D::new(5.0, 0.0)).unwrap();
        assert!(segment1.is_collinear_with(&collinear_segment, f64::EPSILON));
        assert!(!segment1.is_collinear_with(&segment2, f64::EPSILON));
    }

    #[test]
    fn test_intersection() {
        let segment1 = LineSegment2D::new(Point2D::new(0.0, 0.0), Point2D::new(4.0, 0.0)).unwrap();

        let segment2 = LineSegment2D::new(Point2D::new(2.0, -1.0), Point2D::new(2.0, 1.0)).unwrap();

        // 交差する場合
        let intersection = segment1.intersection_with_segment(&segment2);
        assert!(intersection.is_some());
        let point = intersection.unwrap();
        assert!((point.x() - 2.0).abs() < f64::EPSILON);
        assert!((point.y() - 0.0).abs() < f64::EPSILON);

        // 交差しない場合
        let segment3 = LineSegment2D::new(Point2D::new(5.0, -1.0), Point2D::new(5.0, 1.0)).unwrap();

        let no_intersection = segment1.intersection_with_segment(&segment3);
        assert!(no_intersection.is_none());
    }

    #[test]
    fn test_split_operations() {
        let segment = LineSegment2D::new(Point2D::new(0.0, 0.0), Point2D::new(4.0, 0.0)).unwrap();

        // 分割
        let (first, second) = segment.split_at(0.25);
        assert_eq!(first.start(), Point2D::new(0.0, 0.0));
        assert_eq!(first.end(), Point2D::new(1.0, 0.0));
        assert_eq!(second.start(), Point2D::new(1.0, 0.0));
        assert_eq!(second.end(), Point2D::new(4.0, 0.0));

        // 延長
        let extended = segment.extend_to_length(8.0).unwrap();
        assert_eq!(extended.start(), Point2D::new(0.0, 0.0));
        assert_eq!(extended.end(), Point2D::new(8.0, 0.0));

        // 両端延長
        let both_extended = segment.extend_both_ends(1.0, 2.0);
        assert_eq!(both_extended.start(), Point2D::new(-1.0, 0.0));
        assert_eq!(both_extended.end(), Point2D::new(6.0, 0.0));
    }

    #[test]
    fn test_geometric_properties() {
        let segment = LineSegment2D::new(Point2D::new(1.0, 2.0), Point2D::new(4.0, 6.0)).unwrap();

        // 面積（線分は常に0）
        assert_eq!(segment.area(), 0.0);

        // 重心（中点）
        let centroid = segment.centroid();
        assert_eq!(centroid, segment.midpoint());

        // 勾配
        let slope = segment.slope().unwrap();
        assert!((slope - (4.0 / 3.0)).abs() < 1e-10);

        // 退化判定
        assert!(!segment.is_degenerate(f64::EPSILON));

        let tiny_segment =
            LineSegment2D::new(Point2D::new(0.0, 0.0), Point2D::new(1e-10, 0.0)).unwrap();
        assert!(tiny_segment.is_degenerate(1e-8));
    }

    #[test]
    fn test_rotation() {
        let segment = LineSegment2D::new(Point2D::new(1.0, 0.0), Point2D::new(2.0, 0.0)).unwrap();

        // 90度回転
        let rotated = segment.rotate_around_origin(Angle::from_radians(std::f64::consts::PI / 2.0));
        let rotated_start = rotated.start();
        let rotated_end = rotated.end();

        // 回転後の点をチェック（数値誤差を考慮）
        assert!((rotated_start.x() - 0.0).abs() < f64::EPSILON);
        assert!((rotated_start.y() - 1.0).abs() < f64::EPSILON);
        assert!((rotated_end.x() - 0.0).abs() < f64::EPSILON);
        assert!((rotated_end.y() - 2.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_distance_between_segments() {
        let segment1 = LineSegment2D::new(Point2D::new(0.0, 0.0), Point2D::new(2.0, 0.0)).unwrap();

        let segment2 = LineSegment2D::new(Point2D::new(0.0, 3.0), Point2D::new(2.0, 3.0)).unwrap();

        let distance = segment1.distance_to_segment(&segment2);
        assert!((distance - 3.0).abs() < f64::EPSILON);
    }

    // === Foundation トレイトのテスト ===

    #[test]
    fn test_geometry_foundation_traits() {
        let segment = LineSegment2D::new(Point2D::new(1.0, 2.0), Point2D::new(5.0, 6.0)).unwrap();

        // CoreFoundation
        let bbox = segment.bounding_box();
        assert_eq!(bbox.min(), Point2D::new(1.0, 2.0));
        assert_eq!(bbox.max(), Point2D::new(5.0, 6.0));
    }

    #[test]
    fn test_basic_metrics_trait() {
        let segment = LineSegment2D::new(Point2D::new(0.0, 0.0), Point2D::new(3.0, 4.0)).unwrap();

        // BasicMetrics
        let length = segment.length();
        assert!((length - 5.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_basic_containment_trait() {
        let segment = LineSegment2D::new(Point2D::new(0.0, 0.0), Point2D::new(4.0, 0.0)).unwrap();

        // BasicContainment
        assert!(segment.contains_point(&Point2D::new(2.0, 0.0), f64::EPSILON));
        assert!(!segment.contains_point(&Point2D::new(2.0, 1.0), f64::EPSILON));

        assert!(segment.on_boundary(&Point2D::new(2.0, 0.0), f64::EPSILON));

        let distance = segment.distance_to_point(&Point2D::new(2.0, 3.0));
        assert!((distance - 3.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_basic_parametric_trait() {
        let segment = LineSegment2D::new(Point2D::new(0.0, 0.0), Point2D::new(4.0, 0.0)).unwrap();

        // BasicParametric
        let (start_param, end_param) = segment.parameter_range();
        assert_eq!(start_param, 0.0);
        assert_eq!(end_param, 1.0);

        let mid_point = segment.point_at_parameter(0.5);
        assert_eq!(mid_point, Point2D::new(2.0, 0.0));

        let tangent = segment.tangent_at_parameter(0.5);
        assert_eq!(tangent, Vector2D::new(1.0, 0.0)); // 正規化された接線方向
    }

    #[test]
    fn test_basic_directional_trait() {
        let segment = LineSegment2D::new(Point2D::new(0.0, 0.0), Point2D::new(3.0, 4.0)).unwrap();

        // BasicDirectional
        let direction = segment.direction();
        let expected = Vector2D::new(3.0, 4.0).normalize();
        assert!((direction.x() - expected.x()).abs() < f64::EPSILON);
        assert!((direction.y() - expected.y()).abs() < f64::EPSILON);

        let reversed = segment.reverse_direction();
        assert_eq!(reversed.start(), Point2D::new(3.0, 4.0));
        assert_eq!(reversed.end(), Point2D::new(0.0, 0.0));
    }

    #[test]
    fn test_f32_compatibility() {
        // f32での基本操作
        let segment =
            LineSegment2D::new(Point2D::new(0.0f32, 0.0f32), Point2D::new(3.0f32, 4.0f32)).unwrap();

        assert!((segment.length() - 5.0f32).abs() < f32::EPSILON);

        let midpoint = segment.midpoint();
        assert!((midpoint.x() - 1.5f32).abs() < f32::EPSILON);
        assert!((midpoint.y() - 2.0f32).abs() < f32::EPSILON);

        // foundation トレイト
        let bbox = segment.bounding_box();
        assert_eq!(bbox.min(), Point2D::new(0.0f32, 0.0f32));
        assert_eq!(bbox.max(), Point2D::new(3.0f32, 4.0f32));
    }
}
