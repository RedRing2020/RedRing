/// 球面サーフェスプリミティブ

use crate::geometry3d::Point3D;
use geo_core::{Vector3D, Scalar};
use crate::{GeometricPrimitive, PrimitiveKind, BoundingBox};

/// 3D球面
#[derive(Debug, Clone)]
pub struct Sphere {
    center: Point3D,
    radius: f64,
}

impl Sphere {
    /// 新しい球面を作成
    pub fn new(center: Point3D, radius: f64) -> Option<Self> {
        if radius <= 0.0 {
            None
        } else {
            Some(Self { center, radius })
        }
    }

    /// 中心点を取得
    pub fn center(&self) -> &Point3D {
        &self.center
    }

    /// 半径を取得
    pub fn radius(&self) -> f64 {
        self.radius
    }

    /// パラメトリック評価 (u: [0, 2π], v: [0, π])
    pub fn evaluate(&self, u: f64, v: f64) -> Point3D {
        let x = self.radius * u.cos() * v.sin();
        let y = self.radius * u.sin() * v.sin();
        let z = self.radius * v.cos();
        Point3D::new(
            self.center.x() + x,
            self.center.y() + y,
            self.center.z() + z,
        )
    }

    /// u方向の偏微分ベクトル
    pub fn derivative_u(&self, u: f64, v: f64) -> Vector3D {
        let dx = -self.radius * u.sin() * v.sin();
        let dy = self.radius * u.cos() * v.sin();
        let dz = 0.0;
        Vector3D::new(Scalar::new(dx), Scalar::new(dy), Scalar::new(dz))
    }

    /// v方向の偏微分ベクトル
    pub fn derivative_v(&self, u: f64, v: f64) -> Vector3D {
        let dx = self.radius * u.cos() * v.cos();
        let dy = self.radius * u.sin() * v.cos();
        let dz = -self.radius * v.sin();
        Vector3D::new(Scalar::new(dx), Scalar::new(dy), Scalar::new(dz))
    }

    /// 指定パラメータでの法線ベクトル
    pub fn normal(&self, u: f64, v: f64) -> Vector3D {
        let point = self.evaluate(u, v);
        let to_center = self.center.vector_to(&point);
        // 球面では、中心から表面への方向が法線
        // 長さを1に正規化
        let length = (to_center.x().value().powi(2) + to_center.y().value().powi(2) + to_center.z().value().powi(2)).sqrt();
        if length > 1e-12 {
            Vector3D::new(
                Scalar::new(to_center.x().value() / length),
                Scalar::new(to_center.y().value() / length),
                Scalar::new(to_center.z().value() / length),
            )
        } else {
            Vector3D::new(Scalar::new(0.0), Scalar::new(0.0), Scalar::new(1.0))
        }
    }

    /// 球面の表面積
    pub fn surface_area(&self) -> f64 {
        4.0 * std::f64::consts::PI * self.radius * self.radius
    }

    /// 球の体積
    pub fn volume(&self) -> f64 {
        (4.0 / 3.0) * std::f64::consts::PI * self.radius * self.radius * self.radius
    }

    /// 点が球面上にあるかを判定
    pub fn contains_point(&self, point: &Point3D, tolerance: f64) -> bool {
        let distance = self.center.distance_to(point);
        (distance - self.radius).abs() < tolerance
    }

    /// 点が球の内部にあるかを判定
    pub fn contains_point_inside(&self, point: &Point3D) -> bool {
        self.center.distance_to(point) < self.radius
    }

    /// 点から球面への最近点
    pub fn closest_point(&self, point: &Point3D) -> Point3D {
        let to_point = self.center.vector_to(point);
        let length = (to_point.x().value().powi(2) + to_point.y().value().powi(2) + to_point.z().value().powi(2)).sqrt();
        if length > 1e-12 {
            let normalized_x = to_point.x().value() / length;
            let normalized_y = to_point.y().value() / length;
            let normalized_z = to_point.z().value() / length;
            Point3D::new(
                self.center.x() + self.radius * normalized_x,
                self.center.y() + self.radius * normalized_y,
                self.center.z() + self.radius * normalized_z,
            )
        } else {
            // 点が中心と同じ場合、任意の球面上の点を返す
            Point3D::new(self.center.x() + self.radius, self.center.y(), self.center.z())
        }
    }

    /// 半径を変更した新しい球を作成
    pub fn with_radius(&self, new_radius: f64) -> Option<Sphere> {
        Self::new(self.center.clone(), new_radius)
    }

    /// 中心を移動した新しい球を作成
    pub fn translate(&self, dx: f64, dy: f64, dz: f64) -> Sphere {
        Self {
            center: self.center.translate(dx, dy, dz),
            radius: self.radius,
        }
    }

    /// 指定点を中心にスケール
    pub fn scale(&self, factor: f64, center: &Point3D) -> Option<Sphere> {
        if factor <= 0.0 {
            return None;
        }

        let new_center = self.center.scale(factor, center);
        let new_radius = self.radius * factor;

        Self::new(new_center, new_radius)
    }
}

