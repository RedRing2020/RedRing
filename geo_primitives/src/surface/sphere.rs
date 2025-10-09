//! 球（Sphere）の具体的実装
//!
//! 3次元空間における球面および球体の実装を提供する。

use crate::geometry3d::point::{Point3DF32, Point3DF64};
use geo_foundation::{game, precision};
use geo_foundation::Sphere as SphereTrait;
use geo_foundation::{SphereKind, SphericalCoordinates};
use std::fmt;

/// 3次元球の実装（f64版）
///
/// 中心点と半径で定義される球面・球体
#[derive(Debug, Clone, PartialEq)]
pub struct SphereF64 {
    center: Point3DF64,
    radius: f64,
}

/// 3次元球の実装（f32版）
///
/// 中心点と半径で定義される球面・球体
#[derive(Debug, Clone, PartialEq)]
pub struct SphereF32 {
    center: Point3DF32,
    radius: f32,
}

impl SphereF64 {
    /// 新しい球を作成
    ///
    /// # Arguments
    /// * `center` - 中心点
    /// * `radius` - 半径（0以上の値）
    ///
    /// # Examples
    /// ```
    /// use geo_primitives::geometry3d::Point3D;
    /// use geo_primitives::surface::SphereF64;
    /// use geo_foundation::Sphere as SphereTrait; // トレイトをインポート
    ///
    /// let center = Point3D::new(1.0, 2.0, 3.0);
    /// let sphere = SphereF64::new(center, 5.0);
    /// assert_eq!(sphere.radius(), 5.0);
    /// ```
    pub fn new(center: crate::geometry3d::Point3D, radius: f64) -> Self {
        Self { center, radius }
    }

    /// 単位球を作成（原点中心、半径1）
    pub fn unit() -> Self {
        Self::new(crate::geometry3d::Point3D::origin(), 1.0)
    }

    /// 原点中心の球を作成
    pub fn from_radius(radius: f64) -> Self {
        Self::new(crate::geometry3d::Point3D::origin(), radius)
    }
}

impl SphereF32 {
    /// 新しい球を作成
    ///
    /// # Arguments
    /// * `center` - 中心点
    /// * `radius` - 半径（0以上の値）
    pub fn new(center: crate::geometry3d::Point3D, radius: f32) -> Self {
        Self { center, radius }
    }

    /// 単位球を作成（原点中心、半径1）
    pub fn unit() -> Self {
        Self::new(crate::geometry3d::Point3D::origin(), 1.0)
    }

    /// 原点中心の球を作成
    pub fn from_radius(radius: f32) -> Self {
        Self::new(crate::geometry3d::Point3D::origin(), radius)
    }
}

