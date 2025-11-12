//! Point3D のテスト

use crate::{Point3D, Vector3D};
use geo_foundation::BasicTransform;

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
        use geo_foundation::extensions::BasicTransform;

        let point = Point3D::new(1.0, 2.0, 3.0);
        let translation = Vector3D::new(2.0, 3.0, 4.0);
        let translated = BasicTransform::translate(&point, translation);

        assert_eq!(translated.x(), 3.0);
        assert_eq!(translated.y(), 5.0);
        assert_eq!(translated.z(), 7.0);
    }

    #[test]
    fn test_transform_scale_from_origin() {
        let point = Point3D::new(2.0, 3.0, 4.0);
        let scaled = BasicTransform::scale(&point, Point3D::origin(), 2.0);

        assert_eq!(scaled.x(), 4.0);
        assert_eq!(scaled.y(), 6.0);
        assert_eq!(scaled.z(), 8.0);
    }

    #[test]
    fn test_transform_translate_axes() {
        let point = Point3D::new(1.0, 2.0, 3.0);

        let translated_x = BasicTransform::translate(&point, Vector3D::new(1.0, 0.0, 0.0));
        assert_eq!(translated_x, Point3D::new(2.0, 2.0, 3.0));

        let translated_y = BasicTransform::translate(&point, Vector3D::new(0.0, 1.0, 0.0));
        assert_eq!(translated_y, Point3D::new(1.0, 3.0, 3.0));

        let translated_xy = BasicTransform::translate(&point, Vector3D::new(1.0, 1.0, 0.0));
        assert_eq!(translated_xy, Point3D::new(2.0, 3.0, 3.0));
    }

    #[test]
    fn test_transform_scale_from_center() {
        let point = Point3D::new(4.0, 6.0, 8.0);
        let center = Point3D::new(2.0, 3.0, 4.0);
        let scaled = BasicTransform::scale(&point, center, 2.0);

        // (4-2)*2+2=6, (6-3)*2+3=9, (8-4)*2+4=12
        assert_eq!(scaled.x(), 6.0);
        assert_eq!(scaled.y(), 9.0);
        assert_eq!(scaled.z(), 12.0);
    }
}
