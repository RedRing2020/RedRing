//! Point3D のテスト

use crate::Point3D;
use geo_foundation::abstract_types::geometry::foundation::*;

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
    fn test_geometry_foundation() {
        let point = Point3D::new(1.0, 2.0, 3.0);
        let bbox = point.bounding_box();

        // 点の境界ボックス = 点自身
        assert_eq!(bbox.min(), point);
        assert_eq!(bbox.max(), point);
    }

    #[test]
    fn test_basic_containment() {
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
        assert_eq!(point.distance_to_point(&same_point), 0.0);
        assert_eq!(point.distance_to_point(&Point3D::new(1.0, 2.0, 6.0)), 3.0);
    }
}
