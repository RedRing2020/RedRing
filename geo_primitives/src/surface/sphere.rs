//! 球（Sphere）の具体的実装
//!
//! 3次元空間における球面および球体の実装を提供する。

use geo_foundation::Sphere3D as Sphere3DTrait;
use geo_foundation::{SphereKind, SphericalCoordinates};
use std::fmt;

/// 3次元球の実装（f64固定）
///
/// 中心点と半径で定義される球面・球体
#[derive(Debug, Clone, PartialEq)]
pub struct Sphere {
    center: crate::geometry3d::Point,
    radius: f64,
}

impl Sphere {
    /// 新しい球を作成
    ///
    /// # Arguments
    /// * `center` - 中心点
    /// * `radius` - 半径（0以上の値）
    ///
    /// # Examples
    /// ```
    /// use geo_primitives::geometry3d::Point;
    /// use geo_primitives::surface::Sphere;
    /// use geo_foundation::Sphere3D; // トレイトをインポート
    ///
    /// let center = Point::new(1.0, 2.0, 3.0);
    /// let sphere = Sphere::new(center, 5.0);
    /// assert_eq!(sphere.radius(), 5.0);
    /// ```
    pub fn new(center: crate::geometry3d::Point, radius: f64) -> Self {
        Self { center, radius }
    }

    /// 単位球を作成（原点中心、半径1）
    pub fn unit() -> Self {
        Self::new(crate::geometry3d::Point::origin(), 1.0)
    }

    /// 原点中心の球を作成
    pub fn from_radius(radius: f64) -> Self {
        Self::new(crate::geometry3d::Point::origin(), radius)
    }
}

impl Sphere3DTrait for Sphere {
    type Scalar = f64;
    type Point = crate::geometry3d::Point;
    type Vector = crate::geometry3d::Vector;

    fn center(&self) -> Self::Point {
        self.center
    }

    fn radius(&self) -> Self::Scalar {
        self.radius
    }

    fn surface_area(&self) -> Self::Scalar {
        // 4πr²
        let four = 4.0;
        let r_squared = self.radius * self.radius;
        four * std::f64::consts::PI * r_squared
    }

    fn volume(&self) -> Self::Scalar {
        // (4/3)πr³
        let four_thirds = 4.0 / 3.0;
        let r_cubed = self.radius * self.radius * self.radius;
        four_thirds * std::f64::consts::PI * r_cubed
    }

    fn point_on_surface(&self, point: &Self::Point) -> bool {
        let distance = self.distance_from_center(point);
        (distance - self.radius).abs() < f64::EPSILON
    }

    fn contains_point(&self, point: &Self::Point) -> bool {
        let distance = self.distance_from_center(point);
        distance <= self.radius + f64::EPSILON
    }

    fn contains_point_strict(&self, point: &Self::Point) -> bool {
        let distance = self.distance_from_center(point);
        distance < self.radius - f64::EPSILON
    }

    fn point_at_spherical(&self, theta: Self::Scalar, phi: Self::Scalar) -> Self::Point {
        // 球面座標から直交座標への変換
        // x = r * sin(φ) * cos(θ)
        // y = r * sin(φ) * sin(θ)
        // z = r * cos(φ)
        let sin_phi = phi.sin();
        let cos_phi = phi.cos();
        let sin_theta = theta.sin();
        let cos_theta = theta.cos();

        let x = self.radius * sin_phi * cos_theta;
        let y = self.radius * sin_phi * sin_theta;
        let z = self.radius * cos_phi;

        crate::geometry3d::Point::new(
            self.center.x() + x,
            self.center.y() + y,
            self.center.z() + z,
        )
    }

    fn distance_from_center(&self, point: &Self::Point) -> Self::Scalar {
        let diff = *point - self.center;
        diff.length()
    }

    fn distance_to_surface(&self, point: &Self::Point) -> Self::Scalar {
        let distance_from_center = self.distance_from_center(point);
        distance_from_center - self.radius
    }

    fn translated(&self, offset: &Self::Vector) -> Self {
        Self::new(self.center + *offset, self.radius)
    }

    fn scaled(&self, factor: Self::Scalar) -> Self {
        Self::new(self.center, self.radius * factor)
    }

    fn bounding_box(&self) -> (Self::Point, Self::Point) {
        let r = self.radius;
        let min_point = crate::geometry3d::Point::new(
            self.center.x() - r,
            self.center.y() - r,
            self.center.z() - r,
        );
        let max_point = crate::geometry3d::Point::new(
            self.center.x() + r,
            self.center.y() + r,
            self.center.z() + r,
        );
        (min_point, max_point)
    }

    fn intersects_sphere(&self, other: &Self) -> bool {
        let center_distance = self.distance_from_center(&other.center);
        center_distance <= self.radius + other.radius + f64::EPSILON
    }

    fn sphere_kind(&self) -> SphereKind {
        if self.radius < f64::EPSILON {
            SphereKind::Degenerate
        } else if (self.radius - 1.0).abs() < f64::EPSILON
            && self.center.distance_to(&crate::geometry3d::Point::origin()) < f64::EPSILON
        {
            SphereKind::Unit
        } else {
            SphereKind::Standard
        }
    }
}

impl SphericalCoordinates for Sphere {
    type Scalar = f64;
    type Point = crate::geometry3d::Point;

