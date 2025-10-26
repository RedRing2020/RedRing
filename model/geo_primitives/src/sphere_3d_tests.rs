//! Sphere3D のテスト

use crate::{Point3D, Sphere3D, Vector3D};
use geo_foundation::tolerance_migration::DefaultTolerances;

fn assert_approx_eq(a: f64, b: f64, tolerance: f64) {
    assert!((a - b).abs() < tolerance, "Expected {}, got {}", b, a);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sphere_creation() {
        // 有効な球の作成
        let center = Point3D::new(1.0, 2.0, 3.0);
        let radius = 5.0;
        let sphere = Sphere3D::new(center, radius);
        assert!(sphere.is_some());

        let sphere = sphere.unwrap();
        assert_eq!(sphere.center(), center);
        assert_eq!(sphere.radius(), radius);

        // 無効な半径での作成
        assert!(Sphere3D::new(center, 0.0).is_none());
        assert!(Sphere3D::new(center, -1.0).is_none());
    }

    #[test]
    fn test_sphere_from_diameter() {
        let center = Point3D::new(0.0, 0.0, 0.0);
        let diameter = 10.0;
        let sphere = Sphere3D::from_diameter(center, diameter).unwrap();

        assert_eq!(sphere.radius(), 5.0);
        assert_eq!(sphere.diameter(), diameter);
    }

    #[test]
    fn test_sphere_at_origin() {
        let sphere = Sphere3D::new_at_origin(3.0).unwrap();
        assert_eq!(sphere.center(), Point3D::new(0.0, 0.0, 0.0));
        assert_eq!(sphere.radius(), 3.0);
    }

    #[test]
    fn test_geometric_properties() {
        let center = Point3D::new(0.0, 0.0, 0.0);
        let radius = 3.0;
        let sphere = Sphere3D::new(center, radius).unwrap();

        // 直径
        assert_eq!(sphere.diameter(), 6.0);

        // 表面積: S = 4πr²
        let expected_surface_area = 4.0 * std::f64::consts::PI * 9.0;
        assert_approx_eq(sphere.surface_area(), expected_surface_area, 1e-10);

        // 体積: V = (4/3)πr³
        let expected_volume = 4.0 * std::f64::consts::PI * 27.0 / 3.0;
        assert_approx_eq(sphere.volume(), expected_volume, 1e-10);
    }

    #[test]
    fn test_point_containment() {
        let center = Point3D::new(0.0, 0.0, 0.0);
        let radius = 5.0;
        let sphere = Sphere3D::new(center, radius).unwrap();

        // 内部の点
        assert!(sphere.contains_point(Point3D::new(0.0, 0.0, 0.0))); // 中心
        assert!(sphere.contains_point(Point3D::new(3.0, 4.0, 0.0))); // 距離5の点（境界上）
        assert!(sphere.contains_point(Point3D::new(1.0, 1.0, 1.0))); // 内部

        // 外部の点
        assert!(!sphere.contains_point(Point3D::new(6.0, 0.0, 0.0))); // X軸上遠方
        assert!(!sphere.contains_point(Point3D::new(4.0, 4.0, 0.0))); // 距離5.66の点
    }

    #[test]
    fn test_surface_distance() {
        let center = Point3D::new(0.0, 0.0, 0.0);
        let radius = 5.0;
        let sphere = Sphere3D::new(center, radius).unwrap();

        // 中心からの距離
        assert_approx_eq(sphere.distance_to_surface(center), -radius, 1e-10);

        // 表面上の点
        let surface_point = Point3D::new(5.0, 0.0, 0.0);
        assert_approx_eq(sphere.distance_to_surface(surface_point), 0.0, 1e-10);

        // 外部の点
        let external_point = Point3D::new(10.0, 0.0, 0.0);
        assert_approx_eq(sphere.distance_to_surface(external_point), 5.0, 1e-10);
    }

    #[test]
    fn test_point_on_surface() {
        let center = Point3D::new(0.0, 0.0, 0.0);
        let radius = 5.0;
        let sphere = Sphere3D::new(center, radius).unwrap();

        // 表面上の点の判定
        assert!(sphere.point_on_surface(Point3D::new(5.0, 0.0, 0.0)));
        assert!(sphere.point_on_surface(Point3D::new(0.0, 5.0, 0.0)));
        assert!(sphere.point_on_surface(Point3D::new(0.0, 0.0, 5.0)));
        assert!(sphere.point_on_surface(Point3D::new(3.0, 4.0, 0.0)));

        // 表面外の点の判定
        assert!(!sphere.point_on_surface(Point3D::new(0.0, 0.0, 0.0))); // 中心
        assert!(!sphere.point_on_surface(Point3D::new(6.0, 0.0, 0.0))); // 外部
    }

    #[test]
    fn test_surface_point_generation() {
        let center = Point3D::new(1.0, 2.0, 3.0);
        let radius = 4.0;
        let sphere = Sphere3D::new(center, radius).unwrap();

        // 方向ベクトルから表面点を生成
        let direction = Vector3D::new(1.0, 0.0, 0.0);
        let surface_point = sphere.point_on_surface_in_direction(direction).unwrap();

        let expected_point = Point3D::new(5.0, 2.0, 3.0); // center + (1,0,0) * radius
        assert_approx_eq(surface_point.x(), expected_point.x(), 1e-10);
        assert_approx_eq(surface_point.y(), expected_point.y(), 1e-10);
        assert_approx_eq(surface_point.z(), expected_point.z(), 1e-10);

        // ゼロベクトルの場合
        assert!(sphere
            .point_on_surface_in_direction(Vector3D::new(0.0, 0.0, 0.0))
            .is_none());
    }

    #[test]
    fn test_point_towards_target() {
        let center = Point3D::new(0.0, 0.0, 0.0);
        let radius = 5.0;
        let sphere = Sphere3D::new(center, radius).unwrap();

        let target = Point3D::new(10.0, 0.0, 0.0);
        let surface_point = sphere.point_on_surface_towards(target).unwrap();

        assert_approx_eq(surface_point.x(), 5.0, 1e-10);
        assert_approx_eq(surface_point.y(), 0.0, 1e-10);
        assert_approx_eq(surface_point.z(), 0.0, 1e-10);
    }

    #[test]
    fn test_bounding_box() {
        let center = Point3D::new(2.0, 3.0, 4.0);
        let radius = 1.5;
        let sphere = Sphere3D::new(center, radius).unwrap();

        let bbox = sphere.bounding_box();

        assert_approx_eq(bbox.min().x(), 0.5, 1e-10);
        assert_approx_eq(bbox.min().y(), 1.5, 1e-10);
        assert_approx_eq(bbox.min().z(), 2.5, 1e-10);

        assert_approx_eq(bbox.max().x(), 3.5, 1e-10);
        assert_approx_eq(bbox.max().y(), 4.5, 1e-10);
        assert_approx_eq(bbox.max().z(), 5.5, 1e-10);
    }

    #[test]
    fn test_transformations() {
        let center = Point3D::new(1.0, 1.0, 1.0);
        let radius = 2.0;
        let sphere = Sphere3D::new(center, radius).unwrap();

        // 平行移動
        let translation = Vector3D::new(3.0, 4.0, 5.0);
        let translated = sphere.translate(translation);
        assert_eq!(translated.center(), Point3D::new(4.0, 5.0, 6.0));
        assert_eq!(translated.radius(), radius);

        // 均等スケーリング
        let scaled = sphere.scale(2.0).unwrap();
        assert_eq!(scaled.center(), center);
        assert_eq!(scaled.radius(), 4.0);

        // 無効なスケーリング
        assert!(sphere.scale(0.0).is_none());
        assert!(sphere.scale(-1.0).is_none());
    }

    #[test]
    fn test_scale_about_point() {
        let center = Point3D::new(2.0, 2.0, 2.0);
        let radius = 1.0;
        let sphere = Sphere3D::new(center, radius).unwrap();

        let scale_center = Point3D::new(0.0, 0.0, 0.0);
        let factor = 2.0;
        let scaled = sphere.scale_about_point(scale_center, factor).unwrap();

        // 中心が (2,2,2) → (4,4,4) に移動
        assert_eq!(scaled.center(), Point3D::new(4.0, 4.0, 4.0));
        // 半径が 1.0 → 2.0 に拡大
        assert_eq!(scaled.radius(), 2.0);
    }

    #[test]
    fn test_degenerate_sphere() {
        let center = Point3D::new(0.0, 0.0, 0.0);

        // 通常の球
        let normal_sphere = Sphere3D::new(center, 1.0).unwrap();
        assert!(!normal_sphere.is_degenerate());

        // 非常に小さい球（退化している）
        let tiny_radius = DefaultTolerances::distance::<f64>() / 2.0;
        let tiny_sphere = Sphere3D::new(center, tiny_radius).unwrap();
        assert!(tiny_sphere.is_degenerate());
    }

    #[test]
    fn test_sphere_display() {
        let center = Point3D::new(1.0, 2.0, 3.0);
        let radius = 5.0;
        let sphere = Sphere3D::new(center, radius).unwrap();

        let display_str = format!("{}", sphere);
        assert!(display_str.contains("Sphere3D"));
        assert!(display_str.contains("center:"));
        assert!(display_str.contains("radius:"));
    }
}