impl GeometricPrimitive for Sphere {
    fn primitive_kind(&self) -> PrimitiveKind {
        PrimitiveKind::Sphere
    }

    fn bounding_box(&self) -> BoundingBox {
        let min = Point3D::new(
            self.center.x() - self.radius,
            self.center.y() - self.radius,
            self.center.z() - self.radius,
        ).to_geo_core();
        let max = Point3D::new(
            self.center.x() + self.radius,
            self.center.y() + self.radius,
            self.center.z() + self.radius,
        ).to_geo_core();
        BoundingBox::new(min, max)
    }

    fn measure(&self) -> Option<f64> {
        Some(self.surface_area())
    }
}

impl PartialEq for Sphere {
    fn eq(&self, other: &Self) -> bool {
        self.center == other.center && (self.radius - other.radius).abs() < 1e-10
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sphere_creation() {
        let center = Point3D::new(1.0, 2.0, 3.0);
        let sphere = Sphere::new(center, 5.0).unwrap();
        assert_eq!(sphere.radius(), 5.0);
    }

    #[test]
    fn test_sphere_invalid_radius() {
        let center = Point3D::new(0.0, 0.0, 0.0);
        assert!(Sphere::new(center.clone(), -1.0).is_none());
        assert!(Sphere::new(center, 0.0).is_none());
    }

    #[test]
    fn test_sphere_evaluate() {
        let center = Point3D::new(0.0, 0.0, 0.0);
        let sphere = Sphere::new(center, 1.0).unwrap();

        // 北極点 (u=0, v=0)
        let north_pole = sphere.evaluate(0.0, 0.0);
        assert!((north_pole.x() - 0.0).abs() < 1e-10);
        assert!((north_pole.y() - 0.0).abs() < 1e-10);
        assert!((north_pole.z() - 1.0).abs() < 1e-10);

        // 赤道上の点 (u=0, v=π/2)
        let equator_point = sphere.evaluate(0.0, std::f64::consts::PI / 2.0);
        assert!((equator_point.x() - 1.0).abs() < 1e-10);
        assert!((equator_point.y() - 0.0).abs() < 1e-10);
        assert!((equator_point.z() - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_sphere_surface_area_and_volume() {
        let center = Point3D::new(0.0, 0.0, 0.0);
        let sphere = Sphere::new(center, 2.0).unwrap();

        let expected_surface_area = 4.0 * std::f64::consts::PI * 4.0;
        let expected_volume = (4.0 / 3.0) * std::f64::consts::PI * 8.0;

        assert!((sphere.surface_area() - expected_surface_area).abs() < 1e-10);
        assert!((sphere.volume() - expected_volume).abs() < 1e-10);
    }

    #[test]
    fn test_sphere_contains_point() {
        let center = Point3D::new(0.0, 0.0, 0.0);
        let sphere = Sphere::new(center, 1.0).unwrap();

        let on_sphere = Point3D::new(1.0, 0.0, 0.0);
        let inside = Point3D::new(0.5, 0.0, 0.0);
        let outside = Point3D::new(2.0, 0.0, 0.0);

        assert!(sphere.contains_point(&on_sphere, 1e-10));
        assert!(sphere.contains_point_inside(&inside));
        assert!(!sphere.contains_point_inside(&outside));
    }

    #[test]
    fn test_sphere_closest_point() {
        let center = Point3D::new(0.0, 0.0, 0.0);
        let sphere = Sphere::new(center, 1.0).unwrap();

        let point = Point3D::new(2.0, 0.0, 0.0);
        let closest = sphere.closest_point(&point);

        assert!((closest.x() - 1.0).abs() < 1e-10);
        assert!((closest.y() - 0.0).abs() < 1e-10);
        assert!((closest.z() - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_sphere_translate() {
        let center = Point3D::new(1.0, 1.0, 1.0);
        let sphere = Sphere::new(center, 2.0).unwrap();
        let translated = sphere.translate(3.0, 4.0, 5.0);

        assert_eq!(translated.center().x(), 4.0);
        assert_eq!(translated.center().y(), 5.0);
        assert_eq!(translated.center().z(), 6.0);
        assert_eq!(translated.radius(), 2.0);
    }

    #[test]
    fn test_sphere_scale() {
        let center = Point3D::new(1.0, 1.0, 1.0);
        let sphere = Sphere::new(center, 2.0).unwrap();
        let scale_center = Point3D::new(0.0, 0.0, 0.0);
        let scaled = sphere.scale(2.0, &scale_center).unwrap();

        assert_eq!(scaled.center().x(), 2.0);
        assert_eq!(scaled.center().y(), 2.0);
        assert_eq!(scaled.center().z(), 2.0);
        assert_eq!(scaled.radius(), 4.0);
    }
}
