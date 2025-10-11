//! 球（Sphere）のテスト
//!
//! SphereF64とSphereF32の両方の実装をテストする

use super::test_utils::helpers::*;
use super::test_utils::*;
use crate::surface::{SphereF32, SphereF64};
use geo_foundation::{Sphere as SphereTrait, SphereKind};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sphere_creation() {
        let center = TestPoint3D::new(1.0, 2.0, 3.0);
        let sphere = SphereF64::new(center, 5.0);

        assert_eq!(sphere.center(), center);
        assert_eq!(sphere.radius(), 5.0);
    }

    #[test]
    fn test_unit_sphere() {
        let sphere = SphereF64::unit();

        assert_eq!(sphere.center(), TestPoint3D::origin());
        assert_eq!(sphere.radius(), 1.0);
        assert_eq!(sphere.sphere_kind(), SphereKind::Unit);
    }

    #[test]
    fn test_surface_area_and_volume() {
        let sphere = SphereF64::from_radius(2.0);

        // 表面積 = 4πr² = 4π(4) = 16π
        let expected_area = 16.0 * std::f64::consts::PI;
        assert!(approx_eq_f64_with_tolerance(
            sphere.surface_area(),
            expected_area,
            1e-10
        ));

        // 体積 = (4/3)πr³ = (4/3)π(8) = (32/3)π
        let expected_volume = (32.0 / 3.0) * std::f64::consts::PI;
        assert!(approx_eq_f64_with_tolerance(
            sphere.volume(),
            expected_volume,
            1e-10
        ));
    }

    #[test]
    fn test_point_containment() {
        let sphere = SphereF64::from_radius(5.0);

        // 中心点
        assert!(sphere.contains_point(&TestPoint3D::origin()));

        // 球面上の点
        let surface_point = TestPoint3D::new(5.0, 0.0, 0.0);
        assert!(sphere.contains_point(&surface_point));

        // 内部の点
        let inside_point = TestPoint3D::new(2.0, 3.0, 0.0);
        assert!(sphere.contains_point(&inside_point));

        // 外部の点
        let outside_point = TestPoint3D::new(10.0, 0.0, 0.0);
        assert!(!sphere.contains_point(&outside_point));
    }

    #[test]
    fn test_basic_sphere_operations() {
        let sphere = SphereF64::from_radius(2.0);

        // 基本的な操作のテスト（球面座標系ではなく）
        assert_eq!(sphere.center(), TestPoint3D::origin());
        assert_eq!(sphere.radius(), 2.0);

        // 球面上の点を確認する（球面座標を使わずに）
        let point_on_sphere = TestPoint3D::new(2.0, 0.0, 0.0);
        assert!(sphere.point_on_surface(&point_on_sphere));
    }

    #[test]
    fn test_sphere_intersection() {
        let sphere1 = SphereF64::new(TestPoint3D::new(0.0, 0.0, 0.0), 3.0);
        let sphere2 = SphereF64::new(TestPoint3D::new(4.0, 0.0, 0.0), 3.0);

        // 交差している球
        assert!(sphere1.intersects_sphere(&sphere2));

        // 離れている球
        let sphere3 = SphereF64::new(TestPoint3D::new(10.0, 0.0, 0.0), 3.0);
        assert!(!sphere1.intersects_sphere(&sphere3));

        // 内部に含まれる球
        let sphere4 = SphereF64::new(TestPoint3D::new(0.0, 0.0, 0.0), 1.0);
        assert!(sphere1.intersects_sphere(&sphere4));
    }

    #[test]
    fn test_sphere_transformations() {
        let sphere = SphereF64::from_radius(2.0);

        // 平行移動
        let translation = TestVector3D::new(3.0, 4.0, 5.0);
        let translated = sphere.translated(&translation);
        let expected_center = TestPoint3D::new(3.0, 4.0, 5.0);
        assert_eq!(translated.center(), expected_center);
        assert_eq!(translated.radius(), sphere.radius());

        // スケール変換
        let scaled = sphere.scaled(2.0);
        assert_eq!(scaled.center(), sphere.center());
        assert_eq!(scaled.radius(), 4.0);
    }

    #[test]
    fn test_distance_to_point() {
        let sphere = SphereF64::from_radius(5.0);

        // 中心から外部の点への距離
        let external_point = TestPoint3D::new(10.0, 0.0, 0.0);
        let distance = sphere.distance_to_surface(&external_point);
        assert!(approx_eq_f64_with_tolerance(distance, 5.0, 1e-10));

        // 球面上の点
        let surface_point = TestPoint3D::new(5.0, 0.0, 0.0);
        let surface_distance = sphere.distance_to_surface(&surface_point);
        assert!(approx_eq_f64_with_tolerance(surface_distance, 0.0, 1e-10));

        // 内部の点
        let inner_point = TestPoint3D::new(2.0, 0.0, 0.0);
        let inner_surface_distance = sphere.distance_to_surface(&inner_point);
        assert!(approx_eq_f64_with_tolerance(
            inner_surface_distance,
            -3.0,
            1e-10
        ));
    }

    #[test]
    fn test_distance_from_center() {
        let sphere = SphereF64::from_radius(5.0);

        // 中心からの距離を確認
        let external_point = TestPoint3D::new(10.0, 0.0, 0.0);
        let distance = sphere.distance_from_center(&external_point);
        assert!(approx_eq_f64_with_tolerance(distance, 10.0, 1e-10));

        // 球面上の点
        let surface_point = TestPoint3D::new(5.0, 0.0, 0.0);
        let surface_distance = sphere.distance_from_center(&surface_point);
        assert!(approx_eq_f64_with_tolerance(surface_distance, 5.0, 1e-10));
    }

    // f32精度の球をテスト
    #[test]
    fn test_sphere_f32_precision() {
        let center = TestPoint3D::new(1.0, 2.0, 3.0);
        let sphere_f64 = SphereF64::new(center, 5.0);
        let sphere_f32 = SphereF32::new(center, 5.0);

        // f64とf32で基本的な操作が動作することを確認（精度の違いを考慮）
        assert!(approx_eq_f64_with_tolerance(
            sphere_f64.radius(),
            sphere_f32.radius() as f64,
            1e-6
        ));
        assert!(approx_eq_f64_with_tolerance(
            sphere_f64.surface_area(),
            sphere_f32.surface_area() as f64,
            1e-3
        ));
    }

    #[test]
    fn test_sphere_bounding_box() {
        let center = TestPoint3D::new(2.0, 3.0, 4.0);
        let sphere = SphereF64::new(center, 1.5);

        let bounding_box = sphere.bounding_box();

        // バウンディングボックスはタプル形式で返される
        let (min_point, max_point) = bounding_box;

        // 最小点の確認
        assert!(approx_eq_f64(min_point.x(), 0.5));
        assert!(approx_eq_f64(min_point.y(), 1.5));
        assert!(approx_eq_f64(min_point.z(), 2.5));

        // 最大点の確認
        assert!(approx_eq_f64(max_point.x(), 3.5));
        assert!(approx_eq_f64(max_point.y(), 4.5));
        assert!(approx_eq_f64(max_point.z(), 5.5));
    }

    #[test]
    fn test_sphere_normed_trait() {
        let sphere = SphereF64::from_radius(3.0);

        // 球の「ノルム」として半径を返すことを確認
        assert!(approx_eq_f64(sphere.radius(), 3.0));
    }

    #[test]
    fn test_sphere_clone_and_equality() {
        let sphere1 = SphereF64::new(TestPoint3D::new(1.0, 2.0, 3.0), 5.0);
        let sphere2 = sphere1.clone();

        assert_eq!(sphere1, sphere2);
        assert_eq!(sphere1.center(), sphere2.center());
        assert_eq!(sphere1.radius(), sphere2.radius());
    }
}