    fn cartesian_to_spherical(
        &self,
        point: &Self::Point,
    ) -> (Self::Scalar, Self::Scalar, Self::Scalar) {
        let relative = *point - self.center;
        let x = relative.x();
        let y = relative.y();
        let z = relative.z();

        let r = (x * x + y * y + z * z).sqrt();
        let theta = y.atan2(x); // 方位角
        let phi = if r > f64::EPSILON {
            (z / r).acos() // 仰角
        } else {
            0.0
        };

        (r, theta, phi)
    }

    fn spherical_to_cartesian(
        &self,
        r: Self::Scalar,
        theta: Self::Scalar,
        phi: Self::Scalar,
    ) -> Self::Point {
        let sin_phi = phi.sin();
        let cos_phi = phi.cos();
        let sin_theta = theta.sin();
        let cos_theta = theta.cos();

        let x = r * sin_phi * cos_theta;
        let y = r * sin_phi * sin_theta;
        let z = r * cos_phi;

        crate::geometry3d::Point::new(
            self.center.x() + x,
            self.center.y() + y,
            self.center.z() + z,
        )
    }
}

impl fmt::Display for Sphere {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Sphere(center: {:?}, radius: {})",
            self.center, self.radius
        )
    }
}

// 型エイリアス
pub type Sphere3D = Sphere; // デフォルトはf64

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry3d::Point;

    #[test]
    fn test_sphere_creation() {
        let center = Point::new(1.0, 2.0, 3.0);
        let sphere = Sphere::new(center, 5.0);

        assert_eq!(sphere.center(), center);
        assert_eq!(sphere.radius(), 5.0);
    }

    #[test]
    fn test_unit_sphere() {
        let sphere = Sphere::unit();

        assert_eq!(sphere.center(), Point::origin());
        assert_eq!(sphere.radius(), 1.0);
        assert_eq!(sphere.sphere_kind(), SphereKind::Unit);
    }

    #[test]
    fn test_surface_area_and_volume() {
        let sphere = Sphere::from_radius(2.0);

        // 表面積 = 4πr² = 4π(4) = 16π
        let expected_area = 16.0 * std::f64::consts::PI;
        assert!((sphere.surface_area() - expected_area).abs() < 1e-10);

        // 体積 = (4/3)πr³ = (4/3)π(8) = (32/3)π
        let expected_volume = (32.0 / 3.0) * std::f64::consts::PI;
        assert!((sphere.volume() - expected_volume).abs() < 1e-10);
    }

    #[test]
    fn test_point_containment() {
        let sphere = Sphere::from_radius(5.0);

        // 中心点
        assert!(sphere.contains_point(&Point::origin()));
        assert!(sphere.contains_point_strict(&Point::origin()));

        // 球面上の点
        let surface_point = Point::new(5.0, 0.0, 0.0);
        assert!(sphere.point_on_surface(&surface_point));
        assert!(sphere.contains_point(&surface_point));
        assert!(!sphere.contains_point_strict(&surface_point));

        // 内部の点
        let inner_point = Point::new(3.0, 0.0, 0.0);
        assert!(!sphere.point_on_surface(&inner_point));
        assert!(sphere.contains_point(&inner_point));
        assert!(sphere.contains_point_strict(&inner_point));

        // 外部の点
        let outer_point = Point::new(6.0, 0.0, 0.0);
        assert!(!sphere.point_on_surface(&outer_point));
        assert!(!sphere.contains_point(&outer_point));
        assert!(!sphere.contains_point_strict(&outer_point));
    }

    #[test]
    fn test_spherical_coordinates() {
        let sphere = Sphere::from_radius(1.0);

        // θ=0, φ=π/2 (x軸正方向)
        let point = sphere.point_at_spherical(0.0, std::f64::consts::PI / 2.0);
        let expected = Point::new(1.0, 0.0, 0.0);

        assert!((point.x() - expected.x()).abs() < 1e-10);
        assert!((point.y() - expected.y()).abs() < 1e-10);
        assert!((point.z() - expected.z()).abs() < 1e-10);
    }

    #[test]
    fn test_transformations() {
        let original = Sphere::new(Point::new(1.0, 2.0, 3.0), 4.0);

        // 平行移動
        let offset = crate::geometry3d::Vector::new(1.0, -1.0, 2.0);
        let translated = original.translated(&offset);
        let expected_center = Point::new(2.0, 1.0, 5.0);
        assert_eq!(translated.center(), expected_center);
        assert_eq!(translated.radius(), 4.0);

        // 拡大縮小
        let scaled = original.scaled(2.0);
        assert_eq!(scaled.center(), original.center());
        assert_eq!(scaled.radius(), 8.0);
    }

    #[test]
    fn test_sphere_intersection() {
        let sphere1 = Sphere::new(Point::new(0.0, 0.0, 0.0), 5.0);
        let sphere2 = Sphere::new(Point::new(8.0, 0.0, 0.0), 4.0);
        let sphere3 = Sphere::new(Point::new(15.0, 0.0, 0.0), 3.0);

        // 交差する球
        assert!(sphere1.intersects_sphere(&sphere2));

        // 交差しない球
        assert!(!sphere1.intersects_sphere(&sphere3));
    }

    #[test]
    fn test_bounding_box() {
        let sphere = Sphere::new(Point::new(2.0, 3.0, 4.0), 1.5);
        let (min, max) = sphere.bounding_box();

        assert_eq!(min, Point::new(0.5, 1.5, 2.5));
        assert_eq!(max, Point::new(3.5, 4.5, 5.5));
    }
}
