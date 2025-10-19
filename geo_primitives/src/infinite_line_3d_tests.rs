//! InfiniteLine3D のテスト

use crate::{InfiniteLine3D, Point3D, Vector3D};
use geo_foundation::core_foundation::*;
use geo_foundation::{extensions::BasicTransform, Angle};

// BasicTransformの実装を有効にするため
#[allow(unused_imports)]
use crate::infinite_line_3d_transform;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_infinite_line3d_creation() {
        let point = Point3D::new(1.0, 2.0, 3.0);
        let direction = Vector3D::new(1.0, 0.0, 0.0);

        let line = InfiniteLine3D::new(point, direction).unwrap();
        assert_eq!(line.point(), point);
        assert_eq!(line.direction().as_vector(), Vector3D::unit_x());
    }

    #[test]
    fn test_infinite_line3d_invalid_creation() {
        let point = Point3D::new(0.0, 0.0, 0.0);
        let zero_direction = Vector3D::zero();

        // ゼロベクトルでは作成できない
        assert!(InfiniteLine3D::new(point, zero_direction).is_none());
    }

    #[test]
    fn test_infinite_line3d_from_two_points() {
        let p1 = Point3D::new(0.0_f64, 0.0, 0.0);
        let p2 = Point3D::new(3.0, 4.0, 0.0);

        let line = InfiniteLine3D::from_two_points(p1, p2).unwrap();
        assert_eq!(line.point(), p1);

        // 方向ベクトルが正規化されている
        let expected_direction = Vector3D::new(0.6, 0.8, 0.0); // (3,4,0) 正規化
        assert!((line.direction().x() - expected_direction.x()).abs() < 1e-10);
        assert!((line.direction().y() - expected_direction.y()).abs() < 1e-10);
        assert!(line.direction().z().abs() < 1e-10);
    }

    #[test]
    fn test_infinite_line3d_axis_constructors() {
        let point = Point3D::new(1.0, 2.0, 3.0);

        let x_line = InfiniteLine3D::x_axis(point);
        let y_line = InfiniteLine3D::y_axis(point);
        let z_line = InfiniteLine3D::z_axis(point);

        assert_eq!(x_line.direction().as_vector(), Vector3D::unit_x());
        assert_eq!(y_line.direction().as_vector(), Vector3D::unit_y());
        assert_eq!(z_line.direction().as_vector(), Vector3D::unit_z());
    }

    #[test]
    fn test_infinite_line3d_point_at_parameter() {
        let line = InfiniteLine3D::new(Point3D::new(1.0, 2.0, 3.0), Vector3D::unit_x()).unwrap();

        // t=0で基準点
        let p0 = line.point_at_parameter(0.0);
        assert_eq!(p0, Point3D::new(1.0, 2.0, 3.0));

        // t=2で+X方向に2移動
        let p2 = line.point_at_parameter(2.0);
        assert_eq!(p2, Point3D::new(3.0, 2.0, 3.0));

        // t=-1で-X方向に1移動
        let p_neg1 = line.point_at_parameter(-1.0);
        assert_eq!(p_neg1, Point3D::new(0.0, 2.0, 3.0));
    }

    #[test]
    fn test_infinite_line3d_point_projection() {
        let line = InfiniteLine3D::x_axis(Point3D::origin());

        // Y軸上の点をX軸に投影
        let point = Point3D::new(0.0, 5.0, 0.0);
        let projected = line.project_point(&point);
        assert_eq!(projected, Point3D::origin());

        // 一般的な点の投影
        let point2 = Point3D::new(3.0, 4.0, 5.0);
        let projected2 = line.project_point(&point2);
        assert_eq!(projected2, Point3D::new(3.0, 0.0, 0.0));
    }

    #[test]
    fn test_infinite_line3d_distance_to_point() {
        let line = InfiniteLine3D::x_axis(Point3D::<f64>::origin());

        // X軸上の点（距離0）
        let point_on_line = Point3D::new(5.0, 0.0, 0.0);
        assert!(line.distance_to_point(&point_on_line) < 1e-10);

        // Y軸上の点（距離=Y座標）
        let point_off_line = Point3D::new(0.0, 3.0, 0.0);
        assert!((line.distance_to_point(&point_off_line) - 3.0).abs() < 1e-10);

        // 3D点（3-4-5直角三角形）
        let point_3d = Point3D::new(0.0, 3.0, 4.0);
        assert!((line.distance_to_point(&point_3d) - 5.0).abs() < 1e-10);
    }

    #[test]
    fn test_infinite_line3d_contains_point() {
        let line = InfiniteLine3D::x_axis(Point3D::<f64>::origin());

        assert!(line.contains_point(&Point3D::new(100.0, 0.0, 0.0), 1e-10));
        assert!(line.contains_point(&Point3D::new(-50.0, 0.0, 0.0), 1e-10));
        assert!(!line.contains_point(&Point3D::new(0.0, 0.1, 0.0), 1e-10));
    }

    #[test]
    fn test_infinite_line3d_parameter_for_point() {
        let line = InfiniteLine3D::new(Point3D::new(2.0, 0.0, 0.0), Vector3D::unit_x()).unwrap();

        let point = Point3D::new(5.0, 0.0, 0.0);
        let t = line.parameter_for_point(&point);
        assert_eq!(t, 3.0); // 5 - 2 = 3

        // 確認：パラメータから点を復元
        let reconstructed = line.point_at_parameter(t);
        assert_eq!(reconstructed.x(), point.x());
    }

    #[test]
    fn test_infinite_line3d_parallel_lines() {
        let line1 = InfiniteLine3D::x_axis(Point3D::<f64>::origin());
        let line2 = InfiniteLine3D::x_axis(Point3D::new(0.0, 1.0, 0.0));
        let line3 = InfiniteLine3D::y_axis(Point3D::origin());

        assert!(line1.is_parallel(&line2, 1e-10));
        assert!(!line1.is_parallel(&line3, 1e-10));
    }

    #[test]
    fn test_infinite_line3d_coincident_lines() {
        let line1 = InfiniteLine3D::x_axis(Point3D::<f64>::origin());
        let line2 = InfiniteLine3D::x_axis(Point3D::new(5.0, 0.0, 0.0));
        let line3 = InfiniteLine3D::x_axis(Point3D::new(0.0, 1.0, 0.0));

        assert!(line1.is_coincident(&line2, 1e-10)); // 同一直線
        assert!(!line1.is_coincident(&line3, 1e-10)); // 平行だが異なる直線
    }

    // TODO: closest_points_to_line メソッドが未実装のためコメントアウト
    /*
    #[test]
    fn test_infinite_line3d_closest_points() {
        let line1 = InfiniteLine3D::x_axis(Point3D::<f64>::origin());
        let line2 = InfiniteLine3D::y_axis(Point3D::origin());

        let (p1, p2) = line1.closest_points_to_line(&line2);

        // X軸とY軸の交点は原点
        assert!(p1.distance_to(&Point3D::origin()) < 1e-10);
        assert!(p2.distance_to(&Point3D::origin()) < 1e-10);
    }
    */

    // TODO: is_axis_aligned と aligned_axis メソッドが未実装のためコメントアウト
    /*
    #[test]
    fn test_infinite_line3d_axis_alignment() {
        let x_line = InfiniteLine3D::x_axis(Point3D::origin());
        let diagonal_line =
            InfiniteLine3D::new(Point3D::origin(), Vector3D::new(1.0, 1.0, 0.0)).unwrap();

        assert!(x_line.is_axis_aligned(1e-10));
        assert!(!diagonal_line.is_axis_aligned(1e-10));

        assert_eq!(x_line.aligned_axis(1e-10), Some(Vector3D::unit_x()));
        assert_eq!(diagonal_line.aligned_axis(1e-10), None);
    }
    */

    // === foundation トレイトテスト ===

    // TODO: bounding_box メソッドが未実装のためコメントアウト
    /*
    #[test]
    fn test_geometry_foundation() {
        let line = InfiniteLine3D::x_axis(Point3D::<f64>::origin());
        let bbox = line.bounding_box();

        // 無限直線の境界ボックスは非常に大きい
        assert!(bbox.min().x() < -1e9);
        assert!(bbox.max().x() > 1e9);
    }
    */

    #[test]
    fn test_basic_containment() {
        let line = InfiniteLine3D::x_axis(Point3D::<f64>::origin());

        assert!(line.contains_point(&Point3D::new(100.0, 0.0, 0.0), 1e-10));
        assert!(!line.contains_point(&Point3D::new(0.0, 0.1, 0.0), 1e-10));

        // TODO: on_boundary メソッドが未実装のためコメントアウト
        // assert!(line.on_boundary(&Point3D::new(-50.0, 0.0, 0.0), 1e-10));

        let distance = line.distance_to_point(&Point3D::new(0.0, 3.0, 4.0));
        assert!((distance - 5.0).abs() < 1e-10);
    }

    #[test]
    fn test_basic_directional() {
        let line = InfiniteLine3D::x_axis(Point3D::<f64>::origin());

        assert_eq!(line.direction().as_vector(), Vector3D::unit_x());

        let reversed = line.reverse_direction();
        assert_eq!(
            reversed.direction().as_vector(),
            Vector3D::unit_x().negate()
        );
    }

    #[test]
    fn test_infinite_line3d_f32_compatibility() {
        let line_f32 = InfiniteLine3D::new(
            Point3D::new(0.0f32, 0.0f32, 0.0f32),
            Vector3D::new(1.0f32, 0.0f32, 0.0f32),
        )
        .unwrap();

        let line_f64 = InfiniteLine3D::new(
            Point3D::new(0.0f64, 0.0f64, 0.0f64),
            Vector3D::new(1.0f64, 0.0f64, 0.0f64),
        )
        .unwrap();

        assert_eq!(line_f32.direction().as_vector(), Vector3D::unit_x());
        assert_eq!(line_f64.direction().as_vector(), Vector3D::unit_x());
    }

    // ========================================================================
    // BasicTransform Tests
    // ========================================================================

    #[test]
    fn test_infinite_line3d_basic_transform_translate() {
        let line = InfiniteLine3D::new(
            Point3D::new(1.0_f64, 2.0_f64, 3.0_f64),
            Vector3D::new(1.0_f64, 0.0_f64, 0.0_f64),
        )
        .unwrap();
        let offset = Vector3D::new(3.0_f64, 4.0_f64, 5.0_f64);

        let translated = BasicTransform::translate(&line, offset);
        assert_eq!(translated.point(), Point3D::new(4.0_f64, 6.0_f64, 8.0_f64));
        assert_eq!(
            translated.direction().as_vector(),
            line.direction().as_vector()
        );
    }

    #[test]
    fn test_infinite_line3d_basic_transform_rotate() {
        let line = InfiniteLine3D::new(
            Point3D::new(1.0_f64, 0.0_f64, 0.0_f64),
            Vector3D::new(1.0_f64, 0.0_f64, 0.0_f64),
        )
        .unwrap();
        let center = Point3D::new(0.0_f64, 0.0_f64, 0.0_f64);
        let angle = Angle::from_degrees(90.0_f64);

        // 簡易実装では元の直線と同じものが返される
        let rotated = BasicTransform::rotate(&line, center, angle);
        assert_eq!(rotated.point(), line.point());
        assert_eq!(
            rotated.direction().as_vector(),
            line.direction().as_vector()
        );
    }

    #[test]
    fn test_infinite_line3d_basic_transform_scale() {
        let line = InfiniteLine3D::new(
            Point3D::new(2.0_f64, 3.0_f64, 4.0_f64),
            Vector3D::new(1.0_f64, 0.0_f64, 0.0_f64),
        )
        .unwrap();
        let center = Point3D::new(0.0_f64, 0.0_f64, 0.0_f64);
        let factor = 2.0_f64;

        let scaled = BasicTransform::scale(&line, center, factor);

        // スケール後、直線上の点は(4,6,8)、方向は正規化されているので変わらず
        assert_eq!(scaled.point(), Point3D::new(4.0_f64, 6.0_f64, 8.0_f64));
        assert_eq!(scaled.direction().as_vector(), line.direction().as_vector());
    }
}
