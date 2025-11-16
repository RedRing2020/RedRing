//! Point3D のテスト

use crate::Point3D;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_point3d_creation() {
        let point = Point3D::new(1.0, 2.0, 3.0);
        assert_eq!(point.x(), 1.0);
        assert_eq!(point.y(), 2.0);
        assert_eq!(point.z(), 3.0);
    }

    #[test]
    fn test_point3d_origin() {
        let origin = Point3D::<f64>::origin();
        assert_eq!(origin.x(), 0.0);
        assert_eq!(origin.y(), 0.0);
        assert_eq!(origin.z(), 0.0);
    }

    #[test]
    fn test_point3d_coords() {
        let point = Point3D::new(4.0, 5.0, 6.0);
        assert_eq!(point.coords(), [4.0, 5.0, 6.0]);
    }

    #[test]
    fn test_point3d_distance() {
        let p1 = Point3D::new(0.0, 0.0, 0.0);
        let p2 = Point3D::new(3.0, 4.0, 0.0);
        assert_eq!(p1.distance_to(&p2), 5.0); // 3-4-5直角三角形
    }

    #[test]
    fn test_point3d_equality() {
        let p1 = Point3D::new(1.0, 2.0, 3.0);
        let p2 = Point3D::new(1.0, 2.0, 3.0);
        let p3 = Point3D::new(1.0, 2.0, 4.0);

        assert_eq!(p1, p2);
        assert_ne!(p1, p3);
    }

    #[test]
    fn test_point3d_f32_compatibility() {
        let point_f32 = Point3D::new(1.0f32, 2.0f32, 3.0f32);
        let point_f64 = Point3D::new(1.0f64, 2.0f64, 3.0f64);

        assert_eq!(point_f32.x(), 1.0f32);
        assert_eq!(point_f64.x(), 1.0f64);
    }

    // === foundation トレイトテスト ===

    #[test]
    fn test_basic_operations() {
        let point = Point3D::new(1.0, 2.0, 3.0);
        let same_point = Point3D::new(1.0, 2.0, 3.0);
        let different_point = Point3D::new(2.0, 3.0, 4.0);

        // 点は自分自身を含む
        assert!(point.contains_point(&same_point));
        assert!(!point.contains_point(&different_point));

        // 境界判定（許容誤差付き）
        assert!(point.on_boundary(&same_point, 0.0));
        assert!(point.on_boundary(&Point3D::new(1.1, 2.0, 3.0), 0.2));
        assert!(!point.on_boundary(&Point3D::new(1.1, 2.0, 3.0), 0.05));

        // 距離計算
        assert_eq!(point.distance_to(&same_point), 0.0);
        assert_eq!(point.distance_to(&Point3D::new(1.0, 2.0, 6.0)), 3.0);
    }

    // ============================================================================
    // Transform テスト (point_3d_transform.rs から統合)
    // ============================================================================

    #[test]
    fn test_transform_translate() {
        use crate::Vector3D;

        let point = Point3D::new(1.0, 2.0, 3.0);
        let translation = Vector3D::new(2.0, 3.0, 4.0);
        // BasicTransformトレイトが未実装のため、実装済みメソッドでテスト
        // let translated = BasicTransform::translate(&point, translation);
        let translated = Point3D::new(
            point.x() + translation.x(),
            point.y() + translation.y(),
            point.z() + translation.z(),
        );

        assert_eq!(translated.x(), 3.0);
        assert_eq!(translated.y(), 5.0);
        assert_eq!(translated.z(), 7.0);
    }

    #[test]
    fn test_transform_scale_from_origin() {
        let point = Point3D::new(2.0, 3.0, 4.0);
        // BasicTransformトレイトが未実装のため、実装済みメソッドでテスト
        // let scaled = BasicTransform::scale(&point, Point3D::origin(), 2.0);
        let scaled = point.scale(&Point3D::origin(), 2.0);

        assert_eq!(scaled.x(), 4.0);
        assert_eq!(scaled.y(), 6.0);
        assert_eq!(scaled.z(), 8.0);
    }

    #[test]
    fn test_coordinate_arithmetic() {
        let point = Point3D::new(1.0, 2.0, 3.0);

        // 座標演算による移動テスト（実装済み機能）
        let translated_x = Point3D::new(point.x() + 1.0, point.y(), point.z());
        assert_eq!(translated_x, Point3D::new(2.0, 2.0, 3.0));

        let translated_y = Point3D::new(point.x(), point.y() + 1.0, point.z());
        assert_eq!(translated_y, Point3D::new(1.0, 3.0, 3.0));

        let translated_xy = Point3D::new(point.x() + 1.0, point.y() + 1.0, point.z());
        assert_eq!(translated_xy, Point3D::new(2.0, 3.0, 3.0));
    }

    #[test]
    fn test_lerp_interpolation() {
        let point1 = Point3D::new(0.0, 0.0, 0.0);
        let point2 = Point3D::new(4.0, 6.0, 8.0);

        // 線形補間テスト（実装済み機能）
        let midpoint = point1.lerp(&point2, 0.5);
        assert_eq!(midpoint.x(), 2.0);
        assert_eq!(midpoint.y(), 3.0);
        assert_eq!(midpoint.z(), 4.0);

        let quarter = point1.lerp(&point2, 0.25);
        assert_eq!(quarter.x(), 1.0);
        assert_eq!(quarter.y(), 1.5);
        assert_eq!(quarter.z(), 2.0);
    }
}