impl SphereTrait for SphereF64 {
    type Scalar = f64;
    type Point = crate::geometry3d::Point3D;
    type Vector = crate::geometry3d::Vector3D;

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
        four * precision::PI * r_squared
    }

    fn volume(&self) -> Self::Scalar {
        // (4/3)πr³
        let four_thirds = 4.0 / 3.0;
        let r_cubed = self.radius * self.radius * self.radius;
        four_thirds * precision::PI * r_cubed
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

        crate::geometry3d::Point3D::new(
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
        let min_point = crate::geometry3d::Point3D::new(
            self.center.x() - r,
            self.center.y() - r,
            self.center.z() - r,
        );
        let max_point = crate::geometry3d::Point3D::new(
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
            && self
                .center
                .distance_to(&crate::geometry3d::Point3D::origin())
                < f64::EPSILON
        {
            SphereKind::Unit
        } else {
            SphereKind::Standard
        }
    }
}

impl SphereTrait for SphereF32 {
    type Scalar = f32;
    type Point = crate::geometry3d::Point3D;
    type Vector = crate::geometry3d::Vector3D;

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
        four * game::PI * r_squared
    }

    fn volume(&self) -> Self::Scalar {
        // (4/3)πr³
        let four_thirds = 4.0 / 3.0;
        let r_cubed = self.radius * self.radius * self.radius;
        four_thirds * game::PI * r_cubed
    }

    fn point_on_surface(&self, point: &Self::Point) -> bool {
        let distance = self.distance_from_center(point);
        (distance - self.radius).abs() < f32::EPSILON
    }

    fn contains_point(&self, point: &Self::Point) -> bool {
        let distance = self.distance_from_center(point);
        distance <= self.radius + f32::EPSILON
    }

    fn contains_point_strict(&self, point: &Self::Point) -> bool {
        let distance = self.distance_from_center(point);
        distance < self.radius - f32::EPSILON
    }

    fn point_at_spherical(&self, theta: Self::Scalar, phi: Self::Scalar) -> Self::Point {
        let sin_phi = phi.sin();
        let cos_phi = phi.cos();
        let sin_theta = theta.sin();
        let cos_theta = theta.cos();

        let x = self.radius * sin_phi * cos_theta;
        let y = self.radius * sin_phi * sin_theta;
        let z = self.radius * cos_phi;

        crate::geometry3d::Point3D::new(
            self.center.x() + x as f64,
            self.center.y() + y as f64,
            self.center.z() + z as f64,
        )
    }

    fn distance_from_center(&self, point: &Self::Point) -> Self::Scalar {
        let diff = *point - self.center;
        diff.length() as f32
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
        let r = self.radius as f64;
        let min_point = crate::geometry3d::Point3D::new(
            self.center.x() - r,
            self.center.y() - r,
            self.center.z() - r,
        );
        let max_point = crate::geometry3d::Point3D::new(
            self.center.x() + r,
            self.center.y() + r,
            self.center.z() + r,
        );
        (min_point, max_point)
    }

    fn intersects_sphere(&self, other: &Self) -> bool {
        let center_distance = self.distance_from_center(&other.center);
        center_distance <= self.radius + other.radius + f32::EPSILON
    }

    fn sphere_kind(&self) -> SphereKind {
        if self.radius < f32::EPSILON {
            SphereKind::Degenerate
        } else if (self.radius - 1.0).abs() < f32::EPSILON
            && (self
                .center
                .distance_to(&crate::geometry3d::Point3D::origin()) as f32)
                < f32::EPSILON
        {
            SphereKind::Unit
        } else {
            SphereKind::Standard
        }
    }
}

impl SphericalCoordinates for SphereF64 {
    type Scalar = f64;
    type Point = crate::geometry3d::Point3D;

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

        crate::geometry3d::Point3D::new(
            self.center.x() + x,
            self.center.y() + y,
            self.center.z() + z,
        )
    }
}

impl SphericalCoordinates for SphereF32 {
    type Scalar = f32;
    type Point = crate::geometry3d::Point3D;

    fn cartesian_to_spherical(
        &self,
        point: &Self::Point,
    ) -> (Self::Scalar, Self::Scalar, Self::Scalar) {
        let relative = *point - self.center;
        let x = relative.x() as f32;
        let y = relative.y() as f32;
        let z = relative.z() as f32;

        let r = (x * x + y * y + z * z).sqrt();
        let theta = y.atan2(x);
        let phi = if r > f32::EPSILON {
            (z / r).acos()
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

        crate::geometry3d::Point3D::new(
            self.center.x() + x as f64,
            self.center.y() + y as f64,
            self.center.z() + z as f64,
        )
    }
}

impl fmt::Display for SphereF64 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "SphereF64(center: {:?}, radius: {})",
            self.center, self.radius
        )
    }
}

impl fmt::Display for SphereF32 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "SphereF32(center: {:?}, radius: {})",
            self.center, self.radius
        )
    }
}

// 型エイリアス
pub type Sphere = SphereF64; // デフォルトはf64
pub type Sphere3D = SphereF64; // 後方互換性のため
